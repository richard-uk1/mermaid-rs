use anyhow::Result;
use kurbo::Size;
use nom::Finish;
use once_cell::sync::Lazy;
use piet::{Color, RenderContext};
use std::{fmt, fs, io, path::Path};

mod parse;
mod render;

/// The default style used with [`Pie::render`].
pub static DEFAULT_STYLE: Lazy<PieStyle> = Lazy::new(PieStyle::default);
pub static DARK_STYLE: Lazy<PieStyle> = Lazy::new(PieStyle::default_dark);

#[derive(Debug)]
pub struct Pie<'input> {
    pub title: Option<&'input str>,
    pub show_data: bool,
    pub data: Vec<Datum<'input>>,
}

impl<'input> Pie<'input> {
    pub fn parse(src: &'input str) -> Result<Self> {
        let (_, pie) = parse::parse_pie(src.trim()).finish().unwrap();
        Ok(pie)
    }

    pub fn render<RC: RenderContext>(&self, ctx: &mut RC) -> Result<(), piet::Error> {
        self.render_with_style(&DEFAULT_STYLE, ctx)
    }

    pub fn render_with_style<RC: RenderContext>(
        &self,
        style: &PieStyle,
        ctx: &mut RC,
    ) -> Result<(), piet::Error> {
        render::render(self, style, ctx)
    }

    pub fn to_svg(&self, writer: impl io::Write, style: Option<&PieStyle>) -> io::Result<()> {
        let mut rc = piet_svg::RenderContext::new(Size::new(800., 800.));
        if let Some(style) = style {
            self.render_with_style(style, &mut rc).unwrap();
        } else {
            self.render(&mut rc).unwrap();
        }
        rc.write(writer)
    }

    pub fn to_svg_file(
        &self,
        filename: impl AsRef<Path>,
        style: Option<&PieStyle>,
    ) -> io::Result<()> {
        let file = io::BufWriter::new(fs::File::create(filename)?);
        self.to_svg(file, style)?;
        Ok(())
    }

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

#[derive(Debug)]
pub struct Datum<'input> {
    pub label: &'input str,
    pub value: f64,
}

#[derive(Clone)]
pub struct PieStyle {
    pub background_color: Color,
    pub title: TextStyle,
    pub segment_outline_color: Color,
    pub segment_outline_thickness: f64,
    pub segment_colors: Box<dyn ColorPalette + Send + Sync>,
    // if `None` labels will not be drawn
    pub segment_label: Option<TextStyle>,
    pub legend_label: TextStyle,
}

impl fmt::Debug for PieStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PieStyle")
            .field("background_color", &self.background_color)
            .field("title", &self.title)
            .field("segment_outline_color", &self.segment_outline_color)
            .field("segment_outline_thickness", &self.segment_outline_thickness)
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
            segment_outline_color: Color::BLACK,
            segment_outline_thickness: 1.5,
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

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: Color,
    pub font_size: f64,
    pub bold: bool,
}

impl TextStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            font_size: 16.,
            bold: false,
        }
    }

    fn default_dark() -> Self {
        Self {
            color: Color::WHITE,
            font_size: 16.,
            bold: false,
        }
    }

    fn with_font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }
}

pub trait ColorPalette: dyn_clone::DynClone {
    /// This function is expected to give the same answer for the same input (i.e. be a pure fn).
    fn color(&self, index: usize) -> piet::Color;
}

dyn_clone::clone_trait_object!(ColorPalette);

#[derive(Copy, Clone)]
pub struct DefaultPalette;
impl ColorPalette for DefaultPalette {
    fn color(&self, index: usize) -> piet::Color {
        let hue = (index as f64 * 140.).rem_euclid(360.);
        piet::Color::hlc(hue, 40., 40.)
    }
}
