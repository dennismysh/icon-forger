//! Mode 2 — SVG import pipeline.
//!
//! Figma → export SVG → resvg rasterizes → all sizes.
//! Handles SVG parsing (usvg) and high-quality rasterization (resvg).

use resvg::tiny_skia::Pixmap;
use thiserror::Error;
use usvg::{Options, Tree};

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("failed to parse SVG: {0}")]
    Parse(String),
    #[error("failed to create pixmap for size {0}×{0}")]
    Pixmap(u32),
}

/// Parsed SVG ready for rasterization at any size.
pub struct SvgAsset {
    tree: Tree,
}

impl SvgAsset {
    /// Parse an SVG string (e.g. exported from Figma).
    pub fn from_svg_str(svg: &str) -> Result<Self, ImportError> {
        let opts = Options::default();
        let tree = Tree::from_str(svg, &opts).map_err(|e| ImportError::Parse(e.to_string()))?;
        Ok(Self { tree })
    }

    /// Parse raw SVG bytes.
    pub fn from_svg_bytes(data: &[u8]) -> Result<Self, ImportError> {
        let svg = std::str::from_utf8(data).map_err(|e| ImportError::Parse(e.to_string()))?;
        Self::from_svg_str(svg)
    }

    /// Rasterize the SVG to a square pixmap of the given `size`.
    pub fn render(&self, size: u32) -> Result<Pixmap, ImportError> {
        let mut pixmap = Pixmap::new(size, size).ok_or(ImportError::Pixmap(size))?;

        let svg_size = self.tree.size();
        let sx = size as f32 / svg_size.width();
        let sy = size as f32 / svg_size.height();
        let scale = sx.min(sy);

        // Centre the SVG in the square canvas
        let tx = (size as f32 - svg_size.width() * scale) / 2.0;
        let ty = (size as f32 - svg_size.height() * scale) / 2.0;

        let transform =
            resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(tx, ty);

        resvg::render(&self.tree, transform, &mut pixmap.as_mut());

        Ok(pixmap)
    }

    /// Batch-render to multiple sizes at once.
    pub fn render_sizes(&self, sizes: &[u32]) -> Result<Vec<(u32, Pixmap)>, ImportError> {
        sizes.iter().map(|&s| Ok((s, self.render(s)?))).collect()
    }
}
