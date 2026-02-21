# Week 2 Edge Cases & Bug Fixes

Eight bugs were discovered and fixed during the Week 2 vertical slice integration. Each represents a failure mode that will likely resurface in different forms during the Windows port and cross-platform work.

---

## 1. Overlay Race Condition

**Symptom:** Overlay window opened blank — screenshot image never appeared.

**Root cause:** The Rust backend emitted a `capture-ready` event with the screenshot path immediately after creating the overlay window. But the webview's JavaScript hadn't loaded yet, so the event listener wasn't registered when the event fired. The event was lost.

**Fix:** Replaced the event-based approach with a command-based pull model. The Rust backend stores the screenshot path + metadata in `CaptureState`. The overlay JS calls `invoke("get_capture_info")` on load. Since Tauri commands only execute after JS is ready, the race is eliminated.

**Files:** `lib.rs:188-198`, `capture/mod.rs:17`, `tray.rs:122`

**Windows port risk:** Same pattern applies to any data passed from Rust to a newly-created webview window. Always use commands (pull) instead of events (push) for initial data.

---

## 2. Image Format Encoding Bottleneck

**Symptom:** Debug builds took 2.6-3.2 seconds for the capture step alone. The `image` crate's `to_rgb8()` conversion was ~1,800ms in debug mode due to per-pixel copy without optimization.

**Root cause:** The initial implementation saved the full-screen screenshot as BMP (avoiding encoding cost), then loaded it into the overlay. But the OCR pipeline needed PNG, creating a format mismatch that required conversion.

**Fix:** Two changes: (1) Save as PNG directly since the overlay needs it anyway. (2) Add per-crate optimization in `Cargo.toml` for `image`, `png`, `flate2`, `miniz_oxide`, and `zune-jpeg` — setting `opt-level = 2` in dev profile. This dropped encoding from ~1,800ms to ~100ms in debug builds while keeping our own code debuggable.

**Files:** `tray.rs:67-70,96-102`, `Cargo.toml:31-47`

**Windows port risk:** The same `image` crate optimization applies. Windows may also need format-specific handling for `windows-capture` output format (likely BGRA, not RGBA).

---

## 3. Retina/HiDPI Coordinate Scaling

**Symptom:** Bounding box coordinates from the overlay didn't match the actual screenshot pixels. On a Retina display, the user drew a box at CSS coordinates (100, 200) but the crop targeted different pixels in the full-resolution screenshot.

**Root cause:** macOS "Looks like 1440x900" on a 2560x1600 panel gives `devicePixelRatio=2`, but `xcap` captures at the actual panel resolution (2560x1600), not `cssSize * dpr` (which would be 2880x1800). The DPR is not a reliable scaling factor for mapping CSS pixels to screenshot pixels.

**Fix:** Calculate scale factors from actual image dimensions: `scaleX = imgW / cssW`, `scaleY = imgH / cssH`. The overlay loads the screenshot image and uses its natural dimensions (not DPR) to compute the CSS-to-pixel mapping.

**Files:** `overlay.ts:31,38-43,137-147`

**Windows port risk:** Windows has its own HiDPI scaling (100%, 125%, 150%, 200%). Multi-monitor setups with mixed scaling are common. The same image-dimension-based approach should work, but test with mixed-DPI multi-monitor configurations.

---

## 4. Clipboard Access in Transparent Webview

**Symptom:** `navigator.clipboard.writeText()` failed silently in the action menu window. Text was not copied to clipboard.

**Root cause:** The Web Clipboard API requires a "secure context" and an "active document" with focus. Transparent, frameless Tauri webview windows (used for the overlay) don't reliably satisfy these requirements on macOS. The API call resolves without error but doesn't actually write to the system clipboard.

**Fix:** Bypassed the Web Clipboard API entirely. Added `arboard` crate (v3) as a Rust dependency, exposed a `copy_to_clipboard` Tauri command, and invoked it from JS. `arboard` uses native OS clipboard APIs directly (NSPasteboard on macOS) and works regardless of webview state.

**Files:** `Cargo.toml:29`, `lib.rs:215-225,275`

