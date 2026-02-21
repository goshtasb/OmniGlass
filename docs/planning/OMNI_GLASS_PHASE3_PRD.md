# Omni-Glass Phase 3: PRD & Engineering Brief

**Document Status:** Draft for Review  
**Baseline:** `v0.4.0-phase2c` — 7 built-in actions, MCP plugin system, macOS sandbox, text launcher, 74 tests  
**Date:** February 2026

---

## 1. Where We Are

Omni-Glass is a working product with a working platform. Phase 1 built the core loop (snip → OCR → LLM → actions). Phase 2 opened it up (MCP plugins, sandbox, text launcher). The README is live, the repo is public, and a developer can clone it, build it, and extend it today.

But three hard truths constrain what happens next:

**Truth 1: The product only works on macOS.** Windows code compiles in CI but has never been tested on real hardware. Linux has no code at all. The sandbox only works on macOS. Every user we gain on macOS is a user we could lose when they switch to their work Windows machine and it doesn't work.

**Truth 2: The product requires an internet connection and a paid API key.** Every action goes through Claude or Gemini. A developer on an airplane, in a secure facility, or simply watching their API bill has no alternative. Local LLM support was the most-requested item deferred from Phase 2.

**Truth 3: The plugin ecosystem has exactly one real plugin.** The GitHub Issues plugin works, the template exists, and the developer guide is written. But an ecosystem with one plugin is not an ecosystem. Discovery is "clone a folder from GitHub." Installation is "copy it to a hidden directory." Updates don't exist. This is fine for early adopters, not for growth.

Phase 3 addresses all three truths. The work divides into four pillars, each independent, each with its own gate.

---

## 2. The Four Pillars

| Pillar | Problem | Solution | Timeline |
|--------|---------|----------|----------|
| **A: Cross-Platform** | Only macOS works | Windows runtime verification + Linux port + sandbox on both | Weeks 1-4 |
| **B: Local LLM** | Requires cloud API + internet | llama.cpp integration with Qwen-2.5-3B, provider abstraction | Weeks 3-6 |
| **C: Plugin Registry** | No discovery, manual install | Web-based registry, in-app browse/install, plugin signing | Weeks 5-8 |
| **D: Rich Path** | OCR-only, no UI understanding | RT-DETR + Florence-2 for UI element detection, click targeting | Weeks 7-12 |

Pillars A and B are engineering debt and user demand. Ship them first.  
Pillars C and D are growth and differentiation. Ship them once the foundation is solid.

---

## 3. Pillar A: Cross-Platform (Weeks 1-4)

### 3.1 The Problem

Windows code exists in the repo behind `#[cfg(target_os = "windows")]` guards. It has never been run on real Windows hardware. The CI compiles it, but compilation is not verification. We have no idea if:

- `xcap` captures the screen correctly on Windows 10/11
- `Windows.Media.Ocr` produces usable OCR output
- The tray icon, overlay, and action menu render correctly
- The streaming pipeline hits acceptable latency
- The sandbox (currently macOS-only) can be ported to AppContainer

Linux has zero platform code. No capture, no OCR, no sandbox.

### 3.2 Windows Verification + Fixes (Weeks 1-2)

**Approach:** Spin up a Windows 11 VM (Azure/AWS), clone the repo, build, and run the full pipeline.

**Ticket sequence:**

| Ticket | Description | Est. |
|--------|-------------|------|
| WIN-01 | Set up Windows 11 dev VM with Rust, Node.js, Tauri prerequisites | 0.5 day |
| WIN-02 | Build and run on Windows — document every failure | 1 day |
| WIN-03 | Fix capture issues (xcap on Windows, DPI scaling, multi-monitor) | 2-3 days |
| WIN-04 | Fix OCR issues (Windows.Media.Ocr language packs, accuracy vs Apple Vision) | 1-2 days |
| WIN-05 | Fix UI issues (tray icon, overlay rendering, window positioning) | 1-2 days |
| WIN-06 | Run full verification: 7 E2E tests on Windows, benchmark 3 snips | 1 day |
| WIN-07 | Windows AppContainer sandbox implementation | 3-5 days |

**WIN-07: Windows AppContainer**

Replace the macOS `sandbox-exec` stub on Windows with a real AppContainer sandbox. AppContainer is the Windows equivalent — a low-integrity process token with restricted capabilities.

