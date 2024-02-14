use keyframe::{
    ease,
    functions::{EaseInOutQuad, EaseOutQuart},
};
use serde::{Deserialize, Serialize};
use tetra::{
    graphics::Color,
    math::{num_traits::clamp, Vec3},
};

use crate::util::HsvColor;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Palette {
    Simple {
        dark: Color,
        light: Color,
    },
    Lerp {
        dark1: Color,
        dark2: Color,
        light1: Color,
        light2: Color,
    },
    Trippy,
}

impl Default for Palette {
    fn default() -> Self {
        Self::Simple {
            dark: Color::BLACK,
            light: Color::WHITE,
        }
    }
}

impl Palette {
    pub fn default_simple() -> Palette {
        Palette::default()
    }

    pub fn default_lerp() -> Palette {
        Palette::Lerp {
            dark1: Color::BLACK,
            dark2: Color::BLACK,
            light1: Color::WHITE,
            light2: Color::WHITE,
        }
    }

    pub fn default_trippy() -> Palette {
        Palette::Trippy
    }

    pub fn default_all() -> [Palette; 3] {
        [
            Self::default_simple(),
            Self::default_lerp(),
            Self::default_trippy(),
        ]
    }

    pub fn type_str(&self) -> &str {
        match self {
            Palette::Simple { .. } => "Simple",
            Palette::Lerp { .. } => "Lerp",
            Palette::Trippy => "Trippy",
        }
    }
}

fn col_to_vec(c: Color) -> Vec3<f32> {
    Vec3::new(c.r, c.g, c.b)
}

fn vec_to_col(v: Vec3<f32>) -> Color {
    Color {
        r: v.x,
        g: v.y,
        b: v.z,
        a: 1.,
    }
}

fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let a = col_to_vec(a);
    let b = col_to_vec(b);
    let f = Vec3::lerp(a, b, t);
    vec_to_col(f)
}

pub struct PaletteSystem {
    palette: Palette,
    from_dark: Color,
    start_dark: Color,
    current_dark: Color,
    from_light: Color,
    start_light: Color,
    current_light: Color,
    start_timer: f32,
    lerp_progress: f32,
    lerp_back: bool,
    current_hue: f32,
}

impl PaletteSystem {
    const LERP_TIME: f32 = 4.;
    const START_TIME: f32 = 1.;
    const HUE_CYCLE_TIME: f32 = 6.;

    pub fn new(palette: Palette) -> Self {
        let mut ps = Self {
            palette,
            from_dark: Color::BLACK,
            start_dark: Color::BLACK,
            current_dark: Color::BLACK,
            from_light: Color::WHITE,
            start_light: Color::WHITE,
            current_light: Color::WHITE,
            start_timer: 0.,
            lerp_progress: 0.,
            lerp_back: false,
            current_hue: 0.,
        };
        ps.change_palette(palette);
        ps
    }

    pub fn change_palette(&mut self, palette: Palette) {
        self.palette = palette;
        self.start_timer = 0.;
        self.lerp_back = false;
        self.lerp_progress = 0.;
        self.from_dark = self.current_dark;
        self.from_light = self.current_light;
        self.start_dark = self.current_dark;
        self.start_light = self.current_light;
        self.current_hue = 0.;
    }

    pub fn update(&mut self, dt: f32) {
        match self.palette {
            Palette::Simple { dark, light } => {
                self.current_dark = dark;
                self.current_light = light;
            }
            Palette::Lerp {
                dark1,
                dark2,
                light1,
                light2,
            } => {
                match self.lerp_back {
                    false => self.lerp_progress += dt / Self::LERP_TIME,
                    true => self.lerp_progress -= dt / Self::LERP_TIME,
                }
                if !(0.0..1.0).contains(&self.lerp_progress) {
                    self.lerp_back = !self.lerp_back;
                    self.lerp_progress = clamp(self.lerp_progress, 0.0, 1.0);
                }
                let easing = ease(EaseInOutQuad, 0., 1., self.lerp_progress);
                self.current_dark = color_lerp(dark1, dark2, easing);
                self.current_light = color_lerp(light1, light2, easing);
            }
            Palette::Trippy => {
                let mut hue = self.current_hue;
                hue += dt / Self::HUE_CYCLE_TIME;
                if hue >= 1. {
                    hue -= 1.;
                }
                self.current_hue = hue;
                self.current_dark = HsvColor::new(hue, 1., 1.).into();
                hue += 0.5;
                if hue >= 1. {
                    hue -= 1.;
                }
                self.current_light = HsvColor::new(hue, 1., 1.).into();
            }
        }
        if self.start_timer < 1. {
            self.start_timer += dt / Self::START_TIME;
            let easing = ease(EaseOutQuart, 0., 1., self.start_timer);
            self.start_dark = color_lerp(self.from_dark, self.current_dark, easing);
            self.start_light = color_lerp(self.from_light, self.current_light, easing);
        }
    }

    pub fn dark(&self) -> &Color {
        if self.start_timer < 1. {
            return &self.start_dark;
        }
        &self.current_dark
    }

    pub fn light(&self) -> &Color {
        if self.start_timer < 1. {
            return &self.start_light;
        }
        &self.current_light
    }
}
