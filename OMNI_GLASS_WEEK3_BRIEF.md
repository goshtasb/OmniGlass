# Week 3: Windows Parity + Provider Showdown

**Branch from:** `main` (merge `feat/week-2-vertical-slice` first)  
**Baseline:** Haiku streaming — skeleton < 1.1s, full actions < 3.3s, 3/3 classifications correct  
**Goal:** The product works on Windows, and we know which LLM provider is the default

---

## What We Learned in Week 2

The core loop works. The architecture is sound. But we also learned that **the LLM provider choice is the single biggest product decision we haven't made yet.** Haiku at 2.7-3.3s is acceptable. But Gemini 2.0 Flash may cut that in half at one-fifth the cost. If that's true, it changes the default provider, the onboarding flow, the cost documentation, and the competitive story.

We need that data before we build a provider abstraction layer — because the abstraction should be designed around the providers we actually ship with, not a theoretical set.

**Week 3 has two parallel tracks:**

- **Track A (Platform):** Windows screen capture + OCR. This is on the critical path regardless of provider choice.
- **Track B (LLM):** Gemini Flash integration + benchmark, then build the abstraction layer around the winners.

These tracks are independent. They can run simultaneously with zero coordination until the end-of-week integration.

---

## Track A: Windows Parity (Platform Team)

### A1: Windows Screen Capture (Days 1-3)

Port the screen capture spike to Windows. The macOS implementation uses `xcap` — start there since `xcap` claims cross-platform support.

**Test `xcap` on Windows first.** If it works, you're done in a day. If it doesn't, fall back to:

```
Primary: `xcap` crate (same as macOS — test this first)
Fallback: `windows-capture` crate
Last resort: `windows-rs` direct FFI to Windows.Graphics.Capture API
```

**The Windows overlay is simpler than macOS.** No private API needed. A standard Tauri webview window with these properties works:

```
- Fullscreen, borderless
- Always on top
- Transparent background
- No taskbar icon
- WS_EX_TRANSPARENT extended style for click-through (if needed)
```

No monthly permission re-prompts. No private API instability. Windows is the easier platform for this feature.

**SmartScreen warning:** The first time an unsigned app runs on Windows, SmartScreen shows a "Windows protected your PC" dialog. This is expected. Users click "More info" → "Run anyway." We'll handle code signing in Phase 4. For now, document it.

**Definition of done:**
- [ ] Tray icon appears in Windows system tray
- [ ] Click → screen freezes with dimmed overlay
- [ ] Drag rectangle → crop captured as pixel buffer
- [ ] Full capture flow < 750ms on reference Windows hardware

### A2: Windows OCR (Days 2-4)

Port the OCR bridge to Windows using `Windows.Media.Ocr`.

```
Access path: `windows-rs` crate → Windows.Media.Ocr.OcrEngine
The UWP OCR API is available on Windows 10 version 1809+

Key API:
  OcrEngine::TryCreateFromUserProfileLanguages()
  engine.RecognizeAsync(bitmap)
  result.Lines → iterate → concatenate text
```

