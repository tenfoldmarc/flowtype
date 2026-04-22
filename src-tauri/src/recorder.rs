use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

use crate::audio::AudioRecorder;
use crate::cleanup::{ai_cleanup, basic_cleanup, is_whisper_hallucination};
use crate::paste::paste_text;
use crate::settings::Settings;
use crate::transcribe_local;
use crate::transcribe_groq;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum RecordingState {
    Ready,
    Recording,
    Transcribing,
}

fn update_overlay(app: &AppHandle, state: &RecordingState) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let class = match state {
            RecordingState::Ready => "mic",
            RecordingState::Recording => "mic recording",
            RecordingState::Transcribing => "mic transcribing",
        };
        let js = format!("document.getElementById('mic').className = '{}';", class);
        let _ = overlay.eval(&js);
    }
}

pub struct Recorder {
    state: Arc<Mutex<RecordingState>>,
    audio_recorder: Arc<Mutex<AudioRecorder>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RecordingState::Ready)),
            audio_recorder: Arc::new(Mutex::new(AudioRecorder::new())),
        }
    }

    pub fn get_state(&self) -> RecordingState {
        self.state.lock().unwrap().clone()
    }

    pub fn start_recording(&self, app: &AppHandle, mic_name: &str) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if *state != RecordingState::Ready {
            return Err("Already recording or transcribing".to_string());
        }

        let mut recorder = self.audio_recorder.lock().unwrap();
        recorder.start(mic_name)?;

        *state = RecordingState::Recording;
        let _ = app.emit("recording-state", RecordingState::Recording);
        update_overlay(app, &RecordingState::Recording);
        Ok(())
    }

    pub async fn stop_and_transcribe(
        &self,
        app: &AppHandle,
        settings: &Settings,
        app_dir: &PathBuf,
    ) -> Result<String, String> {
        // Wrap the real work in an inner function so we can ALWAYS reset state on any exit path —
        // previously, errors returned early and left the app stuck in Transcribing forever.
        let result = self.do_stop_and_transcribe(app, settings, app_dir).await;

        // Unconditional state reset — success or failure, we go back to Ready
        {
            let mut state = self.state.lock().unwrap();
            *state = RecordingState::Ready;
        }
        let _ = app.emit("recording-state", RecordingState::Ready);
        update_overlay(app, &RecordingState::Ready);

        if let Err(ref e) = result {
            eprintln!("[Flowtype] Transcription failed: {}", e);
        }
        result
    }

    async fn do_stop_and_transcribe(
        &self,
        app: &AppHandle,
        settings: &Settings,
        app_dir: &PathBuf,
    ) -> Result<String, String> {
        let overall_start = std::time::Instant::now();

        {
            let mut state = self.state.lock().unwrap();
            if *state != RecordingState::Recording {
                return Err("Not currently recording".to_string());
            }
            *state = RecordingState::Transcribing;
            let _ = app.emit("recording-state", RecordingState::Transcribing);
            update_overlay(app, &RecordingState::Transcribing);
        }

        let temp_path = app_dir.join("temp_recording.wav");

        {
            let mut recorder = self.audio_recorder.lock().unwrap();
            recorder.stop_and_save(&temp_path)?;
        }

        // Transcribe
        let transcribe_start = std::time::Instant::now();
        let raw_text = match settings.engine.as_str() {
            "local" => {
                let model_path = app_dir.join(transcribe_local::model_filename(&settings.whisper_model));
                transcribe_local::transcribe_local(app, &model_path, &temp_path).await?
            }
            "cloud" => {
                transcribe_groq::transcribe_groq(&settings.groq_api_key, &temp_path).await?
            }
            _ => return Err(format!("Unknown engine: {}", settings.engine)),
        };
        println!(
            "[Flowtype] Transcription done in {:.2}s ({} chars)",
            transcribe_start.elapsed().as_secs_f64(),
            raw_text.len()
        );

        let _ = std::fs::remove_file(&temp_path);

        if is_whisper_hallucination(&raw_text) {
            println!("[Flowtype] Discarded silence hallucination: '{}'", raw_text.trim());
            return Ok(String::new());
        }

        // AI cleanup (may hit network)
        let cleanup_start = std::time::Instant::now();
        let cleaned = if settings.ai_cleanup_enabled && !settings.groq_api_key.is_empty() {
            ai_cleanup(
                &raw_text,
                &settings.groq_api_key,
                &settings.cleanup_style,
                &settings.custom_dictionary,
            )
            .await
        } else {
            basic_cleanup(&raw_text)
        };
        println!(
            "[Flowtype] Cleanup done in {:.2}s",
            cleanup_start.elapsed().as_secs_f64()
        );

        if !cleaned.is_empty() {
            paste_text(&cleaned)?;
        }

        println!(
            "[Flowtype] Total pipeline: {:.2}s",
            overall_start.elapsed().as_secs_f64()
        );
        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_ready() {
        let recorder = Recorder::new();
        assert_eq!(recorder.get_state(), RecordingState::Ready);
    }
}
