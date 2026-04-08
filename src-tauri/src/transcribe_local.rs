use std::path::PathBuf;
use std::process::Command;

pub fn transcribe_local(
    whisper_binary: &PathBuf,
    model_path: &PathBuf,
    audio_path: &PathBuf,
) -> Result<String, String> {
    if !model_path.exists() {
        return Err("Whisper model not found. Please download a model first.".to_string());
    }

    let output = Command::new(whisper_binary)
        .args([
            "-m",
            model_path.to_str().unwrap(),
            "-f",
            audio_path.to_str().unwrap(),
            "--no-timestamps",
            "-l",
            "en",
        ])
        .output()
        .map_err(|e| format!("Failed to run whisper.cpp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("whisper.cpp failed: {}", stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(text)
}

pub fn model_filename(model_size: &str) -> String {
    format!("ggml-{}.bin", model_size)
}

pub fn model_download_url(model_size: &str) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_filename() {
        assert_eq!(model_filename("small"), "ggml-small.bin");
        assert_eq!(model_filename("medium"), "ggml-medium.bin");
    }

    #[test]
    fn test_model_download_url() {
        assert_eq!(
            model_download_url("small"),
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
        );
    }

    #[test]
    fn test_transcribe_missing_model() {
        let binary = PathBuf::from("/nonexistent/whisper");
        let model = PathBuf::from("/nonexistent/model.bin");
        let audio = PathBuf::from("/nonexistent/audio.wav");
        let result = transcribe_local(&binary, &model, &audio);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
