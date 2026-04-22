# Flowtype

### Voice-to-text for your Mac. Hold a key, speak, clean text appears.

Stop typing. Flowtype transcribes your voice and pastes cleaned-up text into any app — Slack, Gmail, Notes, Claude, ChatGPT, anywhere.

**Made by [Tenfold Marketing](https://tenfoldmarketing.com).**

---

## 👉 [Download Flowtype for Mac](https://github.com/tenfoldmarc/flowtype/releases/latest)

Free. Apple Silicon (M1/M2/M3/M4). ~7 MB.

---

## Features

- **Global hotkey** — `Cmd+Shift+Space` from anywhere
- **AI cleanup** — removes ums and fixes grammar automatically via Llama-3.3
- **Custom dictionary** — preserves your brand names and jargon exactly as you want them
- **Fast** — transcribes in 1-3 seconds on average
- **Works everywhere** — any Mac app that accepts text
- **Privacy-respecting** — audio never stored, only your Groq key is used
- **Tiny** — 7MB download, near-instant startup

---

## Requirements

- **Apple Silicon Mac** (M1/M2/M3/M4) — macOS 12 or later
- **Free [Groq API key](https://console.groq.com/keys)** — takes 30 seconds, no credit card required

---

## Install

1. [Download the latest DMG](https://github.com/tenfoldmarc/flowtype/releases/latest) (click the `.dmg` file in the Assets section)
2. Open the DMG and drag **Flowtype.app** to your `/Applications` folder
3. Open Flowtype from Spotlight (`Cmd+Space` → type "Flowtype")

### Getting past macOS security warning (one-time)

Because Flowtype isn't Apple-signed yet, macOS blocks it the first time. Here's the 30-second fix:

**Option A — clicks only:**
1. Double-click Flowtype → macOS shows "Not Opened" warning → click **Done**
2. Open **System Settings → Privacy & Security**
3. Scroll down to **Security** → click **Open Anyway** next to the Flowtype message
4. Enter your password → Flowtype launches

**Option B — one terminal command (faster):**
Open Terminal and run:
```bash
xattr -d com.apple.quarantine /Applications/Flowtype.app
```
Now Flowtype opens normally, no more warnings.

### Once it's open

4. The onboarding wizard walks you through:
   - Pasting your Groq API key
   - Granting Microphone permission (macOS will prompt)
   - Granting Accessibility permission (macOS will prompt — this is what lets Flowtype paste into other apps)
5. Hit `Cmd+Shift+Space`, speak a sentence, hit it again — clean text pastes wherever your cursor is

---

## Usage

**Toggle mode (default):** Hit the hotkey to start recording, hit it again to stop. Best for longer dictations.

**Push-to-talk mode:** Hold the hotkey while speaking, release to stop. Best for short messages.

Switch modes in **Settings → Recording**.

### Custom dictionary

Add brand names, product names, or technical jargon in **Settings → AI Cleanup → Custom Dictionary** (one per line). Flowtype will preserve these exactly — no mis-transcriptions of your own terminology.

### Writing style

Set your preferred tone in **Settings → AI Cleanup → Writing Style**:
- **Natural** — balanced (default)
- **Formal** — polished, professional
- **Casual** — conversational
- **Concise** — tight, no fluff

---

## How it works

1. **Record** — your mic captures audio while you hold the hotkey
2. **Transcribe** — audio is sent to Groq's Whisper-large-v3-turbo model (fastest Whisper available, ~150x realtime)
3. **Clean up** — the raw transcription is polished by Llama-3.3 (removes filler words, fixes grammar, respects your custom dictionary)
4. **Paste** — cleaned text is auto-pasted via Cmd+V into whatever app is focused

Your audio is never stored — it's streamed to Groq, transcribed, and discarded immediately.

---

## Free tier reality check

Groq's free tier gives every user:
- **2,000 transcription requests/day** (more than enough for heavy dictation)
- **1,000 AI cleanup calls/day**
- **28,800 seconds of audio/day** (8 hours)
- **100,000 Llama tokens/day**

Typical power-user dictation (50-150 messages/day) uses less than 15% of these limits.

---

## Privacy

- Audio is **never stored** on your machine or on Tenfold's servers (Tenfold has no servers — Flowtype talks directly to Groq)
- Your Groq API key lives in `~/Library/Application Support/com.tenfoldmarketing.flowtype/config.json` on your Mac only
- We don't collect analytics, telemetry, or crash reports
- Source code is open for review

---

## Credits

Built with:
- [Tauri](https://tauri.app) — lightweight Rust-based app framework
- [OpenAI Whisper](https://openai.com/research/whisper) via [Groq](https://groq.com) — transcription
- [Meta Llama 3.3](https://llama.com) via Groq — cleanup

Inspired by [Wispr Flow](https://wisprflow.ai).

Initial Tauri scaffolding based on [typr](https://github.com/albertshiney/typr) by Albert Shiney (MIT-licensed).

---

## License

MIT — see [LICENSE](./LICENSE).

---

## About Tenfold Marketing

[Tenfold Marketing](https://tenfoldmarketing.com) teaches non-technical people how to actually use AI. No jargon, no fluff, no wasted time. Follow [@tenfoldmarc](https://instagram.com/tenfoldmarc) for daily AI automation builds.
