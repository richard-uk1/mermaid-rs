//! Types and traits for our drawing API.
//!
//! You don't need to look here unless you want to implement your own drawing backend.
//!
//! Some of this code is copied from the `piet` crate.

mod color;
mod gradient;
pub mod svg;

pub use color::Color;
use kurbo::Shape;

/// A trait that you can implement to draw in your preferred way.
///
/// We provide some useful implementations (SVG, ..todo more).
///
/// The structure of this trait emphasizes simplicity over performance. If people want to re-render
/// these diagrams often (e.g. once per frame) we might need to re-think this. More specifically,
/// we don't retain anything between draws.
pub trait Drawer {
    fn draw_shape(
        &mut self,
        shape: impl Shape,
        stroke_style: Option<StrokeStyle>,
        fill_style: Option<FillStyle>,
    );
}

#[derive(Debug)]
pub struct StrokeStyle {
    pub color: Color,
    pub width: f64,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
}

impl StrokeStyle {
    pub fn new(color: Color, width: f64) -> Self {
        Self {
            color,
            width,
            line_cap: Default::default(),
            line_join: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct FillStyle {
    pub color: Color,
}

/// Options for angled joins in strokes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LineJoin {
    /// The outer edges of the two paths are extended until they intersect.
    ///
    /// Because the miter length can be extreme for small angles, you must supply
    /// a 'limit' at which we will fallback on [`LineJoin::Bevel`].
    ///
    /// This limit is the distance from the point where the inner edges of the
    /// stroke meet to the point where the outer edges meet.
    ///
    /// The default limit is `10.0`.
    ///
    /// This is also currently the default `LineJoin`; you should only need to
    /// construct it if you need to customize the `limit`.
    Miter {
        /// The maximum distance between the inner and outer stroke edges before beveling.
        limit: f64,
    },
    /// The two lines are joined by a circular arc.
    Round,
    /// The two segments are capped with [`LineCap::Butt`], and the notch is filled.
    Bevel,
}

impl LineJoin {
    /// The default maximum length for a [`LineJoin::Miter`].
    ///
    /// This is defined in the [Postscript Language Reference][PLRMv3] (pp 676).
    ///
    /// [PLRMv3]: https://www.adobe.com/content/dam/acom/en/devnet/actionscript/articles/PLRM.pdf
    pub const DEFAULT_MITER_LIMIT: f64 = 10.0;
}

/// Options for the cap of stroked lines.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LineCap {
    /// The stroke is squared off at the endpoint of the path.
    Butt,
    /// The stroke ends in a semicircular arc with a diameter equal to the line width.
    Round,
    /// The stroke projects past the end of the path, and is squared off.
    ///
    /// The stroke projects for a distance equal to half the width of the line.
    Square,
}

impl Default for LineJoin {
    fn default() -> Self {
        LineJoin::Miter {
            limit: LineJoin::DEFAULT_MITER_LIMIT,
        }
    }
}

impl Default for LineCap {
    fn default() -> Self {
        LineCap::Butt
    }
}
