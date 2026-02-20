//! Windows OCR via Windows.Media.Ocr (WinRT).
//!
//! This module is only compiled on Windows. It uses the `windows` crate
//! to access the WinRT OcrEngine API, which ships with Windows 10+.
//!
//! NOTE: This code has been written on macOS and needs testing on Windows.
//! The WinRT API surface has been verified against Microsoft docs, but
//! the exact `windows` crate bindings may need minor adjustments.

use super::{OcrOutput, RecognitionLevel};
use std::time::Instant;

use windows::{
    Graphics::Imaging::BitmapDecoder,
    Media::Ocr::OcrEngine,
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
};

/// Run OCR on in-memory PNG bytes via Windows.Media.Ocr.
///
/// Uses WinRT OcrEngine with user profile languages (auto-detects
/// language from installed Windows language packs).
pub fn recognize_text_from_bytes(png_bytes: Vec<u8>, level: RecognitionLevel) -> OcrOutput {
    let start = Instant::now();

    match recognize_inner(&png_bytes) {
        Ok(text) => {
            let char_count = text.len() as i64;
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            let level_name = match level {
                RecognitionLevel::Fast => "fast",
                RecognitionLevel::Accurate => "accurate",
            };
            // Windows OCR doesn't expose per-character confidence scores.
            // Use 0.85 as a reasonable default when text is extracted.
            let confidence = if text.is_empty() { 0.0 } else { 0.85 };

            OcrOutput {
                text,
                char_count,
                latency_ms,
                confidence,
                recognition_level: level_name.to_string(),
            }
        }
        Err(e) => {
            log::error!("[OCR] Windows OCR failed: {}", e);
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            OcrOutput {
                text: String::new(),
                char_count: 0,
                latency_ms,
                confidence: 0.0,
                recognition_level: "fast".to_string(),
            }
        }
    }
}

/// Internal OCR implementation using WinRT APIs.
///
/// Flow: PNG bytes → InMemoryRandomAccessStream → BitmapDecoder
///       → SoftwareBitmap → OcrEngine::RecognizeAsync → text
fn recognize_inner(png_bytes: &[u8]) -> windows::core::Result<String> {
    // Step 1: Write PNG bytes into an in-memory stream
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(png_bytes)?;
    writer.StoreAsync()?.get()?;
    writer.FlushAsync()?.get()?;
    writer.DetachStream()?;

    // Step 2: Seek stream back to beginning for the decoder
    stream.Seek(0)?;

    // Step 3: Decode the PNG into a SoftwareBitmap
    let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
    let bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;

    // Step 4: Create OCR engine from user's installed language packs
    // TryCreateFromUserProfileLanguages auto-selects the best language
    let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;

    // Step 5: Run OCR recognition
    let result = engine.RecognizeAsync(&bitmap)?.get()?;

    // Step 6: Extract text — OcrResult.Text() returns all lines joined
    let text = result.Text()?.to_string();

    Ok(text)
}

/// Warm up Windows OCR engine by pre-loading the engine.
///
/// This loads the OCR DLLs and language data on first call,
/// avoiding a cold-start penalty on the first snip.
pub fn warm_up() {
    match OcrEngine::TryCreateFromUserProfileLanguages() {
        Ok(_) => log::info!("[OCR] Windows OCR engine warm-up complete"),
        Err(e) => log::warn!("[OCR] Windows OCR warm-up failed: {}", e),
    }
}
