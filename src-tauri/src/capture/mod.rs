//! Screen capture domain — public API.
//!
//! This module owns all screen capture functionality.
//! External code should only use the public functions exported here.

mod region;
mod screenshot;

pub use region::crop_to_png_bytes;
pub use screenshot::capture_primary_monitor;

use image::DynamicImage;
use std::sync::Mutex;

/// Thread-safe storage for the current full-screen capture.
/// Held between capture and crop so the user can draw a rectangle.
pub struct CaptureState {
    pub screenshot: Mutex<Option<DynamicImage>>,
}

impl CaptureState {
    pub fn new() -> Self {
        Self {
            screenshot: Mutex::new(None),
        }
    }
}
