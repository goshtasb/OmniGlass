//! Full-screen capture using the `xcap` crate.
//!
//! This is the infrastructure layer — it talks to the OS.
//! If xcap fails on macOS 26.3, this file is the one we replace
//! with a ScreenCaptureKit FFI implementation.

use image::DynamicImage;
use xcap::Monitor;

/// Captures the primary monitor's screen as a `DynamicImage`.
///
/// Returns the full-screen screenshot including all pixels.
/// The caller is responsible for cropping to the user's selection.
pub fn capture_primary_monitor() -> Result<DynamicImage, CaptureError> {
    let monitors = Monitor::all().map_err(|e| CaptureError::MonitorEnumeration(e.to_string()))?;

    let primary = monitors
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or_else(|| {
            // Fallback: if no monitor reports as primary, use the first one
            let all = Monitor::all().ok()?;
            all.into_iter().next()
        })
        .ok_or(CaptureError::NoPrimaryMonitor)?;

    let image = primary
        .capture_image()
        .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

    Ok(DynamicImage::ImageRgba8(image))
}

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("Failed to enumerate monitors: {0}")]
    MonitorEnumeration(String),

    #[error("No primary monitor found")]
    NoPrimaryMonitor,

    #[error("Screen capture failed: {0}")]
    CaptureFailed(String),
}
