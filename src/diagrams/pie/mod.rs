//! Types and functions for creating pie charts.

use crate::style::{ColorPalette, DefaultPalette, StrokeStyle, TextStyle};
use anyhow::Result;
use kurbo::Size;
use nom::Finish;
use once_cell::sync::Lazy;
use piet::{Color, RenderContext};
use std::{fmt, fs, io, path::Path};

mod parse;
mod render;

pub use parse::{Error, ErrorKind};

/// The default style used with [`Pie::render`].
pub static DEFAULT_STYLE: Lazy<PieStyle> = Lazy::new(PieStyle::default);
/// A default style for use with dark themes.
pub static DARK_STYLE: Lazy<PieStyle> = Lazy::new(PieStyle::default_dark);

/// A parsed pie chart.
#[derive(Debug)]
pub struct Pie<'input> {
    /// A title to display above the chart.
    ///
    /// If `Some("")` then space will be left for a title, wherease if `None`, then no space will
    /// be taken.
    pub title: Option<&'input str>,
    /// Whether to show the values of the data in the legend.
    pub show_data: bool,
    /// The data to chart.
    pub data: Vec<Datum<'input>>,
}

impl<'input> Pie<'input> {
    /// Parse a chart description.
    pub fn parse(src: &'input str) -> Result<Self, Error> {
        let (_, pie) = parse::parse_pie(src).finish()?;
        Ok(pie)
    }

    /// Use a [`piet::RenderContext`] to render this chart.
    pub fn render<RC: RenderContext>(&self, ctx: &mut RC) -> Result<(), piet::Error> {
        self.render_with_style(&DEFAULT_STYLE, ctx)
    }

    /// Like [`Pie::render`] but allows specifying a custom style.
    pub fn render_with_style<RC: RenderContext>(
        &self,
        style: &PieStyle,
        ctx: &mut RC,
    ) -> Result<(), piet::Error> {
        render::render(self, style, ctx)
    }

    /// Write out an svg image to `writer`, with optional custom styling.
    pub fn to_svg(&self, writer: impl io::Write, style: Option<&PieStyle>) -> io::Result<()> {
        let mut rc = piet_svg::RenderContext::new(Size::new(800., 800.));
        if let Some(style) = style {
            self.render_with_style(style, &mut rc).unwrap();
        } else {
            self.render(&mut rc).unwrap();
        }
        rc.write(writer)
    }

    /// Write out an svg image to a file at `filename`, with optional custom styling.
    pub fn to_svg_file(
        &self,
        filename: impl AsRef<Path>,
        style: Option<&PieStyle>,
    ) -> io::Result<()> {
        let file = io::BufWriter::new(fs::File::create(filename)?);
        self.to_svg(file, style)?;
        Ok(())
    }

    /// Write out a png image to a file at `filename`, with optional custom styling.
    ///
    /// `px_scale` allows for rendering at a larger scale, either for extra zoom or for high DPI
    /// screens.
    pub fn to_png_file(
        &self,
        filename: impl AsRef<Path>,
        px_scale: f64,
        style: Option<&PieStyle>,
    ) -> io::Result<()> {
        let mut device = piet_common::Device::new().unwrap();
        let size = (800. * px_scale) as usize;
        let mut bitmap = device.bitmap_target(size, size, px_scale).unwrap();
        let mut rc = bitmap.render_context();
        if let Some(style) = style {
            self.render_with_style(style, &mut rc).unwrap();
        } else {
            self.render(&mut rc).unwrap();
        }
        rc.finish().unwrap();
        drop(rc);

        bitmap.save_to_file(filename).unwrap();
        Ok(())
    }
}

/// A numeric data point in the pie chart.
#[derive(Debug)]
pub struct Datum<'input> {
    /// What to label this data point in the legend.
    pub label: &'input str,
    /// The data value.
    pub value: f64,
}

/// Styling for the pie chart.
#[derive(Clone)]
pub struct PieStyle {
    /// What color to clear the background with.
    ///
    /// The default is transparent.
    pub background_color: Color,
    /// How to style the title text.
    pub title: TextStyle,
    /// How to style the outline of pie segments.
    pub segment_outline: StrokeStyle,
    /// How to choose the color of each pie segment.
    pub segment_colors: Box<dyn ColorPalette + Send + Sync>,
    /// How to style segment labels (showing the percentage of the total a particular segment takes
    /// up).
    ///
    /// If this is `None` then labels will not be drawn.
    pub segment_label: Option<TextStyle>,
    /// How to style the labels for each data point in the legend.
    pub legend_label: TextStyle,
}

impl fmt::Debug for PieStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PieStyle")
            .field("background_color", &self.background_color)
            .field("title", &self.title)
            .field("segment_outline", &self.segment_outline)
            .field("segment_colors", &"dyn ColorPalette")
            .field("segment_label", &self.segment_label)
            .field("legend_label", &self.legend_label)
            .finish()
    }
}

impl PieStyle {
    pub fn default() -> Self {
        Self {
            background_color: Color::TRANSPARENT,
            title: TextStyle::default().with_bold(true),
            segment_outline: StrokeStyle::new(1.5, Color::BLACK),
            segment_colors: Box::new(DefaultPalette),
            segment_label: Some(TextStyle::default_dark().with_font_size(12.)),
            legend_label: TextStyle::default(),
        }
    }
    pub fn default_dark() -> Self {
        let mut this = Self::default();
        this.title = TextStyle::default_dark().with_bold(true);
        this.legend_label = TextStyle::default_dark();
        this
    }
}
