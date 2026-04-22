use reqwest::multipart;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub async fn transcribe_groq(api_key: &str, audio_path: &PathBuf) -> Result<String, String> {
    if api_key.is_empty() {
        return Err("Groq API key not set. Please enter your API key in settings.".to_string());
    }

    let audio_bytes = std::fs::read(audio_path)
        .map_err(|e| format!("Failed to read audio file: {}", e))?;

    let audio_size_mb = audio_bytes.len() as f64 / (1024.0 * 1024.0);
    println!("[Flowtype] Uploading {:.2}MB audio to Groq...", audio_size_mb);

    // Groq's limit is 25MB per file for free tier
    if audio_bytes.len() > 25 * 1024 * 1024 {
        return Err(format!(
            "Recording too long ({:.1}MB) — Groq limit is 25MB. Keep dictations under ~15 minutes.",
            audio_size_mb
        ));
    }

    let file_part = multipart::Part::bytes(audio_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let form = multipart::Form::new()
        .text("model", "whisper-large-v3-turbo")
        .text("language", "en")
        .text("response_format", "json")
        .part("file", file_part);

    // Hard 90s timeout on the entire request — guarantees we never hang forever
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|e| format!("HTTP client build failed: {}", e))?;

    let start = Instant::now();
    let response = client
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                format!("Groq request timed out after 90s. Try a shorter recording or check your network.")
            } else {
                format!("Groq API request failed: {}", e)
            }
        })?;
    println!("[Flowtype] Groq responded in {:.2}s", start.elapsed().as_secs_f64());

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Groq API error ({}): {}", status, body));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Groq response: {}", e))?;

    json["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or("No 'text' field in Groq response".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_api_key() {
        let path = PathBuf::from("/tmp/test.wav");
        let result = transcribe_groq("", &path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key not set"));
    }
}
