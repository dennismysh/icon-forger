//! Parametric shape primitives built on kurbo geometry.
//!
//! Each shape is defined by a small parameter set and can be
//! rasterized to a tiny-skia `Pixmap` at any requested size.

use kurbo::{BezPath, Circle, Point, Rect, RoundedRect, Shape};
use serde::{Deserialize, Serialize};
use tiny_skia::{
    FillRule, Paint, PathBuilder, Pixmap, Transform,
};

/// RGBA colour in 0-255 range.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    fn to_paint(&self) -> Paint<'static> {
        let mut p = Paint::default();
        p.set_color_rgba8(self.r, self.g, self.b, self.a);
        p.anti_alias = true;
        p
    }
}

/// A shape that can be drawn onto a pixmap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IconShape {
    /// Filled circle centred in the canvas. `radius` is 0.0–1.0 relative.
    Circle { radius: f64, color: Rgba },
    /// Rounded rectangle. `corner_radius` is 0.0–1.0 relative.
    RoundedRect { corner_radius: f64, color: Rgba },
    /// Regular polygon (triangle, hexagon, …).
    Polygon { sides: u32, color: Rgba },
    /// Ring / donut. `inner` and `outer` are 0.0–1.0 relative radii.
    Ring { inner: f64, outer: f64, color: Rgba },
}

/// A full icon definition: background + ordered layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconDef {
    pub background: Option<Rgba>,
    pub layers: Vec<IconShape>,
}

impl IconDef {
    /// Render the icon to a `Pixmap` of the given `size` (square).
    pub fn render(&self, size: u32) -> Option<Pixmap> {
        let mut pixmap = Pixmap::new(size, size)?;

        if let Some(bg) = &self.background {
            let paint = bg.to_paint();
            let rect = tiny_skia::Rect::from_xywh(0.0, 0.0, size as f32, size as f32)?;
            let path = PathBuilder::from_rect(rect);
            pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
        }

        let sz = size as f64;
        let centre = Point::new(sz / 2.0, sz / 2.0);

        for layer in &self.layers {
            match layer {
                IconShape::Circle { radius, color } => {
                    let r = radius * sz / 2.0;
                    let circle = Circle::new(centre, r);
                    draw_kurbo_shape(&circle, color, &mut pixmap);
                }
                IconShape::RoundedRect { corner_radius, color } => {
                    let inset = sz * 0.05;
                    let rect = Rect::new(inset, inset, sz - inset, sz - inset);
                    let cr = corner_radius * sz / 2.0;
                    let rr = RoundedRect::from_rect(rect, cr);
                    draw_kurbo_shape(&rr, color, &mut pixmap);
                }
                IconShape::Polygon { sides, color } => {
                    let path = regular_polygon(*sides, centre, sz * 0.45);
                    draw_kurbo_shape(&path, color, &mut pixmap);
                }
                IconShape::Ring { inner, outer, color } => {
                    let outer_c = Circle::new(centre, outer * sz / 2.0);
                    let inner_c = Circle::new(centre, inner * sz / 2.0);
                    // Outer filled, inner subtracted via EvenOdd
                    let mut bez = BezPath::new();
                    for el in outer_c.path_elements(0.1) {
                        bez.push(el);
                    }
                    for el in inner_c.path_elements(0.1) {
                        bez.push(el);
                    }
                    draw_kurbo_path(&bez, color, FillRule::EvenOdd, &mut pixmap);
                }
            }
        }

        Some(pixmap)
    }
}

/// Build a regular polygon as a `BezPath`.
fn regular_polygon(sides: u32, centre: Point, radius: f64) -> BezPath {
    let mut path = BezPath::new();
    let n = sides.max(3) as usize;
    for i in 0..n {
        let angle = std::f64::consts::TAU * (i as f64) / (n as f64) - std::f64::consts::FRAC_PI_2;
        let pt = Point::new(
            centre.x + radius * angle.cos(),
            centre.y + radius * angle.sin(),
        );
        if i == 0 {
            path.move_to(pt);
        } else {
            path.line_to(pt);
        }
    }
    path.close_path();
    path
}

/// Convert kurbo elements into a tiny-skia path and fill.
fn draw_kurbo_shape(shape: &impl Shape, color: &Rgba, pixmap: &mut Pixmap) {
    let bez: BezPath = shape.to_path(0.1);
    draw_kurbo_path(&bez, color, FillRule::Winding, pixmap);
}

fn draw_kurbo_path(bez: &BezPath, color: &Rgba, rule: FillRule, pixmap: &mut Pixmap) {
    let mut pb = PathBuilder::new();
    for el in bez.elements() {
        match el {
            kurbo::PathEl::MoveTo(p) => pb.move_to(p.x as f32, p.y as f32),
            kurbo::PathEl::LineTo(p) => pb.line_to(p.x as f32, p.y as f32),
            kurbo::PathEl::QuadTo(c, p) => {
                pb.quad_to(c.x as f32, c.y as f32, p.x as f32, p.y as f32)
            }
            kurbo::PathEl::CurveTo(c1, c2, p) => pb.cubic_to(
                c1.x as f32, c1.y as f32,
                c2.x as f32, c2.y as f32,
                p.x as f32, p.y as f32,
            ),
            kurbo::PathEl::ClosePath => pb.close(),
        }
    }
    if let Some(path) = pb.finish() {
        let paint = color.to_paint();
        pixmap.fill_path(&path, &paint, rule, Transform::identity(), None);
    }
}