**Important difference from Apple Vision:** Windows.Media.Ocr doesn't have `.fast` / `.accurate` modes. It has one mode. Benchmark it and see where it lands. If it's under 100ms for English text (likely — it's optimized for on-device speed), we're fine. If it's slow, Tesseract is the fallback.

**The OCR module already has a `RecognitionLevel` enum.** On Windows, both `.Fast` and `.Accurate` should route to the same Windows.Media.Ocr call. The enum exists for the macOS dual-mode strategy — on Windows it's a no-op.

**Test corpus:** Use the same 20 images from the macOS benchmark. Copy them to the Windows machine. Run the batch benchmark. The comparison data is:

| Platform | Mode | Median | P99 | Target |
|----------|------|--------|-----|--------|
| macOS | .fast | 26ms | 72ms | < 100ms |
| macOS | .accurate | 98ms | 459ms | < 200ms |
| Windows | (single mode) | ____ms | ____ms | < 300ms |

**Definition of done:**
- [ ] OCR extracts text from all 20 test corpus images on Windows
- [ ] Median latency < 300ms
- [ ] English, German, and Japanese text extracted accurately
- [ ] Results logged to CSV matching the macOS benchmark format

### A3: Windows End-to-End (Day 5)

Stitch capture + OCR + Haiku streaming on Windows. Run the same 3-snip benchmark:

| Snip | Skeleton (ms) | Full actions (ms) | Content type | Correct? |
|------|--------------|-------------------|-------------|----------|
| Code | | | | |
| Prose | | | | |
| Error | | | | |

Targets are the same as macOS: skeleton < 1.5s, full actions < 4s.

---

## Track B: Provider Showdown (LLM Team)

### B1: Gemini Flash Integration (Days 1-2)

Build a minimal Gemini provider alongside the existing Anthropic one. Not a full abstraction yet — just a second concrete implementation.

**File:** `src-tauri/src/llm/gemini.rs`

**API call:**

```
POST https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:streamGenerateContent?alt=sse&key={GEMINI_API_KEY}
Content-Type: application/json

{
  "contents": [
    {
      "role": "user",
      "parts": [
        {
          "text": "<snip_context>...</snip_context>\n\n<extracted_text>...</extracted_text>"
        }
      ]
    }
  ],
  "systemInstruction": {
    "parts": [
      {
        "text": "<the CLASSIFY system prompt — identical to what we send Claude>"
      }
    ]
  },
  "generationConfig": {
    "maxOutputTokens": 512,
    "temperature": 0.1,
    "responseMimeType": "application/json"
  }
}
```

**Key difference from Anthropic:** Gemini supports `responseMimeType: "application/json"` which forces the response to be valid JSON. This means no markdown code fences to strip, no fence-stripping regex needed. If this works reliably, it eliminates an entire failure mode.

**Streaming format:** Gemini uses SSE (Server-Sent Events), similar to Anthropic. Each event contains a `candidates[0].content.parts[0].text` field with the incremental text. Accumulate these chunks the same way you accumulate Anthropic's streaming chunks.

**API key:** `std::env::var("GEMINI_API_KEY")`. The engineer gets a free API key from Google AI Studio (ai.google.dev) — no billing setup needed for the free tier, which has generous rate limits for development.

**Use the exact same CLASSIFY system prompt.** Don't modify it for Gemini. The whole point is to compare providers on identical input.

### B2: The Benchmark (Days 2-3)

Run all 3 snips through both providers. Same content, same system prompt, same max_tokens. Record everything.

```
For each snip (code, prose, error) × each provider (Haiku, Gemini Flash):
  - TTFT (time to first token)
  - Time to skeleton visible
  - Time to full actions
  - Content type returned
  - Number of actions
  - Classification accuracy (is the top action correct?)
  - Estimated cost
  - JSON validity (did it need fence stripping? Did it parse cleanly?)
```

The comparison table we need:

| Metric | Haiku 4.5 | Gemini 2.0 Flash |
|--------|-----------|-----------------|
| TTFT (median) | ~500ms | ____ms |
| Skeleton visible (median) | ~0.9s | ____ms |
| Full actions (median) | ~3.0s | ____ms |
| Classification accuracy | 3/3 | ____/3 |
| JSON parse success | 3/3 (with fence strip) | ____/3 |
| Cost per snip | $0.003 | ____$ |
| JSON enforcement | No (manual parsing) | Yes (responseMimeType) |

**If Gemini Flash beats Haiku on speed AND matches on accuracy**, it becomes the default provider candidate. The free-tier API key with no billing setup is also a significant onboarding advantage — new users can start using Omni-Glass without entering a credit card.

**If Gemini Flash has worse classification quality**, Haiku stays default and Gemini is offered as a "fast/free" option.

**If both are close**, test one more: **OpenAI GPT-4o-mini.** This is a 10-minute integration if we're already building the abstraction layer. GPT-4o-mini is competitive on speed and cost. Having a third data point prevents us from making a two-option false dichotomy.

### B3: OpenAI Integration — Conditional (Day 3, only if Gemini and Haiku are close)

```
POST https://api.openai.com/v1/chat/completions
Headers:
  Authorization: Bearer {OPENAI_API_KEY}
  Content-Type: application/json

{
  "model": "gpt-4o-mini",
  "max_tokens": 512,
  "stream": true,
  "response_format": { "type": "json_object" },
  "messages": [
    { "role": "system", "content": "<CLASSIFY system prompt>" },
    { "role": "user", "content": "<snip_context>...</snip_context>\n\n<extracted_text>...</extracted_text>" }
  ]
}
```

**OpenAI's `response_format: json_object`** also enforces valid JSON output, like Gemini's `responseMimeType`. Same benefit: no fence stripping needed.

**Streaming format:** OpenAI uses SSE with `choices[0].delta.content` for incremental text. Well-documented, widely implemented.

### B4: Build the Provider Abstraction (Days 3-5)

After the benchmarks, you know which 2-3 providers survive. Now build the abstraction layer from the LLM Integration PRD Section 2.

**File structure:**

```
src-tauri/src/llm/
  mod.rs              ← public API, provider selection
  types.rs            ← ActionMenu, Action (already exists)
  prompts.rs          ← system prompt constants (already exists)
  classify.rs         ← shared classification logic, fallback handling
  provider.rs         ← LLMProvider trait definition
  anthropic.rs        ← Anthropic implementation (refactored from existing)
  gemini.rs           ← Gemini implementation (from B1)
  openai.rs           ← OpenAI implementation (if built in B3)
  streaming.rs        ← shared SSE parsing utilities
```

**The trait:**

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;

    /// Stream a CLASSIFY request. Yields partial text chunks.
    async fn classify_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
        max_tokens: u32,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>, LLMError>;

    /// Check if the provider is configured (API key present).
    fn is_configured(&self) -> bool;

    /// Estimate cost for a request.
    fn estimate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64;
}
```

All providers must:
1. Accept the same system prompt and user message (no provider-specific prompt modifications)
2. Stream the response using the same chunk-accumulation pattern
3. Return valid ActionMenu JSON (with or without native JSON enforcement)
4. Expose the same timing metrics (TTFT, total latency, token counts)

**Provider selection logic:**

```rust
// In order of priority:
// 1. User's explicit choice in settings (if set)
// 2. First configured provider in this priority order:
//    a. The benchmark winner (Gemini Flash or Haiku — TBD after B2)
//    b. Second-place provider
//    c. Third-place provider
// 3. Fallback menu (no LLM call)

