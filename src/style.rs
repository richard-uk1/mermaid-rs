#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.) as u8,
            g: (g.clamp(0.0, 1.0) * 255.) as u8,
            b: (b.clamp(0.0, 1.0) * 255.) as u8,
            a: (a.clamp(0.0, 1.0) * 255.) as u8,
        }
    }

    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self::rgba(r, g, b, 1.)
    }

    pub(crate) fn to_piet_color(&self) -> piet::Color {
        piet::Color::rgba8(self.r, self.g, self.b, self.a)
    }
}
