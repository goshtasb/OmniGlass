//! Pure region cropping logic — functional core.
//!
//! This module has zero infrastructure dependencies.
//! It takes pixel data in, returns pixel data out.

use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

/// Crops a `DynamicImage` to the specified rectangle and returns PNG bytes.
///
/// This is a pure function with no side effects.
///
/// # Arguments
/// * `image` - The full screenshot
/// * `x` - Left edge of the crop rectangle
/// * `y` - Top edge of the crop rectangle
/// * `width` - Width of the crop rectangle
/// * `height` - Height of the crop rectangle
///
/// # Returns
/// PNG-encoded bytes of the cropped region
pub fn crop_to_png_bytes(
    image: &DynamicImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, CropError> {
    if width == 0 || height == 0 {
        return Err(CropError::ZeroDimension);
    }

    let (img_width, img_height) = (image.width(), image.height());

    if x + width > img_width || y + height > img_height {
        return Err(CropError::OutOfBounds {
            requested: (x, y, width, height),
            image_size: (img_width, img_height),
        });
    }

    let cropped = image.crop_imm(x, y, width, height);

    let mut png_bytes: Vec<u8> = Vec::new();
    cropped
        .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .map_err(|e| CropError::EncodingFailed(e.to_string()))?;

    Ok(png_bytes)
}

#[derive(Debug, thiserror::Error)]
pub enum CropError {
    #[error("Crop rectangle has zero width or height")]
    ZeroDimension,

    #[error(
        "Crop rectangle ({},{},{},{}) exceeds image bounds ({}x{})",
        requested.0, requested.1, requested.2, requested.3,
        image_size.0, image_size.1
    )]
    OutOfBounds {
        requested: (u32, u32, u32, u32),
        image_size: (u32, u32),
    },

    #[error("PNG encoding failed: {0}")]
    EncodingFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    #[test]
    fn crop_valid_region() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        let result = crop_to_png_bytes(&img, 10, 10, 50, 50);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // PNG magic bytes
        assert_eq!(&bytes[..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn crop_zero_dimension_fails() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        let result = crop_to_png_bytes(&img, 0, 0, 0, 50);
        assert!(matches!(result, Err(CropError::ZeroDimension)));
    }

    #[test]
    fn crop_out_of_bounds_fails() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        let result = crop_to_png_bytes(&img, 80, 80, 30, 30);
        assert!(matches!(result, Err(CropError::OutOfBounds { .. })));
    }
}