fn select_provider(config: &Config) -> Option<Box<dyn LLMProvider>> {
    if let Some(explicit) = &config.preferred_provider {
        return create_provider(explicit);
    }
    for provider_id in &DEFAULT_PRIORITY_ORDER {
        if let Some(p) = create_provider(provider_id) {
            if p.is_configured() {
                return Some(p);
            }
        }
    }
    None
}
```

---

## Track C: Settings Panel (Frontend, Days 3-5)

The settings panel is a Tauri webview window accessible from the tray icon's right-click context menu.

**Sections:**

### AI Provider

```
┌─────────────────────────────────────────────────────┐
│  AI Provider                                        │
│  ─────────────────────────────────────────────────  │
│                                                     │
│  Provider: [▼ Gemini Flash (Recommended)        ]   │
│                                                     │
│  ┌─ Gemini Flash ──────────────────────────────┐   │
│  │  API Key: [••••••••••••••••••] [Test] ✓      │   │
│  │  Speed: ★★★★★  Quality: ★★★★  Cost: Free*   │   │
│  │  * Free tier: 15 requests/minute             │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
│  ┌─ Claude Haiku ──────────────────────────────┐   │
│  │  API Key: [                    ] [Test]       │   │
│  │  Speed: ★★★★  Quality: ★★★★★  Cost: ~$0.002 │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
│  ┌─ Claude Sonnet (Quality Mode) ──────────────┐   │
│  │  Uses same API key as Haiku                   │   │
│  │  Speed: ★★  Quality: ★★★★★  Cost: ~$0.011   │   │
│  │  ⚠ Slower: 7-8s for full actions             │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
│  ┌─ GPT-4o-mini ───────────────────────────────┐   │
│  │  API Key: [                    ] [Test]       │   │
│  │  Speed: ★★★★  Quality: ★★★★  Cost: ~$0.002  │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
└─────────────────────────────────────────────────────┘

Note: Provider names and "Recommended" badge are placeholders.
Update after benchmark results from B2.
```

**The "Test" button** sends a minimal CLASSIFY request with hardcoded text ("Hello world") and checks for a valid JSON response. Shows ✓ (green) on success, ✗ (red) with error message on failure.

**API key storage:** Use the `keyring` crate (or `tauri-plugin-stronghold`) to store keys in the OS credential store:
- macOS: Keychain
- Windows: Credential Manager
- Linux: Secret Service (gnome-keyring / KWallet)

**Never store API keys in:**
- Config files on disk
- Environment variables (development only)
- LocalStorage / browser storage
- Anywhere unencrypted

**Reading keys at startup:**

```rust
// Pseudo-code for key retrieval
fn get_api_key(provider: &str) -> Option<String> {
    // 1. Check OS keychain first (production path)
    if let Ok(key) = keyring::Entry::new("omni-glass", provider)?.get_password() {
        return Some(key);
    }
    // 2. Fall back to environment variable (development path)
    std::env::var(format!("{}_API_KEY", provider.to_uppercase())).ok()
}
```

This means the existing `ANTHROPIC_API_KEY` environment variable workflow still works for development, but production users store keys securely via the settings panel.

### Recognition Mode

```
┌─────────────────────────────────────────────────────┐
│  Recognition                                        │
│  ─────────────────────────────────────────────────  │
│                                                     │
│  OCR Mode: [◉ Fast (default)  ○ Accurate]           │
│  Fast: ~26ms, best for action classification        │
│  Accurate: ~98ms, full text fidelity for exports    │
│                                                     │
│  Note: "Accurate" mode is used automatically for    │
│  text-sensitive actions (Translate, Export CSV)      │
│  regardless of this setting.                        │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### About

