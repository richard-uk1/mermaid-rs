use anyhow::Result;
use kurbo::{Affine, CircleSegment, Point, Rect, Size};
use nom::Finish;
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};
use std::{
    f64::consts::{FRAC_PI_2, PI},
    fs, io,
    path::Path,
    sync::Arc,
};

mod parse;

const PIE_RADIUS: f64 = 100.;
const STROKE_THICKNESS: f64 = 1.5;
const FONT_HEIGHT: f64 = 10.;
const PADDING: f64 = 5.;

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
        // build text layouts
        let title = self
            .title
            .map(|title| {
                let title: Arc<str> = title.into();
                set_font_size::<RC>(ctx.text().new_text_layout(title), 12.).build()
            })
            .transpose()?;
        let legend = Legend::build(self, ctx)?;

        // build brushes
        let stroke_brush = ctx.solid_brush(piet::Color::BLACK);
        let color_brushes = (0..self.data.len())
            .map(|idx| {
                let hue = (idx as f64 * 140.).rem_euclid(360.);
                let color = piet::Color::hlc(hue, 40., 40.);
                ctx.solid_brush(color)
            })
            .collect::<Vec<_>>();

        ctx.clear(None, piet::Color::WHITE);

        // draw title
        if let Some(ref layout) = title {
            let size = layout.size();
            let title_tl = Point {
                x: PIE_RADIUS - size.width * 0.5 + 10.,
                y: 10.,
            };
            ctx.draw_text(&layout, title_tl);
        }

        // draw chart
        let y_offset = match title {
            Some(layout) => layout.size().height + 2. * 10.,
            None => 10.,
        };
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate((10., y_offset)));
            self.draw_pie(ctx, &stroke_brush, &color_brushes[..])
        })?;

        // draw legend
        ctx.with_save(|ctx| {
            let x_offset = PIE_RADIUS * 2. + 20. + 40.;
            let y_offset = y_offset + PIE_RADIUS - legend.size().height * 0.5;
            ctx.transform(Affine::translate((x_offset, y_offset)));
            legend.render(ctx, &stroke_brush, &color_brushes[..])
        })
    }

    /// Draw the actual pie shape with inner labels at (0, 0).
    fn draw_pie<RC: RenderContext>(
        &self,
        ctx: &mut RC,
        stroke_brush: &RC::Brush,
        color_brushes: &[RC::Brush],
    ) -> Result<(), piet::Error> {
        let total: f64 = self.data.iter().map(|d| d.value).sum();
        // the angle to start the segment at
        let mut segment_start = -FRAC_PI_2;

        let pie_center = Point::from((PIE_RADIUS, PIE_RADIUS));
        let pie_radius = PIE_RADIUS;

        for (datum, brush) in self.data.iter().zip(color_brushes) {
            // layout label
            let proportion = datum.value / total;
            let percentage_layout = set_font_size::<RC>(
                ctx.text()
                    .new_text_layout(format!("{:.0}%", proportion * 100.))
                    .text_color(piet::Color::WHITE),
                FONT_HEIGHT,
            )
            .build()?;
            let layout_size = percentage_layout.size();

            // draw segment
            let segment_sweep = PI * 2. * proportion;
            let segment = CircleSegment {
                center: pie_center,
                outer_radius: pie_radius,
                inner_radius: 0.,
                start_angle: segment_start,
                sweep_angle: segment_sweep,
            };
            ctx.fill(&segment, brush);

            ctx.stroke(&segment, stroke_brush, STROKE_THICKNESS);

            // draw label
            let segment_center = segment_start + segment_sweep * 0.5;
            let label_center = Point {
                x: pie_center.x + segment_center.cos() * pie_radius * 0.5,
                y: pie_center.y + segment_center.sin() * pie_radius * 0.5,
            };
            let label_tl = Point {
                x: label_center.x - layout_size.width * 0.5,
                y: label_center.y - layout_size.height * 0.5,
            };
            ctx.draw_text(&percentage_layout, label_tl);

            segment_start += segment_sweep;
        }

        Ok(())
    }

    pub fn to_svg(&self, writer: impl io::Write) -> io::Result<()> {
        let mut rc = piet_svg::RenderContext::new(Size::new(800., 800.));
        self.render(&mut rc).unwrap();
        rc.write(writer)
    }

    pub fn to_svg_file(&self, filename: impl AsRef<Path>) -> io::Result<()> {
        let file = io::BufWriter::new(fs::File::create(filename)?);
        self.to_svg(file)?;
        Ok(())
    }

    pub fn to_png_file(&self, filename: impl AsRef<Path>) -> io::Result<()> {
        let mut device = piet_common::Device::new().unwrap();
        let mut bitmap = device.bitmap_target(800, 800, 1.).unwrap();
        let mut rc = bitmap.render_context();
        self.render(&mut rc).unwrap();
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

// For render

struct Legend<RC: RenderContext> {
    layouts: Vec<RC::TextLayout>,
    size: Size,
}

impl<RC: RenderContext> Legend<RC> {
    fn build(pie: &Pie, ctx: &mut RC) -> Result<Self, piet::Error> {
        let layouts = pie
            .data
            .iter()
            .map(|datum| {
                let text = if pie.show_data {
                    format!("{} [{}]", datum.label, datum.value)
                } else {
                    datum.label.to_string()
                };
                Ok(set_font_size::<RC>(ctx.text().new_text_layout(text), FONT_HEIGHT).build()?)
            })
            .collect::<Result<Vec<_>, piet::Error>>()?;

        // calculate size
        let mut width: f64 = 0.;
        for layout in &layouts {
            let size = layout.size();
            width = width.max(size.width);
        }
        let size = Size {
            // 10. for color + 3*10. for padding
            width: width + 10. + 3. * PADDING,
            // (n+1) * 10. for padding
            height: (10. + PADDING) * layouts.len() as f64 + PADDING,
        };

        Ok(Legend { layouts, size })
    }

    fn size(&self) -> Size {
        self.size
    }

    fn render(
        &self,
        ctx: &mut RC,
        stroke_brush: &RC::Brush,
        color_brushes: &[RC::Brush],
    ) -> Result<(), piet::Error> {
        const COLOR_WIDTH: f64 = FONT_HEIGHT;

        // draw outline
        let outline = self.size.to_rect();
        ctx.stroke(outline, stroke_brush, STROKE_THICKNESS);

        let mut top = PADDING;
        for (layout, brush) in self.layouts.iter().zip(color_brushes) {
            let color_sq_tl = Point::new(PADDING, top);
            let color_sq_sz = Size::new(COLOR_WIDTH, COLOR_WIDTH);
            let color_square = Rect::from_origin_size(color_sq_tl, color_sq_sz);
            ctx.stroke(color_square, stroke_brush, STROKE_THICKNESS);
            ctx.fill(color_square, brush);
            ctx.draw_text(layout, Point::new(2. * PADDING + COLOR_WIDTH, top));
            top += COLOR_WIDTH + PADDING;
        }

        Ok(())
    }
}

fn set_font_size<RC: RenderContext>(
    builder: <RC::Text as Text>::TextLayoutBuilder,
    font_size_px: f64,
) -> <RC::Text as Text>::TextLayoutBuilder {
    builder.default_attribute(piet::TextAttribute::FontSize(px_to_pt(font_size_px)))
}

fn px_to_pt(px: f64) -> f64 {
    0.75 * px
}
