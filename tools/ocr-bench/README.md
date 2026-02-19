# OCR Benchmark Tool

Benchmarks Apple Vision Framework text recognition for Omni-Glass.

## Architecture

This is a **spike/benchmark tool**, not the production OCR integration.

- `swift-src/ocr.swift` — Standalone Swift CLI using `VNRecognizeTextRequest`
- `src/main.rs` — Rust CLI that auto-compiles the Swift helper and drives benchmarks

For production (Week 2), the OCR bridge will use `swift-bridge` for direct
Rust↔Swift FFI within the Tauri app, eliminating subprocess overhead.

## Prerequisites

- macOS 13+ (Apple Vision Framework)
- Xcode Command Line Tools (`xcode-select --install`)
- Rust toolchain

## Usage

```bash
# Single image
cargo run -- path/to/image.png

# Single image, fast recognition mode
cargo run -- path/to/image.png --fast

# Batch mode — all PNGs in a directory → CSV output
cargo run -- --batch ../../test-corpus/

# Batch with fast mode
cargo run -- --batch ../../test-corpus/ --fast
```

## Output

Single image mode prints JSON:
```json
{
  "text": "extracted text...",
  "charCount": 347,
  "latencyMs": 127.3,
  "confidence": 0.95,
  "recognitionLevel": "accurate"
}
```

Batch mode prints CSV to stdout and summary statistics to stderr.

## Performance Targets

| Metric | Target |
|---|---|
| Median latency (accurate) | < 300ms |
| P99 latency (accurate) | < 500ms |
| Median latency (fast) | < 100ms |