Implementation approach:

```rust
// src-tauri/src/mcp/sandbox/windows.rs

use windows::Win32::Security::*;

pub async fn spawn_in_appcontainer(
    command: &str,
    args: &[&str],
    plugin_dir: &Path,
    env: &HashMap<String, String>,
    manifest: &PluginManifest,
) -> Result<tokio::process::Child, McpError> {
    // 1. Create an AppContainer profile for this plugin
    // 2. Set capabilities based on manifest permissions:
    //    - internetClient for network access
    //    - documentsLibrary / picturesLibrary for filesystem
    // 3. Create a restricted token
    // 4. Launch the process with the restricted token
    // 5. Environment filtering (already implemented, cross-platform)
}
```

The Windows sandbox follows the same security model as macOS:
- Default deny for user files
- Network access only if declared
- Environment variables filtered
- Declared filesystem paths explicitly granted

**Gate:** All 7 E2E tests pass on Windows. 3-snip benchmark within 2x of macOS latency (Windows OCR is typically slower). AppContainer blocks `%USERPROFILE%\.ssh` and undeclared network access.

### 3.3 Linux Port (Weeks 3-4)

**Approach:** Linux has no platform-specific code yet. We need capture, OCR, and sandbox.

| Ticket | Description | Est. |
|--------|-------------|------|
| LIN-01 | Screen capture — `xcap` already supports X11/Wayland, verify it works | 1 day |
| LIN-02 | OCR — Tesseract integration via `tesseract-rs` crate (no native OCR API on Linux) | 2-3 days |
| LIN-03 | Tray icon + overlay — verify Tauri's Linux rendering (GTK-based) | 1-2 days |
| LIN-04 | Bubblewrap sandbox — full implementation (not stub) | 3-4 days |
| LIN-05 | Full verification: 7 E2E tests on Ubuntu 22.04/24.04 | 1 day |

**LIN-02: Tesseract OCR**

macOS has Apple Vision. Windows has Windows.Media.Ocr. Linux has Tesseract. The OCR abstraction layer from Week 3 (`#[cfg]` dispatch) already handles this — we need to implement the Linux branch.

```rust
// src-tauri/src/ocr/tesseract.rs

use tesseract::Tesseract;

pub fn recognize_text(image_path: &str, level: RecognitionLevel) -> OcrOutput {
    let mut tess = Tesseract::new(None, Some("eng"))
        .expect("Failed to initialize Tesseract");
    
    tess.set_image(image_path);
    
    // Map RecognitionLevel to Tesseract PSM mode
    match level {
        RecognitionLevel::Fast => tess.set_variable("tessedit_pageseg_mode", "6"),
        RecognitionLevel::Accurate => tess.set_variable("tessedit_pageseg_mode", "3"),
    };
    
    let text = tess.get_text().unwrap_or_default();
    let confidence = tess.mean_text_conf() as f64 / 100.0;
    
    OcrOutput {
        text,
        confidence,
        latency_ms: 0.0, // measured by caller
        recognition_level: level.to_string(),
    }
}
```

**New crate dependency:** `tesseract` (Rust bindings for libtesseract). Requires `libtesseract-dev` system package on Ubuntu.

**LIN-04: Bubblewrap Sandbox**

Bubblewrap (`bwrap`) provides user-namespace-based sandboxing on Linux. It's the same technology used by Flatpak.

