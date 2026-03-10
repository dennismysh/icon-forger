//! # Icon Forger
//!
//! Parametric icon generator and SVG import pipeline that exports
//! to all platform icon formats (PNG, ICO, ICNS, WebP).
//!
//! Two modes of operation:
//! - **Code-first** (`codegen`): draw parametric shapes with tiny-skia
//! - **Import pipeline** (`import`): Figma → SVG → resvg rasterization

pub mod codegen;
pub mod export;
pub mod import;
pub mod shapes;

use wasm_bindgen::prelude::*;

use crate::export::Format;
use crate::shapes::IconDef;

// ── WASM API ─────────────────────────────────────────────────────

/// Render a parametric icon from JSON definition → PNG bytes.
#[wasm_bindgen]
pub fn render_icon_png(json_def: &str, size: u32) -> Result<Vec<u8>, JsValue> {
    let def: IconDef =
        serde_json::from_str(json_def).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let pixmap = def
        .render(size)
        .ok_or_else(|| JsValue::from_str("render failed"))?;
    export::to_png(&pixmap).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Render a parametric icon → export to chosen format.
#[wasm_bindgen]
pub fn render_icon(json_def: &str, size: u32, format: &str) -> Result<Vec<u8>, JsValue> {
    let def: IconDef =
        serde_json::from_str(json_def).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let pixmap = def
        .render(size)
        .ok_or_else(|| JsValue::from_str("render failed"))?;
    let fmt = parse_format(format)?;
    export::export(&pixmap, fmt).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Import SVG string → rasterize at given size → PNG bytes.
#[wasm_bindgen]
pub fn svg_to_png(svg_str: &str, size: u32) -> Result<Vec<u8>, JsValue> {
    let asset =
        import::SvgAsset::from_svg_str(svg_str).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let pixmap = asset
        .render(size)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    export::to_png(&tiny_skia_pixmap_from_resvg(pixmap))
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Import SVG string → export to chosen format.
#[wasm_bindgen]
pub fn svg_to_format(svg_str: &str, size: u32, format: &str) -> Result<Vec<u8>, JsValue> {
    let asset =
        import::SvgAsset::from_svg_str(svg_str).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let pixmap = asset
        .render(size)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let fmt = parse_format(format)?;
    export::export(&tiny_skia_pixmap_from_resvg(pixmap), fmt)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// List available export formats.
#[wasm_bindgen]
pub fn available_formats() -> Vec<String> {
    vec![
        "png".into(),
        "ico".into(),
        "icns".into(),
        "webp".into(),
    ]
}

/// List standard icon sizes.
#[wasm_bindgen]
pub fn standard_sizes() -> Vec<u32> {
    export::STANDARD_SIZES.to_vec()
}

// ── Helpers ──────────────────────────────────────────────────────

fn parse_format(s: &str) -> Result<Format, JsValue> {
    match s.to_ascii_lowercase().as_str() {
        "png" => Ok(Format::Png),
        "ico" => Ok(Format::Ico),
        "icns" => Ok(Format::Icns),
        "webp" => Ok(Format::WebP),
        other => Err(JsValue::from_str(&format!("unknown format: {other}"))),
    }
}

/// Convert resvg's tiny_skia::Pixmap to the tiny_skia::Pixmap used by the rest of the crate.
/// (resvg re-exports its own tiny-skia; the pixel data is identical.)
fn tiny_skia_pixmap_from_resvg(resvg_pm: resvg::tiny_skia::Pixmap) -> tiny_skia::Pixmap {
    tiny_skia::Pixmap::decode_png(&resvg_pm.encode_png().expect("encode resvg pixmap"))
        .expect("decode into tiny_skia pixmap")
}