```
┌─────────────────────────────────────────────────────┐
│  About                                              │
│  ─────────────────────────────────────────────────  │
│                                                     │
│  Omni-Glass v0.1.0-alpha                            │
│  The Open-Source Raycast for Screen Actions          │
│                                                     │
│  GitHub: github.com/goshtasb/omni-glass             │
│  License: MIT                                       │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**No visual polish.** System fonts, basic layout, functional only. The settings panel ships in Week 3 as a working tool, not a design showcase.

---

## End-of-Week-3 Deliverables

### Must Have

1. **Windows demo:** Snip → OCR → Haiku streaming → action menu. Screen recording.
2. **Windows benchmark:** 3 snips with full timing breakdown. Targets: skeleton < 1.5s, full < 4s.
3. **Provider benchmark table:** Haiku vs Gemini Flash (vs GPT-4o-mini if tested). Complete comparison with TTFT, skeleton, full, accuracy, cost.
4. **Provider abstraction:** At least 2 providers hot-swappable via config.
5. **Settings panel:** Provider selection, API key entry with Test button, keys stored in OS keychain.

### Nice to Have

6. **Source app detection on macOS** — get the name of the frontmost application when the tray icon is clicked, pass it as `source_app` in the CLASSIFY request instead of "unknown." On macOS this is `NSWorkspace.shared.frontmostApplication?.localizedName`. Improves classification accuracy ("Terminal" → more likely to be an error; "Chrome" → more likely to be a table or prose).
7. **Window title detection** — harder, platform-specific, skip if time is short.

---

## What NOT to Build This Week

| Don't | Why |
|-------|-----|
| Action execution (buttons do things) | Week 4 |
| Local LLM / llama.cpp / Ollama | Phase 2 |
| MCP plugins | Phase 2 |
| Sensitive data redaction | Week 4 |
| Command blocklist | Week 4 |
| Onboarding wizard | After Week 4 |
| Linux support | Phase 3 |
| Any provider beyond what benchmarks justify | Don't build what you won't ship |

---

## Decision Points

### End of Day 3: Provider Default Decision

After the Gemini Flash benchmark, we make the call:

| Scenario | Decision |
|----------|----------|
| Gemini Flash skeleton < 0.8s AND accuracy 3/3 | Gemini Flash is default. Haiku is "quality" option. |
| Gemini Flash skeleton < 0.8s BUT accuracy < 3/3 | Run 10 more test snips. If accuracy > 80%, Gemini default. Otherwise Haiku default. |
| Gemini Flash skeleton > 1.5s | Haiku stays default. Gemini offered as "free tier" option. |
| Gemini Flash comparable to Haiku (~1s skeleton) | Test GPT-4o-mini as tiebreaker. Best of three becomes default. |

This decision directly affects the settings panel UI (which provider gets the "Recommended" badge) and the onboarding flow (which key to ask for first). The frontend engineer needs this answer by Day 3 to build the settings panel correctly.

### End of Week 3: Windows Go/No-Go

| Result | Decision |
|--------|----------|
| Windows pipeline meets all targets | Proceed to Week 4 on both platforms |
| Windows capture works but OCR is slow | Evaluate Tesseract as fallback OCR, proceed with caveat |
| Windows capture fundamentally broken with xcap | Escalate. Evaluate `windows-capture` crate. May need extra week. |

---

## Communication Cadence

**Day 2 (async):** Gemini Flash first benchmark result. Post the raw numbers. No analysis needed — the numbers speak.

**Day 3 (15-min sync):** Provider default decision. Bring the comparison table. We make the call live and the frontend engineer starts the settings panel with the correct default.

**Day 5 (30-min sync — End of Week 3):**
1. Windows demo (screen recording)
2. Windows benchmark numbers
3. Provider abstraction demo (switch providers in settings, show different response times)
4. Settings panel walkthrough
5. Updated provider recommendation for PRD

---

## What This Week Proves

Week 2 proved the loop works on macOS with one provider. Week 3 proves it's a real cross-platform product with a real provider strategy. After this week, the "demo" becomes a "prototype" — something you could put in front of a developer and say "install this, paste your API key, try snipping something." That's the bar.