```rust
// src-tauri/src/mcp/sandbox/linux.rs

pub async fn spawn_in_bubblewrap(
    command: &str,
    args: &[&str],
    plugin_dir: &Path,
    env: &HashMap<String, String>,
    manifest: &PluginManifest,
) -> Result<tokio::process::Child, McpError> {
    let mut bwrap_args = vec![
        // Mount root filesystem read-only
        "--ro-bind", "/", "/",
        // Mount /dev for basic device access
        "--dev", "/dev",
        // Mount /proc for process info
        "--proc", "/proc",
        // Create new PID namespace (plugin can't see host processes)
        "--unshare-pid",
        // Create new network namespace if no network declared
    ];
    
    // Network isolation
    if manifest.permissions.network.is_none() {
        bwrap_args.extend(&["--unshare-net"]);
    }
    
    // Wall off home directory (same pattern as macOS)
    let home = dirs::home_dir().unwrap();
    bwrap_args.extend(&["--tmpfs", &home.to_string_lossy()]);
    
    // Re-allow plugin directory
    let plugin_dir_str = plugin_dir.to_string_lossy();
    bwrap_args.extend(&[
        "--bind", &plugin_dir_str, &plugin_dir_str,
    ]);
    
    // Re-allow runtime paths (Node.js/Python, if in home dir)
    // ... same find_runtime_paths() logic as macOS
    
    // Re-allow declared filesystem paths
    if let Some(ref fs_perms) = manifest.permissions.filesystem {
        for perm in fs_perms {
            let expanded = expand_tilde(&perm.path);
            match perm.access.as_str() {
                "read" => bwrap_args.extend(&["--ro-bind", &expanded, &expanded]),
                "read-write" => bwrap_args.extend(&["--bind", &expanded, &expanded]),
                _ => {}
            }
        }
    }
    
    // Plugin temp directory
    let tmp_dir = format!("/tmp/omni-glass-{}", manifest.id);
    std::fs::create_dir_all(&tmp_dir).ok();
    bwrap_args.extend(&["--bind", &tmp_dir, &tmp_dir]);
    
    // Add the actual command
    bwrap_args.push("--");
    bwrap_args.push(command);
    bwrap_args.extend(args);
    
    let child = tokio::process::Command::new("bwrap")
        .args(&bwrap_args)
        .envs(env) // already filtered
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    
    Ok(child)
}
```

**Bubblewrap advantages over sandbox-exec:**

