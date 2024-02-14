use tetra::graphics::Color;

pub struct HsvColor {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl HsvColor {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h, s, v }
    }
}

fn max_f32(x: f32, y: f32) -> f32 {
    if x > y {
        x
    }
    else {
        y
    }
}

fn min_f32(x: f32, y: f32) -> f32 {
    if x < y {
        x
    }
    else {
        y
    }
}

impl From<Color> for HsvColor {
    fn from(value: Color) -> Self {
        let r = value.r;
        let g = value.g;
        let b = value.b;
        let c_max = max_f32(r, max_f32(g, b));
        let c_min = min_f32(r, min_f32(g, b));
        let d = c_max - c_min;
        let h = {
            if d == 0. {
                0.
            }
            else if c_max == r {
                60. * (((g - b) / d) % 6.)
            }
            else if c_max == g {
                60. * (((b - r) / d) + 2.)
            }
            else if c_max == b {
                60. * (((r - g) / d) + 4.)
            }
            else {
                0.
            }
        };
        let s = {
            if c_max == 0. {
                0.
            }
            else {
                d / c_max
            }
        };
        let v = c_max;
        Self { h, s, v }
    }
}

impl Into<Color> for HsvColor {
    fn into(self) -> Color {
        let h = self.h * 360.;
        let s = self.s;
        let v = self.v;

        let c = v * s;
        let x = c * (1. - ((h / 60.) % 2. - 1.).abs());
        let m = v - c;
        let (r, g, b) = match h {
            a if (0.0..60.0).contains(&a) => (c, x, 0.),
            a if (60.0..120.0).contains(&a) => (x, c, 0.),
            a if (120.0..180.0).contains(&a) => (0., c, x),
            a if (180.0..240.0).contains(&a) => (0., x, c),
            a if (240.0..300.0).contains(&a) => (x, 0., c),
            a if (300.0..360.0).contains(&a) => (c, 0., x),
            _ => (0., 0., 0.),
        };
        Color::rgb(r + m, g + m, b + m)
    }
}
