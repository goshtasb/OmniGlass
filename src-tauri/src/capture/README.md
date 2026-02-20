# capture/ — Screen Capture & Cropping

## Overview

The capture module handles full-screen screenshot acquisition and region cropping.
It captures the primary monitor via `xcap`, stores the screenshot in thread-safe
state for the overlay to display, and provides a pure function to crop a user-selected
rectangle to PNG bytes for the OCR pipeline.

## Public API

| Export | Type | Description |
|---|---|---|
| `capture_primary_monitor()` | Function | Captures the primary monitor, returns `DynamicImage` |
| `crop_to_png_bytes(image, x, y, w, h)` | Function | Crops a region and encodes to PNG bytes in memory |
| `CaptureState` | Struct | Thread-safe storage for screenshot + capture metadata |
| `CaptureInfo` | Struct | Screenshot path + click timestamp (serializable) |

## Internal Structure

| File | Lines | Responsibility |
|---|---|---|
| `mod.rs` | 38 | Public API re-exports, `CaptureState` and `CaptureInfo` definitions |
| `screenshot.rs` | 44 | `capture_primary_monitor()` — xcap monitor capture |
| `region.rs` | 99 | `crop_to_png_bytes()` — pure crop + PNG encode, with unit tests |

## Dependencies

| Crate | Used For |
|---|---|
| `xcap` | Native screen capture (macOS/Windows) |
| `image` | `DynamicImage`, `ImageFormat::Png`, crop operations |
| `std::sync::Mutex` | Thread-safe state storage |

## Used By

| Module | Imports | Purpose |
|---|---|---|
| `pipeline.rs` | `CaptureState`, `crop_to_png_bytes` | Crop region during snip pipeline |
| `commands.rs` | `CaptureState`, `CaptureInfo` | Serve capture info to overlay frontend |
| `lib.rs` | `CaptureState` | Register as Tauri managed state |

## Architecture Decisions

- **PNG over BMP**: The image crate's 32-bit RGBA BMP is unreadable by both CGImageSource
  (OCR) and WebKit (img tag). PNG handles RGBA natively with no conversion needed.
- **In-memory encoding**: Crop → PNG bytes happen entirely in memory. No temp files
  on the critical path between snip and OCR.
- **Retina scaling**: Coordinate mapping uses `image.width / window.innerWidth` rather
  than `devicePixelRatio` because macOS scaled displays report different ratios.
