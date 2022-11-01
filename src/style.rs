//! Some shared code to support styling charts.
use piet::Color;

/// How to style drawing the outline of a shape.
#[derive(Debug, Clone)]
pub struct StrokeStyle {
    /// The width of the outline.
    pub width: f64,
    /// The color of the outline.
    pub color: Color,
    // todo dashing/linecap/etc
}

impl StrokeStyle {
    /// Helper to create a stroke style.
    pub fn new(width: f64, color: Color) -> Self {
        Self { width, color }
    }
}

/// How to style some text.
#[derive(Debug, Clone)]
pub struct TextStyle {
    /// The text color.
    pub color: Color,
    /// The font height to use.
    pub font_size: f64,
    /// Whether text should be bold.
    pub bold: bool,
}

impl TextStyle {
    pub(crate) fn default() -> Self {
        Self {
            color: Color::BLACK,
            font_size: 16.,
            bold: false,
        }
    }

    pub(crate) fn default_dark() -> Self {
        Self {
            color: Color::WHITE,
            font_size: 16.,
            bold: false,
        }
    }

    /// Set whether the text font height.
    pub fn with_font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// Set whether the text should be bold.
    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }
}

/// A tpye that knows how to select colors for different data in a chart.
pub trait ColorPalette: dyn_clone::DynClone {
    /// Given the index of the data point, select a base color to use.
    ///
    /// This function is expected to give the same answer for the same input (i.e. be a pure fn).
    fn color(&self, index: usize) -> piet::Color;
}

dyn_clone::clone_trait_object!(ColorPalette);

/// A default color palette for choosing contrasting colors for a chart.
#[derive(Copy, Clone)]
pub struct DefaultPalette;
impl ColorPalette for DefaultPalette {
    fn color(&self, index: usize) -> piet::Color {
        let hue = (index as f64 * 140.).rem_euclid(360.);
        piet::Color::hlc(hue, 40., 40.)
    }
}
