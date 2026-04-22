import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface Settings {
  microphone: string;
  engine: string;
  whisperModel: string;
  groqApiKey: string;
  recordingMode: string;
  hotkey: string;
  aiCleanupEnabled: boolean;
  cleanupStyle: string;
  customDictionary: string[];
  hasOnboarded: boolean;
}

interface MicDevice {
  name: string;
  is_default: boolean;
}

// DOM elements
const statusDot = document.getElementById("status-dot")!;
const statusText = document.getElementById("status-text")!;
const micSelect = document.getElementById("mic-select") as HTMLSelectElement;
const groqKey = document.getElementById("groq-key") as HTMLInputElement;
const modeToggle = document.getElementById("mode-toggle")!;
const modePtt = document.getElementById("mode-ptt")!;
const hotkeyBtn = document.getElementById("hotkey-btn") as HTMLButtonElement;
const cleanupOn = document.getElementById("cleanup-on")!;
const cleanupOff = document.getElementById("cleanup-off")!;
const dictionary = document.getElementById("dictionary") as HTMLTextAreaElement;
const cleanupStyle = document.getElementById("cleanup-style") as HTMLSelectElement;
const btnCheckUpdates = document.getElementById("btn-check-updates")!;

// Section navigation
const navItems = document.querySelectorAll(".nav-item");
const sections = document.querySelectorAll(".content-section");

navItems.forEach((item) => {
  item.addEventListener("click", () => {
    const target = item.getAttribute("data-section");
    navItems.forEach((n) => n.classList.remove("active"));
    sections.forEach((s) => s.classList.remove("active"));
    item.classList.add("active");
    document.getElementById(`section-${target}`)?.classList.add("active");
  });
});

// Window drag
const titlebar = document.getElementById("titlebar")!;
const sidebar = document.getElementById("sidebar")!;
const appWindow = getCurrentWindow();

titlebar.addEventListener("mousedown", (e) => {
  if ((e.target as HTMLElement).closest("button, select, input, textarea, a, .nav-item")) return;
  appWindow.startDragging();
});

sidebar.addEventListener("mousedown", (e) => {
  if ((e.target as HTMLElement).closest("button, select, input, textarea, a, .nav-item")) return;
  appWindow.startDragging();
});

let currentSettings: Settings;

async function loadSettings() {
  currentSettings = await invoke<Settings>("get_settings");

  const mics = await invoke<MicDevice[]>("list_microphones");
  micSelect.innerHTML = "";
  mics.forEach((mic) => {
    const option = document.createElement("option");
    option.value = mic.name;
    option.textContent = mic.name + (mic.is_default ? " (default)" : "");
    micSelect.appendChild(option);
  });
  micSelect.value = currentSettings.microphone;

  // Engine is always cloud in v1
  currentSettings.engine = "cloud";
  groqKey.value = currentSettings.groqApiKey;
  setRecordingMode(currentSettings.recordingMode);
  hotkeyBtn.textContent = prettyHotkey(currentSettings.hotkey);

  setCleanup(currentSettings.aiCleanupEnabled);
  cleanupStyle.value = currentSettings.cleanupStyle || "natural";
  dictionary.value = (currentSettings.customDictionary || []).join("\n");
}

function setRecordingMode(mode: string) {
  currentSettings.recordingMode = mode;
  modeToggle.classList.toggle("active", mode === "toggle");
  modePtt.classList.toggle("active", mode === "push-to-talk");
}

function setCleanup(enabled: boolean) {
  currentSettings.aiCleanupEnabled = enabled;
  cleanupOn.classList.toggle("active", enabled);
  cleanupOff.classList.toggle("active", !enabled);
}

async function saveSettings() {
  currentSettings.microphone = micSelect.value;
  currentSettings.groqApiKey = groqKey.value;
  currentSettings.cleanupStyle = cleanupStyle.value;
  currentSettings.customDictionary = dictionary.value
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
  await invoke("save_settings", { settings: currentSettings });
}

micSelect.addEventListener("change", () => saveSettings());
groqKey.addEventListener("change", () => saveSettings());

btnCheckUpdates.addEventListener("click", async () => {
  await invoke("plugin:shell|open", { path: "https://github.com/tenfoldmarc/flowtype/releases/latest" });
});

// ── Hotkey recording ───────────────────────────────────
function prettyHotkey(h: string): string {
  return h
    .replace("CmdOrCtrl", "⌘")
    .replace("Cmd", "⌘")
    .replace("Control", "⌃")
    .replace("Ctrl", "⌃")
    .replace("Shift", "⇧")
    .replace("Alt", "⌥")
    .replace("Option", "⌥")
    .replace(/\+/g, "");
}