- Actually supports network namespace isolation (per-plugin network deny, not just IP filtering)
- Supports PID namespace (plugin can't see or signal host processes)
- Supports filesystem bind mounts with explicit read-only vs read-write
- Widely deployed (ships with Flatpak on most distros)

**New system dependency:** `bubblewrap` package. Check at startup, warn if missing.

**Gate:** All 7 E2E tests pass on Ubuntu 22.04. Bubblewrap blocks home directory access and undeclared network connections. Tesseract OCR produces usable output for English text.

---

## 4. Pillar B: Local LLM (Weeks 3-6)

### 4.1 The Problem

Every LLM call currently goes to Anthropic's API (or Gemini, once benchmarked). This means:

- No internet = no Omni-Glass
- Every snip costs money (~$0.002 per CLASSIFY + EXECUTE with Haiku)
- OCR text is sent to a cloud provider (privacy concern for sensitive content)
- Latency is bound by network round-trip + API queue time

### 4.2 The Honest Latency Reality

Before committing to local LLM, we need to be honest about the performance tradeoff.

| Model | Type | CLASSIFY Est. | EXECUTE Est. | Total Pipeline |
|-------|------|--------------|-------------|----------------|
| Claude Haiku | Cloud API | ~1.0s | ~2.5s | ~3.5s |
| Gemini Flash | Cloud API | ~0.8s (est.) | ~2.0s (est.) | ~2.8s (est.) |
| Qwen-2.5-3B | Local (M1 Air, 8GB) | ~3-5s | ~5-10s | ~8-15s |
| Qwen-2.5-1.5B | Local (M1 Air, 8GB) | ~2-3s | ~3-6s | ~5-9s |
| Qwen-2.5-3B | Local (M2 Pro, 16GB) | ~1.5-3s | ~3-5s | ~4.5-8s |

**Local models will be 3-5x slower than cloud APIs.** This is not a bug, it's physics. A 3B parameter model running on CPU/integrated GPU cannot match a datacenter GPU cluster. The product must be honest about this: local mode is for privacy and offline use, not for speed.

### 4.3 Implementation: llama.cpp via llama-cpp-rs

**Why llama.cpp:** It's the de facto standard for local LLM inference. It supports GGUF model format, Apple Metal acceleration, CUDA, and CPU fallback. The Rust bindings (`llama-cpp-rs`) are mature.

**Why Qwen-2.5-3B:** Best small model for structured output (JSON), multilingual support, and instruction following. The 3B variant runs on 8GB RAM machines. The 1.5B variant is a fallback for constrained hardware.

| Ticket | Description | Est. |
|--------|-------------|------|
| LOCAL-01 | Add `llama-cpp-rs` to Cargo.toml, conditional compilation behind `local-llm` feature flag | 1 day |
| LOCAL-02 | Implement `local.rs` LLM provider (load model, tokenize, generate, stream) | 3-4 days |
| LOCAL-03 | Model management: download, verify hash, store in `~/.config/omni-glass/models/` | 2 days |
| LOCAL-04 | Settings UI: model selection, download progress, GPU/CPU toggle | 2 days |
| LOCAL-05 | CLASSIFY prompt tuning for Qwen-2.5 (different tokenizer, different system prompt format) | 2-3 days |
| LOCAL-06 | EXECUTE prompt tuning for Qwen-2.5 | 2-3 days |
| LOCAL-07 | Benchmark: 3 snips on M1 Air (8GB), M2 Pro (16GB), Intel i7 (16GB) | 1 day |

**LOCAL-01: Feature Flag Architecture**

The local LLM is behind a Cargo feature flag so users who only want cloud providers don't pay the binary size and compilation cost of bundling llama.cpp.

```toml
# Cargo.toml
[features]
default = []
local-llm = ["llama-cpp-rs"]

[dependencies]
llama-cpp-rs = { version = "0.3", optional = true }
```

```rust
// src-tauri/src/llm/local.rs
#[cfg(feature = "local-llm")]
pub mod local {
    use llama_cpp_rs::*;
    
    pub struct LocalLlm {
        model: Model,
        context: Context,
        model_path: PathBuf,
    }
    
    impl LocalLlm {
        pub fn load(model_path: &Path, params: ModelParams) -> Result<Self, LlmError> {
            let model = Model::load_from_file(model_path, params)?;
            let ctx_params = ContextParams::default()
                .with_n_ctx(2048)       // Context window
                .with_n_batch(512)      // Batch size
                .with_n_threads(4);     // CPU threads
            let context = Context::new(&model, ctx_params)?;
            Ok(Self { model, context, model_path: model_path.to_path_buf() })
        }
        
        pub async fn generate(
            &mut self,
            prompt: &str,
            max_tokens: u32,
            stop_sequences: &[&str],
        ) -> Result<String, LlmError> {
            let tokens = self.model.tokenize(prompt)?;
            self.context.eval(&tokens)?;
            
            let mut output = String::new();
            for _ in 0..max_tokens {
                let next_token = self.context.sample_token()?;
                if next_token == self.model.eos_token() {
                    break;
                }
                let piece = self.model.detokenize(&[next_token])?;
                output.push_str(&piece);
                
                // Check stop sequences
                for stop in stop_sequences {
                    if output.ends_with(stop) {
                        output.truncate(output.len() - stop.len());
                        return Ok(output);
                    }
                }
                
                self.context.eval(&[next_token])?;
            }
            
            Ok(output)
        }
    }
}
```

**LOCAL-02: Provider Integration**

The provider abstraction from Phase 1 (Week 3) already supports multiple providers via a match statement. Adding a local provider:

```rust
// In the provider dispatch
match provider {
    "anthropic" => classify_streaming(ocr_text, confidence, has_code, has_table, plugin_tools).await,
    "gemini" => classify_streaming_gemini(ocr_text, confidence, has_code, has_table, plugin_tools).await,
    #[cfg(feature = "local-llm")]
    "local" => classify_local(ocr_text, confidence, has_code, has_table, plugin_tools).await,
    _ => Err("Unknown provider".to_string()),
}
```

**LOCAL-03: Model Management**

First-run experience:

1. User selects "Local (Qwen-2.5-3B)" in Settings → Provider
2. App checks `~/.config/omni-glass/models/` for the GGUF file
3. If missing, shows a download dialog with progress bar
4. Downloads from Hugging Face (qwen/Qwen2.5-3B-Instruct-GGUF, Q4_K_M quantization)
5. Verifies SHA-256 hash
6. Model loads into memory (~2GB for Q4_K_M)

Model files are large (2-4GB). The download must be resumable and show progress. Use `reqwest` with content-length tracking.

**LOCAL-05/06: Prompt Tuning**

Qwen-2.5 uses a different prompt format than Claude:

```
<|im_start|>system
You are Omni-Glass, a Visual Action Engine...
<|im_end|>
<|im_start|>user
{ocr_text}
<|im_end|>
<|im_start|>assistant
```

The CLASSIFY and EXECUTE prompts need to be adapted:

- Shorter system prompts (smaller context window)
- More explicit JSON formatting instructions (smaller models need more guidance)
- Fewer examples (token budget is tighter)
- Explicit stop sequences to prevent rambling

This is the hardest part of the local LLM integration. A 3B model is not as capable as Claude Haiku at structured output. Expect 2-3 days of prompt iteration to get reliable JSON from CLASSIFY and EXECUTE.

**Gate:** Local CLASSIFY returns valid ActionMenu JSON for 3 test snips. Local EXECUTE returns valid ActionResult for explain/fix/translate. Benchmark data recorded for M1 Air. User can switch between cloud and local providers in Settings without restarting.

---

## 5. Pillar C: Plugin Registry (Weeks 5-8)

### 5.1 The Problem

Today, installing a plugin means:

1. Find the plugin's GitHub repo (how? search? word of mouth?)
2. Clone it
3. Copy it to `~/.config/omni-glass/plugins/`
4. Restart Omni-Glass
5. Approve permissions

This is acceptable for developer early adopters. It's unacceptable for growth. We need:

- **Discovery:** browse available plugins from inside the app
- **One-click install:** click "Install" in the app, not `git clone` in terminal
- **Updates:** know when a plugin has a new version, update with one click
- **Trust signals:** star count, download count, verified author badges

### 5.2 Architecture

The registry is a simple static JSON index hosted on GitHub Pages. There is no backend server. Plugins are distributed as GitHub repos. The registry is a curated index that points to them.

**Why no backend:** A registry backend is expensive to build, host, and secure. A static JSON file on GitHub Pages is free, cacheable, and versioned. It scales to thousands of plugins. The tradeoff: no real-time download counts, no user reviews, no server-side search. These are Phase 4 features.

**Registry index format:**

```json
{
  "version": 1,
  "updated_at": "2026-03-15T00:00:00Z",
  "plugins": [
    {
      "id": "com.omni-glass.github-issues",
      "name": "GitHub Issues",
      "description": "Create GitHub issues from snipped screen content",
      "author": {
        "name": "Omni-Glass Team",
        "github": "goshtasb"
      },
      "version": "1.0.0",
      "repo": "https://github.com/goshtasb/omni-glass",
      "path": "plugins/com.omni-glass.github-issues",
      "permissions_summary": "network, environment",
      "risk_level": "medium",
      "min_omni_glass_version": "0.3.0",
      "downloads": 0,
      "verified": true
    }
  ]
}
```

**Registry repo:** `github.com/goshtasb/omni-glass-registry`

| Ticket | Description | Est. |
|--------|-------------|------|
| REG-01 | Create registry repo with JSON schema and initial index (our 1 plugin + template) | 1 day |
| REG-02 | In-app "Browse Plugins" UI in Settings → Plugins section | 3-4 days |
| REG-03 | One-click install: download plugin from GitHub, place in plugins dir, trigger approval | 2-3 days |
| REG-04 | Plugin update detection: compare installed version vs registry version | 1-2 days |
| REG-05 | One-click update: download new version, re-prompt if permissions changed | 1-2 days |
| REG-06 | Plugin submission process: PR template for adding plugins to the registry | 1 day |
| REG-07 | Plugin code signing: SHA-256 hash of plugin bundle verified at install time | 2-3 days |

**REG-02: Browse Plugins UI**

New section in Settings:

```
┌─────────────────────────────────────────────────────────┐
│  Browse Plugins                                          │
│  ─────────────────────────────────────────────────────   │
│  [Search plugins...]                                     │
│                                                          │
│  ┌─ GitHub Issues ────────────────── ✅ Installed ──┐   │
│  │  Create GitHub issues from snipped content         │   │
│  │  by @goshtasb · v1.0.0 · ⚠ Medium risk           │   │
│  │  [Update Available: v1.1.0]                        │   │
│  └────────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─ Slack Notifier ──────────────────────────────────┐   │
│  │  Send snipped content to Slack channels            │   │
│  │  by @community-dev · v0.2.0 · ⚠ Medium risk      │   │
│  │  [Install]                                         │   │
│  └────────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─ Notion Clipper ──────────────────────────────────┐   │
│  │  Save snipped content to Notion pages              │   │
│  │  by @notion-fan · v1.0.0 · ⚠ Medium risk          │   │
│  │  [Install]                                         │   │
│  └────────────────────────────────────────────────────┘   │
│                                                          │
│  Last updated: 5 minutes ago · 12 plugins available      │
└─────────────────────────────────────────────────────────┘
```

**REG-07: Plugin Signing**

When a plugin is submitted to the registry, we compute the SHA-256 hash of every file in the plugin directory and store it in the registry index. At install time, the app downloads the plugin, computes the hash of the received files, and compares. If they don't match, the install is rejected.

This prevents:
- MITM attacks that modify the plugin during download
- Registry poisoning (someone modifies the GitHub repo after submission)

It does NOT prevent:
- A malicious author publishing a malicious plugin (the permission prompt handles this)
- A compromised GitHub account pushing a malicious update (the hash check catches this — the registry hash won't match)

Full code signing with developer certificates is a Phase 4 feature.

**Gate:** A user can open Settings → Browse Plugins, see available plugins, click Install, approve permissions, and the plugin loads. Plugin updates are detected and applied with one click.

---

## 6. Pillar D: Rich Path (Weeks 7-12)

### 6.1 The Problem

Today, Omni-Glass only understands text. The OCR layer extracts characters, but has no understanding of what they represent visually. It doesn't know:

- That a red button says "Delete Account" and is dangerous
- That a form field is an email input
- That an error message is inside a modal dialog
- Where specific UI elements are positioned on screen

This limits actions to text-based operations. The user can't say "click the Submit button" or "fill in the email field" because Omni-Glass doesn't see buttons or fields — it only sees characters.

### 6.2 What Rich Path Enables

With UI element detection:

- **Click targeting:** "Click the red Delete button" — Omni-Glass locates the button coordinates and simulates a click
- **Form filling:** "Fill in my email" — Omni-Glass finds the email input field, clicks it, types the email
- **Context-aware actions:** The LLM knows this text is inside a dialog, inside a code editor, inside a browser tab
- **Accessibility bridge:** Screen content becomes structured, navigable, actionable — useful for users with visual impairments

### 6.3 The Technical Approach

**Two-stage pipeline:**

```
Stage 1: RT-DETR (Real-Time Detection Transformer)
    Input: Screenshot image
    Output: Bounding boxes + class labels for UI elements
    (button, input, text, image, icon, dropdown, checkbox, etc.)
    Latency target: < 100ms on M1 Air

Stage 2: Florence-2 (Microsoft Vision-Language Model)
    Input: Screenshot + bounding boxes from RT-DETR
    Output: Rich descriptions of each UI element
    ("Red button labeled 'Delete Account' in a confirmation dialog")
    Latency target: < 500ms on M1 Air
```

**Why two models instead of one:** RT-DETR is a fast, small object detector (~10M params) that's excellent at finding UI elements but can't describe them. Florence-2 is a larger vision-language model (~230M params) that can describe what it sees but is too slow to scan the entire screen. By using RT-DETR to find regions of interest and Florence-2 to describe them, we get both speed and understanding.

**Both models run locally.** No cloud dependency for the Rich Path. This is critical — we're adding computer vision that processes raw screenshots, and those screenshots must never leave the device.

| Ticket | Description | Est. |
|--------|-------------|------|
| RICH-01 | Integrate RT-DETR via ONNX Runtime: load model, run inference on screenshot crop | 3-4 days |
| RICH-02 | UI element extraction: parse RT-DETR output into structured UIElement types | 2 days |
| RICH-03 | Integrate Florence-2 via ONNX Runtime: describe detected UI elements | 3-4 days |
| RICH-04 | Merge Rich Path output into CLASSIFY prompt: LLM sees text + UI structure | 2-3 days |
| RICH-05 | Click simulation: translate bounding box coordinates to screen coordinates, simulate mouse click | 2-3 days |
| RICH-06 | Form filling: detect input fields, simulate keyboard input | 2-3 days |
| RICH-07 | Benchmark: Rich Path latency on M1 Air, M2 Pro, Intel i7 | 1 day |
| RICH-08 | Model management: download RT-DETR + Florence-2 ONNX models (~500MB total) | 1-2 days |

**New crate dependencies:**
- `ort` (ONNX Runtime Rust bindings) — mature, supports CoreML/CUDA/CPU backends
- `image` (already in tree via xcap) — image preprocessing

**RICH-04: Enhanced CLASSIFY Prompt**

Today's CLASSIFY prompt receives only OCR text. After Rich Path:

```
<extracted_text>
{ocr_text}
</extracted_text>

<ui_elements>
[
  { "type": "button", "label": "Delete Account", "color": "red", "bounds": [120, 340, 280, 380] },
  { "type": "input", "placeholder": "Enter email", "bounds": [120, 200, 480, 240] },
  { "type": "dialog", "title": "Confirm Deletion", "bounds": [80, 150, 520, 420] },
  { "type": "text", "content": "This action cannot be undone.", "bounds": [120, 280, 480, 300] }
]
</ui_elements>
```

The LLM now understands the visual structure, not just the text content. It can offer actions like "Click 'Delete Account'" or "Fill in the email field" because it knows where those elements are.

**RICH-05: Click Simulation**

```rust
// src-tauri/src/input/click.rs

pub fn simulate_click(screen_x: f64, screen_y: f64) -> Result<(), InputError> {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::event::*;
        let point = CGPoint::new(screen_x, screen_y);
        let click_down = CGEvent::new_mouse_event(
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap(),
            CGEventType::LeftMouseDown,
            point,
            CGMouseButton::Left,
        ).unwrap();
        let click_up = CGEvent::new_mouse_event(
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap(),
            CGEventType::LeftMouseUp,
            point,
            CGMouseButton::Left,
        ).unwrap();
        click_down.post(CGEventTapLocation::HID);
        click_up.post(CGEventTapLocation::HID);
    }
    
    #[cfg(target_os = "windows")]
    {
        // SendInput with MOUSEEVENTF_LEFTDOWN / LEFTUP
    }
    
    #[cfg(target_os = "linux")]
    {
        // XTest extension via x11 crate, or ydotool for Wayland
    }
    
    Ok(())
}
```

**Security implications:** Click simulation requires Accessibility permission on macOS. This is a significant trust escalation — the app can now control the mouse. The permission prompt must be clear: "Omni-Glass needs Accessibility access to click UI elements on your behalf."

**Gate:** RT-DETR detects UI elements in a screenshot with >80% accuracy. Florence-2 describes detected elements correctly. Click simulation moves the cursor and clicks at the correct coordinates. Total Rich Path pipeline (detect + describe + click) completes in under 1 second on M1 Air.

---

## 7. Phase 3 Deferred Items (Phase 4+)

These are explicitly not part of Phase 3. If anyone starts building them, stop.

| Item | Phase | Reason |
|------|-------|--------|
| Plugin marketplace with user accounts, reviews, ratings | Phase 4 | The static registry is sufficient for Phase 3's plugin count (<50) |
| Revenue / monetization | Phase 4+ | Build the ecosystem first. Monetize after PMF. |
| Mobile apps (iOS/Android) | Not planned | Desktop is the product. Mobile platforms restrict screen access. |
| Multi-turn conversation in action menu | Phase 4 | Single-shot actions cover 90%+ of use cases. Conversation adds complexity with marginal value. |
| Editor Bridge (VS Code / LSP integration) | Phase 4 | Requires MCP plugin that connects to VS Code's extension API. Complex trust model. Build as a community plugin, not first-party. |
| Browser extension | Phase 4 | DOM access would be more powerful than screenshot OCR for web content, but it's a separate product. |
| Voice input | Phase 5 | "Hey Omni-Glass, fix that error" — cool demo, low priority. |

---

## 8. Success Metrics

### 8.1 Phase 3 Completion Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| Windows pipeline works end-to-end | All 7 E2E tests pass | Test on real hardware |
| Linux pipeline works end-to-end | All 7 E2E tests pass | Test on Ubuntu 22.04 |
| Windows AppContainer blocks home dir | Sandbox escape test | Same 10 tests as macOS |
| Linux Bubblewrap blocks home dir | Sandbox escape test | Same 10 tests as macOS |
| Local LLM produces valid CLASSIFY JSON | 3/3 test snips | Qwen-2.5-3B on M1 Air |
| Local LLM produces valid EXECUTE results | 3/3 action types | Explain, Fix, Translate |
| Plugin registry has 10+ plugins | Count | Registry index |
| In-app plugin install works | Click Install → plugin loads | Manual test |
| RT-DETR detects UI elements | >80% accuracy on test set | 20 diverse screenshots |
| Click simulation works | Click lands on correct element | 5 manual tests per platform |
| All existing tests pass | 74+ tests, 0 regressions | CI |

### 8.2 Growth Targets (3 Months Post-Phase 3)

| Metric | Target |
|--------|--------|
| GitHub stars | 2,500+ |
| Weekly active users (estimated from download count) | 500+ |
| Community plugins in registry | 25+ |
| Supported platforms | macOS, Windows, Linux |
| LLM providers | Claude Haiku, Gemini Flash, Qwen-2.5 (local) |

---

## 9. Risk Register

| Risk | Severity | Mitigation | Status |
|------|----------|------------|--------|
| Windows build has many unforeseen issues | High | Dedicate full 2 weeks. Budget for surprises. | Open |
| Tesseract OCR quality is significantly worse than Apple Vision | Medium | Offer Apple Vision quality as a selling point for macOS. Tesseract is "good enough" for most use cases. | Open |
| Qwen-2.5-3B JSON output is unreliable | High | Extensive prompt tuning + constrained decoding (grammar-guided generation via llama.cpp grammars). Worst case: fall back to Qwen-2.5-7B which is more reliable but requires 16GB RAM. | Open |
| llama.cpp compilation is painful on Windows | Medium | Use pre-built binaries from llama.cpp releases. Only compile from source on macOS/Linux. | Open |
| RT-DETR accuracy is low on real-world UIs | High | Start with a UI-specific fine-tuned model (e.g., UIElement-RT-DETR). If accuracy is below 70%, defer Rich Path to Phase 4. | Open |
| Florence-2 is too slow for interactive use | Medium | Run Florence-2 only on the top-5 detected elements, not all. Use caching for repeated UI layouts. | Open |
| Bubblewrap not available on all Linux distros | Low | Check at startup, fall back to env filtering only with a warning. Document Bubblewrap as a recommended dependency. | Open |
| Plugin registry curation becomes a bottleneck | Medium | Accept all plugins that pass automated validation (manifest schema, no malware signatures). Manual review only for verified badge. | Open |
| Click simulation triggers macOS security prompts repeatedly | Medium | Guide users through Accessibility permission once during onboarding. Use `AXIsProcessTrusted()` to check before attempting. | Open |
| Model downloads (2-4GB) frustrate users on slow connections | Medium | Resumable downloads, show progress, offer multiple quantization options (Q4_K_M for speed, Q8_0 for quality). | Open |

---

## 10. Implementation Order

The four pillars are partially independent but have strategic dependencies:

```
Week 1-2:  Pillar A (Windows)     — get Windows working
Week 3-4:  Pillar A (Linux)       — get Linux working
Week 3-6:  Pillar B (Local LLM)   — starts in parallel with Linux
Week 5-8:  Pillar C (Registry)    — starts once cross-platform is stable
Week 7-12: Pillar D (Rich Path)   — starts once local models are working
```

**Why this order:**

1. Cross-platform first because every subsequent feature must work on all three platforms. If we build local LLM macOS-only and then port, we double the work.
2. Local LLM overlaps with Linux because they share a dependency (llama.cpp uses the same ONNX/CPU path on Linux).
3. Registry after cross-platform because plugin installs must work on all platforms.
4. Rich Path last because it's the most experimental, has the highest risk of failure, and depends on local model infrastructure from Pillar B.

**Phase 3 gate:** Cross-platform works (3 platforms), local LLM works (offline mode), plugin registry has 10+ plugins with in-app install, and Rich Path detects + clicks UI elements. When all four pillar gates pass, tag `v1.0.0` and launch publicly.

---

## 11. The v1.0.0 Question

Phase 3 is the last phase before v1.0.0. When Phase 3 ships, Omni-Glass will have:

- 3 platforms (macOS, Windows, Linux)
- 3 LLM providers (Claude, Gemini, Qwen local)
- 2 input modes (visual snip, text command)
- 7+ built-in actions
- A sandboxed plugin ecosystem with registry
- UI element detection and click simulation
- Open-source, MIT licensed

That's a v1.0. The product is complete enough that a developer can use it as their daily driver, extend it for their workflow, and trust it with their screen content.

Everything after v1.0 is growth: marketplace, editor integration, multi-turn, voice, monetization. The foundation is done.
