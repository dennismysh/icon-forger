//! Mode 1 — Code-first parametric icon generation.
//!
//! Build icons programmatically by composing `IconDef` layers,
//! then render them at any resolution via tiny-skia.

use crate::shapes::{IconDef, IconShape, Rgba};
use tiny_skia::Pixmap;

/// Convenience builder for creating icons in code.
pub struct IconBuilder {
    def: IconDef,
}

impl IconBuilder {
    pub fn new() -> Self {
        Self {
            def: IconDef {
                background: None,
                layers: Vec::new(),
            },
        }
    }

    pub fn background(mut self, color: Rgba) -> Self {
        self.def.background = Some(color);
        self
    }

    pub fn layer(mut self, shape: IconShape) -> Self {
        self.def.layers.push(shape);
        self
    }

    pub fn build(self) -> IconDef {
        self.def
    }

    /// Build and immediately render at the given size.
    pub fn render(self, size: u32) -> Option<Pixmap> {
        self.def.render(size)
    }
}

impl Default for IconBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick preset: app-store style rounded-rect icon with a centred circle.
pub fn preset_app_icon(bg: Rgba, fg: Rgba, corner_radius: f64) -> IconDef {
    IconBuilder::new()
        .background(bg)
        .layer(IconShape::RoundedRect {
            corner_radius,
            color: bg,
        })
        .layer(IconShape::Circle {
            radius: 0.5,
            color: fg,
        })
        .build()
}

/// Quick preset: simple polygon badge.
pub fn preset_polygon_badge(sides: u32, bg: Rgba, fg: Rgba) -> IconDef {
    IconBuilder::new()
        .background(bg)
        .layer(IconShape::Polygon { sides, color: fg })
        .build()
}