let isRecordingHotkey = false;

hotkeyBtn.addEventListener("click", () => {
  if (isRecordingHotkey) return;
  isRecordingHotkey = true;
  hotkeyBtn.textContent = "Press keys…";
  hotkeyBtn.style.borderColor = "var(--accent)";
  hotkeyBtn.style.color = "var(--accent)";
});

document.addEventListener("keydown", async (e) => {
  if (!isRecordingHotkey) return;
  e.preventDefault();
  e.stopPropagation();

  // Ignore if only modifiers pressed
  const ignoredKeys = ["Meta", "Control", "Alt", "Shift", "Escape"];
  if (ignoredKeys.includes(e.key)) {
    if (e.key === "Escape") {
      isRecordingHotkey = false;
      hotkeyBtn.textContent = prettyHotkey(currentSettings.hotkey);
      hotkeyBtn.style.borderColor = "";
      hotkeyBtn.style.color = "";
    }
    return;
  }

  const parts: string[] = [];
  if (e.metaKey) parts.push("CmdOrCtrl");
  if (e.ctrlKey && !e.metaKey) parts.push("CmdOrCtrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");

  if (parts.length === 0) {
    hotkeyBtn.textContent = "Need a modifier (⌘/⌃/⌥)";
    setTimeout(() => {
      hotkeyBtn.textContent = prettyHotkey(currentSettings.hotkey);
      hotkeyBtn.style.borderColor = "";
      hotkeyBtn.style.color = "";
      isRecordingHotkey = false;
    }, 1500);
    return;
  }

  let keyName = e.key;
  if (keyName === " ") keyName = "Space";
  else if (keyName.length === 1) keyName = keyName.toUpperCase();

  const newHotkey = [...parts, keyName].join("+");

  try {
    await invoke("change_hotkey", { hotkey: newHotkey });
    currentSettings.hotkey = newHotkey;
    hotkeyBtn.textContent = prettyHotkey(newHotkey);
    hotkeyBtn.style.borderColor = "";
    hotkeyBtn.style.color = "";
  } catch (err) {
    console.error("Hotkey change failed:", err);
    hotkeyBtn.textContent = "Invalid — click to retry";
    setTimeout(() => {
      hotkeyBtn.textContent = prettyHotkey(currentSettings.hotkey);
      hotkeyBtn.style.borderColor = "";
      hotkeyBtn.style.color = "";
    }, 1500);
  }
  isRecordingHotkey = false;
});
modeToggle.addEventListener("click", () => { setRecordingMode("toggle"); saveSettings(); });
modePtt.addEventListener("click", () => { setRecordingMode("push-to-talk"); saveSettings(); });

cleanupOn.addEventListener("click", () => { setCleanup(true); saveSettings(); });
cleanupOff.addEventListener("click", () => { setCleanup(false); saveSettings(); });
cleanupStyle.addEventListener("change", () => saveSettings());
dictionary.addEventListener("blur", () => saveSettings());

listen<string>("recording-state", (event) => {
  const state = event.payload;
  statusDot.className = "";
  if (state === "Recording") {
    statusDot.classList.add("recording");
    statusText.textContent = "Recording...";
  } else if (state === "Transcribing") {
    statusDot.classList.add("transcribing");
    statusText.textContent = "Transcribing...";
  } else {
    statusDot.classList.add("ready");
    statusText.textContent = "Ready";
  }
});


// ── Onboarding wizard ───────────────────────────────
const onboarding = document.getElementById("onboarding")!;
const onboardingStepLabel = document.getElementById("onboarding-step-label")!;
const onboardingTitle = document.getElementById("onboarding-title")!;
const onboardingDesc = document.getElementById("onboarding-desc")!;
const onboardingBody = document.getElementById("onboarding-body")!;
const onboardingNext = document.getElementById("onboarding-next") as HTMLButtonElement;
const onboardingSkip = document.getElementById("onboarding-skip")!;
const onboardingDots = document.querySelectorAll(".onboarding-dot");

let onboardingStep = 1;
const ONBOARDING_STEPS = 3;

