# ocr/ — Platform-Abstracted Text Recognition

## Overview

The OCR module extracts text from cropped screenshot regions using platform-native
recognition engines. On macOS it uses Apple Vision Framework via swift-bridge FFI;
on Windows it uses WinRT OCR. It also provides content heuristics (table detection,
code detection) that inform the LLM classify step. Two recognition levels are
supported: `.fast` (~30ms, used for classify) and `.accurate` (~370ms, used for
code-fix actions where every bracket matters).

## Public API

| Export | Type | Description |
|---|---|---|
| `recognize_text_from_bytes(png, level)` | Function | OCR from in-memory PNG bytes, returns `OcrOutput` |
| `recognize_text(path, level)` | Function | OCR from file path (macOS only, legacy) |
| `warm_up()` | Function | Pre-initialize the Vision Framework to avoid cold-start penalty |
| `RecognitionLevel` | Enum | `Accurate` (0) or `Fast` (1) |
| `OcrOutput` | Struct | `text`, `char_count`, `latency_ms`, `confidence`, `recognition_level` |
| `heuristics::detect_table_structure(text)` | Function | Returns `true` if text contains tabular data patterns |
| `heuristics::detect_code_structure(text)` | Function | Returns `true` if text contains code-like patterns |

## Internal Structure

| File | Lines | Responsibility |
|---|---|---|
| `mod.rs` | 77 | Public API, platform dispatch, `OcrOutput` / `RecognitionLevel` types |
| `apple_vision.rs` | 53 | macOS: Apple Vision Framework FFI via swift-bridge |
| `windows_ocr.rs` | 102 | Windows: WinRT OCR implementation |
| `heuristics.rs` | 118 | Content structure detection (tables, code) — platform-independent |

## Dependencies

| Crate / Module | Used For |
|---|---|
| `swift-bridge` | FFI to Swift for Apple Vision Framework (macOS) |
| `image` | PNG decoding for byte-based OCR |
| `std::time::Instant` | Latency measurement |

## Used By

| Module | Imports | Purpose |
|---|---|---|
| `pipeline.rs` | `recognize_text_from_bytes`, `RecognitionLevel`, `heuristics` | OCR in snip pipeline + re-OCR for code fixes |
| `lib.rs` | `warm_up()` | Vision Framework warm-up at app startup |

## Architecture Decisions

- **Two recognition levels**: `.fast` for classify (speed matters, OCR noise is
  tolerable), `.accurate` for code-fix execute (precision matters, every character
  counts). The pipeline stores crop PNG bytes so execute can re-OCR without
  re-capturing.
- **Platform dispatch via cfg**: `mod.rs` uses `#[cfg(target_os)]` to select the
  correct backend. Both backends expose the same `recognize_text` signature.
- **Bytes-first API**: `recognize_text_from_bytes` is the primary entry point.
  No temp files on the OCR path — PNG bytes flow directly from crop to recognition.
- **Warm-up**: Vision Framework has a ~500ms cold-start penalty. `warm_up()` is
  called during app setup so the first snip doesn't pay this cost.
