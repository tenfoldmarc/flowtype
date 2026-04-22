use serde_json::json;

/// Whisper v3 has well-documented silence hallucinations — when given silence or near-silence,
/// it confidently returns common phrases from its training data (mostly YouTube). Return true
/// if the transcription looks like one of these, so the caller can discard it.
pub fn is_whisper_hallucination(text: &str) -> bool {
    let lower = text.trim().to_lowercase();
    let normalized = lower.trim_end_matches(|c: char| c == '.' || c == '!' || c == '?' || c == ',').trim();
    // Only the most definite Whisper-on-silence artifacts — YouTube-training outputs.
    // Removed: "okay", "ok", "yeah", "hmm", "oh", "um", "uh", "you", "bye" — users legitimately say these.
    let known: &[&str] = &[
        "thank you",
        "thanks for watching",
        "thank you for watching",
        "please subscribe",
        "like and subscribe",
        ".",
        "..",
        "...",
        "",
    ];
    known.contains(&normalized)
}

/// Basic deterministic cleanup — used as fallback when AI cleanup is off or fails.
/// Trims, normalizes whitespace, capitalizes sentences, ensures ending punctuation.
pub fn basic_cleanup(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let normalized: String = trimmed
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in normalized.chars() {
        if capitalize_next && ch.is_alphabetic() {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
            if ch == '.' || ch == '!' || ch == '?' {
                capitalize_next = true;
            }
        }
    }

    if let Some(last) = result.chars().last() {
        if !matches!(last, '.' | '!' | '?') {
            result.push('.');
        }
    }

    result
}

/// Detect conversational drift — when Llama replies AS IF the input was a request to it
/// rather than output to clean. Happens when input is too short or ambiguous.
fn looks_like_conversational_reply(output: &str, input: &str) -> bool {
    let lower = output.to_lowercase();
    let bad_prefixes = [
        "i'm ready",
        "i am ready",
        "please provide",
        "sure, i can",
        "sure! i can",
        "of course",
        "happy to help",
        "i'd be happy",
        "i would be happy",
        "here is the cleaned",
        "here's the cleaned",
        "here is the",
        "here's the",
        "the cleaned text",
        "cleaned version:",
        "as an ai",
        "i cannot",
        "i can't clean",
        "it seems",
        "it appears",
    ];
    if bad_prefixes.iter().any(|p| lower.starts_with(p)) {
        return true;
    }
    // If output is meaningfully longer than input (>2.5x) and input was short, likely drifted
    if input.len() < 30 && output.len() > input.len() * 3 {
        return true;
    }
    false
}

/// AI-powered cleanup via Groq Llama-3.3. Removes filler words, fixes grammar,
/// preserves custom dictionary terms, applies writing style.
/// Falls back to basic_cleanup on any error or suspected conversational drift.
pub async fn ai_cleanup(
    text: &str,
    api_key: &str,
    style: &str,
    dictionary: &[String],
) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Skip AI cleanup for very short transcriptions — high risk of Llama conversational drift,
    // and basic_cleanup is perfectly fine for single words / short phrases anyway
    if trimmed.len() < 8 {
        return basic_cleanup(trimmed);
    }

    if api_key.is_empty() {
        return basic_cleanup(trimmed);
    }

    match call_groq_cleanup(trimmed, api_key, style, dictionary).await {
        Ok(cleaned) => {
            if looks_like_conversational_reply(&cleaned, trimmed) {
                eprintln!(
                    "[Flowtype] Detected Llama conversational drift, falling back. Output was: {}",
                    cleaned.chars().take(80).collect::<String>()
                );
                basic_cleanup(trimmed)
            } else {
                cleaned
            }
        }
        Err(e) => {
            eprintln!("[Flowtype] AI cleanup failed, using basic: {}", e);
            basic_cleanup(trimmed)
        }
    }
}

async fn call_groq_cleanup(
    text: &str,
    api_key: &str,
    style: &str,
    dictionary: &[String],
) -> Result<String, String> {
    let style_guide = match style {
        "formal" => "Use formal, professional language.",
        "casual" => "Use casual, conversational language.",
        "concise" => "Make it as concise as possible while preserving meaning.",
        _ => "Keep the original tone and voice natural.",
    };

    let dictionary_guide = if dictionary.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nPreserve these terms EXACTLY as written (case-sensitive, no corrections): {}",
            dictionary.join(", ")
        )
    };

    let system_prompt = format!(
        "You are a transcription touch-up function, NOT a chatbot. You receive voice dictation \
        wrapped in <transcript> tags and return the lightly-cleaned version. Minimal intervention.\n\n\
        YOUR ONE JOB — do these SPECIFIC things and NOTHING MORE:\n\
        1. Remove obvious filler words ONLY when they're clearly verbal tics: 'um', 'uh', 'like' \
        (filler usage only, not as real word), 'you know' (when repeated), 'I mean' (when repeated).\n\
        2. Add punctuation where sentences clearly end.\n\
        3. Capitalize sentence starts and proper nouns.\n\
        4. Fix only BLATANT grammar errors that obviously weren't intended.\n\n\
        YOU MUST NOT:\n\
        - Rephrase or restructure sentences\n\
        - Substitute synonyms for the speaker's words\n\
        - Reorder ideas or combine sentences\n\
        - Add words or context that wasn't said\n\
        - Remove content beyond the filler words listed in rule 1\n\
        - Output quotes, labels, preambles, or any metadata\n\
        - Respond conversationally (never 'I'm ready', 'Sure!', 'Here is...')\n\n\
        When in doubt: DO NOTHING. The speaker's words come out the other side intact.\n\
        {}{}\n\n\
        EXAMPLES:\n\
        <transcript>um so like i went for my motorcycle exam today and i passed it which was really cool</transcript>\n\
        So I went for my motorcycle exam today and I passed it, which was really cool.\n\n\
        <transcript>hello</transcript>\n\
        Hello.\n\n\
        <transcript>is this working</transcript>\n\
        Is this working?\n\n\
        <transcript>okay so this is a short one</transcript>\n\
        Okay, so this is a short one.\n\n\
        <transcript>i need to charge my apple watch and also my phone</transcript>\n\
        I need to charge my Apple Watch and also my phone.",
        style_guide, dictionary_guide
    );

    let user_msg = format!("<transcript>{}</transcript>", text);

    let body = json!({
        "model": "llama-3.3-70b-versatile",
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_msg}
        ],
        "temperature": 0.1,
        "max_tokens": 1024
    });

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Groq request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Groq error ({}): {}", status, body));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Parse failed: {}", e))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in Groq response")?
        .trim()
        .to_string();

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_trim() {
        assert_eq!(basic_cleanup("  hello world  "), "Hello world.");
    }

    #[test]
    fn test_basic_capitalize() {
        assert_eq!(basic_cleanup("hello. world"), "Hello. World.");
    }

    #[test]
    fn test_basic_empty() {
        assert_eq!(basic_cleanup(""), "");
    }

    #[tokio::test]
    async fn test_ai_cleanup_empty_key_falls_back() {
        let result = ai_cleanup("hello world", "", "natural", &[]).await;
        assert_eq!(result, "Hello world.");
    }

    #[tokio::test]
    async fn test_ai_cleanup_empty_text() {
        let result = ai_cleanup("", "fake-key", "natural", &[]).await;
        assert_eq!(result, "");
    }
}
