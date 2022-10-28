mod parse;
use crate::style::Color;
use kurbo::{Point, Size};

pub struct Circle {
    shape: kurbo::Circle,
    color: Color,
}

impl Circle {
    pub fn new(center: Point, radius: f64, color: Color) -> Self {
        Self {
            shape: kurbo::Circle { center, radius },
            color,
        }
    }

    pub fn render(&self, ctx: &mut impl piet::RenderContext) {
        let brush = ctx.solid_brush(self.color.to_piet_color());
        ctx.fill(self.shape, &brush);
    }

    pub fn to_svg(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        let mut rc = piet_svg::RenderContext::new(Size::new(1000., 1000.));
        self.render(&mut rc);
        rc.write(writer)
    }
}