function renderOnboardingStep() {
  onboardingDots.forEach((d, i) => d.classList.toggle("active", i < onboardingStep));
  onboardingStepLabel.textContent = `Step ${onboardingStep} of ${ONBOARDING_STEPS}`;

  if (onboardingStep === 1) {
    onboardingTitle.textContent = "Welcome to Flowtype";
    onboardingDesc.textContent = "Voice-to-text for your Mac. Hold your hotkey, speak naturally, and cleaned-up text pastes wherever you're focused. Let's get you set up in 60 seconds.";
    onboardingBody.innerHTML = `
      <div style="display:flex; flex-direction:column; gap:14px; font-size:14px; color:var(--text-secondary); line-height:1.6;">
        <div style="display:flex; gap:12px; align-items:flex-start;"><span style="color:var(--accent); font-family:var(--font-mono); font-size:12px; margin-top:2px;">01</span> <span>Speak into any app — Slack, Gmail, Notes, Claude.</span></div>
        <div style="display:flex; gap:12px; align-items:flex-start;"><span style="color:var(--accent); font-family:var(--font-mono); font-size:12px; margin-top:2px;">02</span> <span>AI removes filler words, fixes grammar, respects your custom terms.</span></div>
        <div style="display:flex; gap:12px; align-items:flex-start;"><span style="color:var(--accent); font-family:var(--font-mono); font-size:12px; margin-top:2px;">03</span> <span>Runs in the cloud (fast) or on-device (private, offline).</span></div>
      </div>
    `;
    onboardingNext.textContent = "Let's go →";
  } else if (onboardingStep === 2) {
    onboardingTitle.textContent = "Add your Groq API key";
    onboardingDesc.innerHTML = `Flowtype uses Groq for lightning-fast transcription + AI cleanup. Get your free key at <a href="https://console.groq.com/keys" target="_blank" style="color:var(--accent); text-decoration:underline;">console.groq.com/keys</a> — takes 30 seconds, no credit card needed.`;
    onboardingBody.innerHTML = `
      <label>Paste your key here</label>
      <input type="password" id="onboard-key" placeholder="gsk_..." value="${currentSettings?.groqApiKey || ""}" />
      <div style="margin-top:10px; font-size:12px; color:var(--text-tertiary); font-family:var(--font-mono);">Can be added later in Settings → Engine</div>
    `;
    onboardingNext.textContent = "Continue →";
  } else {
    onboardingTitle.textContent = "You're ready";
    onboardingDesc.textContent = "Everything is configured. One last thing — macOS will ask for Accessibility permission when you first dictate. Click Allow — it's what lets Flowtype paste into other apps.";
    onboardingBody.innerHTML = `
      <div style="background:var(--bg-base); border:1px solid var(--border); border-radius:var(--radius); padding:20px; margin-bottom:16px;">
        <div style="font-size:12px; color:var(--text-secondary); margin-bottom:8px; font-family:var(--font-mono); text-transform:uppercase; letter-spacing:0.1em;">How to dictate</div>
        <div style="display:flex; align-items:center; gap:10px; font-size:14px;">
          <kbd style="padding:5px 10px; background:var(--surface); border:1px solid var(--border-strong); border-radius:var(--radius-sm); font-family:var(--font-mono); font-size:11px;">${(currentSettings?.hotkey || "CmdOrCtrl+Shift+Space").replace("CmdOrCtrl", "⌘")}</kbd>
          <span style="color:var(--text-secondary);">→ speak → release → text appears</span>
        </div>
      </div>
    `;
    onboardingNext.textContent = "Start dictating 🎤";
  }
}

async function completeOnboarding() {
  const keyInput = document.getElementById("onboard-key") as HTMLInputElement | null;
  if (keyInput && keyInput.value.trim()) {
    currentSettings.groqApiKey = keyInput.value.trim();
    currentSettings.engine = "cloud";
    groqKey.value = currentSettings.groqApiKey;
  }
  currentSettings.hasOnboarded = true;
  await saveSettings();
  onboarding.classList.remove("active");
}

onboardingNext.addEventListener("click", () => {
  if (onboardingStep === 2) {
    const keyInput = document.getElementById("onboard-key") as HTMLInputElement;
    if (keyInput?.value.trim()) {
      currentSettings.groqApiKey = keyInput.value.trim();
    }
  }
  if (onboardingStep < ONBOARDING_STEPS) {
    onboardingStep++;
    renderOnboardingStep();
  } else {
    completeOnboarding();
  }
});

onboardingSkip.addEventListener("click", completeOnboarding);

async function init() {
  await loadSettings();
  if (!currentSettings.hasOnboarded) {
    onboardingStep = 1;
    renderOnboardingStep();
    onboarding.classList.add("active");
  }
}

init();
