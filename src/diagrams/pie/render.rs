use super::{Pie, PieStyle, TextStyle};
use anyhow::Result;
use kurbo::{Affine, CircleSegment, Point, Rect, Size};
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};
use std::{
    f64::consts::{FRAC_PI_2, PI},
    sync::Arc,
};

const PIE_RADIUS: f64 = 100.;
const STROKE_THICKNESS: f64 = 1.5;
const PADDING: f64 = 5.;

pub fn render<RC: RenderContext>(
    chart: &Pie,
    style: &PieStyle,
    ctx: &mut RC,
) -> Result<(), piet::Error> {
    // build text layouts
    let title = chart
        .title
        .map(|title| {
            let title: Arc<str> = title.into();

            ctx.text()
                .new_text_layout(title)
                .apply_style(&style.title)
                .build()
        })
        .transpose()?;
    let legend = Legend::build(chart, style, ctx)?;

    // build brushes
    let stroke_brush = ctx.solid_brush(style.segment_outline.color);
    let color_brushes = (0..chart.data.len())
        .map(|idx| {
            let color = style.segment_colors.color(idx);
            ctx.solid_brush(color)
        })
        .collect::<Vec<_>>();

    ctx.clear(None, style.background_color);

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
        draw_pie(chart, style, ctx, &stroke_brush, &color_brushes[..])
    })?;

    // draw legend
    ctx.with_save(|ctx| {
        let x_offset = PIE_RADIUS * 2. + 20. + 40.;
        let y_offset = y_offset + PIE_RADIUS - legend.size().height * 0.5;
        ctx.transform(Affine::translate((x_offset, y_offset)));
        legend.render(ctx, style, &stroke_brush, &color_brushes[..])
    })
}

/// Draw the actual pie shape with inner labels at (0, 0).
fn draw_pie<RC: RenderContext>(
    chart: &Pie,
    style: &PieStyle,
    ctx: &mut RC,
    stroke_brush: &RC::Brush,
    color_brushes: &[RC::Brush],
) -> Result<(), piet::Error> {
    let total: f64 = chart.data.iter().map(|d| d.value).sum();
    // the angle to start the segment at
    let mut segment_start = -FRAC_PI_2;

    let pie_center = Point::from((PIE_RADIUS, PIE_RADIUS));
    let pie_radius = PIE_RADIUS;

    for (datum, brush) in chart.data.iter().zip(color_brushes) {
        let proportion = datum.value / total;
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
        ctx.stroke(&segment, stroke_brush, style.segment_outline.width);

        if let Some(ref label_style) = style.segment_label {
            // layout label
            let percentage_layout = ctx
                .text()
                .new_text_layout(format!("{:.0}%", proportion * 100.))
                .apply_style(label_style)
                .build()?;
            let layout_size = percentage_layout.size();

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
        }

        segment_start += segment_sweep;
    }

    Ok(())
}

struct Legend<RC: RenderContext> {
    layouts: Vec<RC::TextLayout>,
    size: Size,
}

impl<RC: RenderContext> Legend<RC> {
    fn build(pie: &Pie, style: &PieStyle, ctx: &mut RC) -> Result<Self, piet::Error> {
        let layouts = pie
            .data
            .iter()
            .map(|datum| {
                let text = if pie.show_data {
                    format!("{} [{}]", datum.label, datum.value)
                } else {
                    datum.label.to_string()
                };
                Ok(ctx
                    .text()
                    .new_text_layout(text)
                    .apply_style(&style.legend_label)
                    .build()?)
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
            width: width + style.legend_label.font_size + 3. * PADDING,
            // (n+1) * 10. for padding
            height: (style.legend_label.font_size + PADDING) * layouts.len() as f64 + PADDING,
        };

        Ok(Legend { layouts, size })
    }

    fn size(&self) -> Size {
        self.size
    }

    fn render(
        &self,
        ctx: &mut RC,
        style: &PieStyle,
        stroke_brush: &RC::Brush,
        color_brushes: &[RC::Brush],
    ) -> Result<(), piet::Error> {
        let color_width = style.legend_label.font_size;

        // draw outline
        let outline = self.size.to_rect();
        ctx.stroke(outline, stroke_brush, STROKE_THICKNESS);

        let mut top = PADDING;
        for (layout, brush) in self.layouts.iter().zip(color_brushes) {
            let color_sq_tl = Point::new(PADDING, top);
            let color_sq_sz = Size::new(color_width, color_width);
            let color_square = Rect::from_origin_size(color_sq_tl, color_sq_sz);
            ctx.stroke(color_square, stroke_brush, STROKE_THICKNESS);
            ctx.fill(color_square, brush);
            ctx.draw_text(layout, Point::new(2. * PADDING + color_width, top));
            top += color_width + PADDING;
        }

        Ok(())
    }
}

trait ApplyStyle {
    fn apply_style(self, style: &TextStyle) -> Self;
}

impl<T: TextLayoutBuilder> ApplyStyle for T {
    fn apply_style(self, style: &TextStyle) -> Self {
        let mut this =
            self.default_attribute(piet::TextAttribute::FontSize(px_to_pt(style.font_size)));
        if style.bold {
            this = this.default_attribute(piet::TextAttribute::Weight(piet::FontWeight::BOLD));
        }
        this.text_color(style.color)
    }
}

fn px_to_pt(px: f64) -> f64 {
    0.75 * px
}