**Windows port risk:** `arboard` supports Windows natively via `SetClipboardData`. Should work out of the box, but test with Windows Terminal and PowerShell clipboard integration.

---

## 5. Transparent Action Menu Window

**Symptom:** Action menu appeared as floating text with no background — just white text on top of whatever was behind the window. Looked broken, not styled.

**Root cause:** The action menu window was created with `.transparent(true)` (copied from the overlay window setup). Combined with `background: transparent` in CSS, this meant the window had no visible surface — only text elements rendered. The dark background color (#1a1a2e) was invisible because the window itself was transparent.

**Fix:** Removed `.transparent(true)` from the action menu window builder. Changed the CSS body background from `transparent` to `#1a1a2e`. The overlay window still uses transparency (it needs to show the screenshot behind the selection box), but the action menu is a solid window.

**Files:** `lib.rs:135-148`, `action-menu.html:12`

**Windows port risk:** Windows transparent windows (`WS_EX_TRANSPARENT` or `WS_EX_LAYERED`) have different behavior than macOS. Test that the overlay transparency works on Windows. The action menu doesn't need transparency, so it's safe.

---

## 6. Missing API Key Handling

**Symptom:** App crashed or hung when no `ANTHROPIC_API_KEY` environment variable was set.

**Root cause:** The initial LLM integration code didn't guard against a missing API key. The HTTP request would fail with an auth error, but the error handling path didn't emit the fallback action menu event, leaving the action menu window stuck in skeleton state.

**Fix:** Check `std::env::var("ANTHROPIC_API_KEY")` at the top of both `classify_streaming()` and `classify()`. If missing or empty, log a warning and immediately return `ActionMenu::fallback()` with a complete event emission. The action menu always renders — either with real actions or the three fallback actions (Copy Text, Explain, Search Web).

**Files:** `classify.rs:45-53,309-315`

**Windows port risk:** Environment variable access works the same on Windows. However, the Week 3 settings panel will need to store the key in the OS keychain (Credential Manager on Windows), so this env-var approach is temporary.

---

## 7. In-Memory PNG Encoding (Eliminating Disk I/O)

**Symptom:** The pipeline wrote the cropped region to a temp file, then read it back for OCR — adding ~50ms of unnecessary disk I/O to the critical path.

**Root cause:** The OCR bench CLI (Week 1 spike) took a file path as input. The initial integration preserved this interface, writing the crop to disk then passing the path.

**Fix:** Added `recognize_text_from_bytes()` to the OCR module — a new entry point that accepts `Vec<u8>` (PNG bytes) directly. The crop is encoded to PNG in memory using `Cursor::new(&mut Vec)` as the write target. The PNG bytes pass directly to the Swift FFI bridge, which creates a `CGImage` from the data. Zero disk I/O in the crop→OCR path.

**Files:** `lib.rs:87-97`, `capture/region.rs:44-47`, `ocr/mod.rs:61-62`

**Windows port risk:** The same in-memory approach works for `Windows.Media.Ocr`, but the Windows OCR API may expect a different image format (BMP or `SoftwareBitmap`). Test the byte-level input path.

---

## 8. Vision Framework Cold Start

**Symptom:** First snip after app launch had noticeably higher OCR latency (~250ms) compared to subsequent snips (~90ms).

**Root cause:** Apple Vision Framework loads its text recognition models lazily on first use. The first `VNRecognizeTextRequest` triggers model loading, adding a one-time ~50-60ms penalty on top of the actual recognition time.

**Fix:** Added `warm_up()` function called during `app.setup()`. It runs a throwaway OCR recognition on a 1x1 pixel image, forcing Vision to load its models at startup instead of on the first real snip. Measured warm-up time: 53-57ms. After warm-up, the first-snip latency matches subsequent snips.

**Files:** `lib.rs:282-286`, `ocr/mod.rs:73-77,22`

**Windows port risk:** `Windows.Media.Ocr` likely has a similar cold-start penalty. Add an equivalent warm-up call during Windows app initialization. Profile to confirm.
