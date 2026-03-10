//! Export layer — encode rendered pixmaps to all platform icon formats.
//!
//! Supported outputs:
//! - PNG  (all platforms)
//! - ICO  (Windows)
//! - ICNS (macOS)
//! - WebP (Android / web)

use image::{DynamicImage, ImageFormat, RgbaImage};
use std::io::Cursor;
use thiserror::Error;
use tiny_skia::Pixmap;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("image encoding failed: {0}")]
    Encode(String),
    #[error("ico encoding failed: {0}")]
    Ico(String),
    #[error("icns encoding failed: {0}")]
    Icns(String),
    #[error("unsupported icns size: {0}×{0}")]
    UnsupportedIcnsSize(u32),
    #[error("pixmap has no data")]
    EmptyPixmap,
}

/// Standard icon sizes for multi-resolution exports.
pub const STANDARD_SIZES: &[u32] = &[16, 32, 48, 64, 128, 256, 512, 1024];

/// Convert a `tiny_skia::Pixmap` to an `image::RgbaImage`.
fn pixmap_to_rgba(pixmap: &Pixmap) -> RgbaImage {
    let width = pixmap.width();
    let height = pixmap.height();
    // tiny-skia stores premultiplied RGBA; convert to straight RGBA
    let mut buf = Vec::with_capacity((width * height * 4) as usize);
    for pixel in pixmap.pixels() {
        let a = pixel.alpha();
        if a == 0 {
            buf.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let r = ((pixel.red() as u16 * 255) / a as u16) as u8;
            let g = ((pixel.green() as u16 * 255) / a as u16) as u8;
            let b = ((pixel.blue() as u16 * 255) / a as u16) as u8;
            buf.extend_from_slice(&[r, g, b, a]);
        }
    }
    RgbaImage::from_raw(width, height, buf).expect("buffer size matches dimensions")
}

/// Resize a pixmap to the target size using Lanczos3.
fn resize_pixmap(pixmap: &Pixmap, target: u32) -> RgbaImage {
    let img = DynamicImage::ImageRgba8(pixmap_to_rgba(pixmap));
    img.resize_exact(target, target, image::imageops::FilterType::Lanczos3)
        .to_rgba8()
}

// ── PNG ──────────────────────────────────────────────────────────

/// Encode a pixmap as PNG bytes.
pub fn to_png(pixmap: &Pixmap) -> Result<Vec<u8>, ExportError> {
    let img = pixmap_to_rgba(pixmap);
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| ExportError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

/// Encode a pixmap resized to `size` as PNG bytes.
pub fn to_png_sized(pixmap: &Pixmap, size: u32) -> Result<Vec<u8>, ExportError> {
    let img = resize_pixmap(pixmap, size);
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| ExportError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

// ── WebP ─────────────────────────────────────────────────────────

/// Encode a pixmap as WebP bytes.
pub fn to_webp(pixmap: &Pixmap) -> Result<Vec<u8>, ExportError> {
    let img = pixmap_to_rgba(pixmap);
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::WebP)
        .map_err(|e| ExportError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

/// Encode a pixmap resized to `size` as WebP bytes.
pub fn to_webp_sized(pixmap: &Pixmap, size: u32) -> Result<Vec<u8>, ExportError> {
    let img = resize_pixmap(pixmap, size);
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::WebP)
        .map_err(|e| ExportError::Encode(e.to_string()))?;
    Ok(buf.into_inner())
}

// ── ICO (Windows) ────────────────────────────────────────────────

/// Standard ICO sizes: 16, 32, 48, 256.
const ICO_SIZES: &[u32] = &[16, 32, 48, 256];

/// Bundle a pixmap into a multi-resolution .ico file.
pub fn to_ico(pixmap: &Pixmap) -> Result<Vec<u8>, ExportError> {
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for &size in ICO_SIZES {
        let rgba = resize_pixmap(pixmap, size);
        let icon_image = ico::IconImage::from_rgba_data(size, size, rgba.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image).map_err(|e| {
            ExportError::Ico(e.to_string())
        })?);
    }

    let mut buf = Vec::new();
    icon_dir
        .write(&mut buf)
        .map_err(|e| ExportError::Ico(e.to_string()))?;
    Ok(buf)
}

// ── ICNS (macOS) ─────────────────────────────────────────────────

/// Map pixel size → icns `IconType`.
fn icns_icon_type(size: u32) -> Result<icns::IconType, ExportError> {
    match size {
        16 => Ok(icns::IconType::RGBA32_16x16),
        32 => Ok(icns::IconType::RGBA32_32x32),
        64 => Ok(icns::IconType::RGBA32_64x64),
        128 => Ok(icns::IconType::RGBA32_128x128),
        256 => Ok(icns::IconType::RGBA32_256x256),
        512 => Ok(icns::IconType::RGBA32_512x512),
        1024 => Ok(icns::IconType::RGBA32_512x512_2x),
        other => Err(ExportError::UnsupportedIcnsSize(other)),
    }
}

/// Bundle a pixmap into a multi-resolution .icns file.
pub fn to_icns(pixmap: &Pixmap) -> Result<Vec<u8>, ExportError> {
    let mut icon_family = icns::IconFamily::new();
    let sizes = [16, 32, 64, 128, 256, 512, 1024];

    for &size in &sizes {
        let rgba = resize_pixmap(pixmap, size);
        let icns_img = icns::Image::from_data(
            icns::PixelFormat::RGBA,
            size,
            size,
            rgba.into_raw(),
        )
        .map_err(|e| ExportError::Icns(e.to_string()))?;

        let icon_type = icns_icon_type(size)?;
        icon_family
            .add_icon_with_type(&icns_img, icon_type)
            .map_err(|e| ExportError::Icns(e.to_string()))?;
    }

    let mut buf = Vec::new();
    icon_family
        .write(&mut buf)
        .map_err(|e| ExportError::Icns(e.to_string()))?;
    Ok(buf)
}

// ── Batch export ─────────────────────────────────────────────────

/// All supported output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Png,
    Ico,
    Icns,
    WebP,
}

/// Export a master pixmap to the requested format.
pub fn export(pixmap: &Pixmap, format: Format) -> Result<Vec<u8>, ExportError> {
    match format {
        Format::Png => to_png(pixmap),
        Format::Ico => to_ico(pixmap),
        Format::Icns => to_icns(pixmap),
        Format::WebP => to_webp(pixmap),
    }
}
