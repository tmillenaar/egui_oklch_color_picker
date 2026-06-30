use crate::conversions::*;
use crate::coordinate_transforms::*;
use egui::*;

pub struct ColorSample {
    pub valid: bool,
    pub color: Color32,
}

pub trait ColorProjection<C> {
    fn color_at(&self, coord: C) -> ColorSample;
}

#[derive(Clone, Debug, PartialEq)]
pub struct StableHueProjection {
    pub hue: f32,
}

impl ColorProjection<CartesianCoord> for StableHueProjection {
    fn color_at(&self, coord: CartesianCoord) -> ColorSample {
        oklch_to_color32(Oklch {
            l: coord.y,
            c: coord.x * 0.325,
            h: self.hue,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StableChromaProjection {
    pub chroma: f32,
}

impl ColorProjection<PolarCoord> for StableChromaProjection {
    fn color_at(&self, coord: PolarCoord) -> ColorSample {
        oklch_to_color32(Oklch {
            l: 1.0 - coord.radius,
            c: self.chroma * 0.325,
            h: coord.angle,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StableLightnessProjection {
    pub lightness: f32,
}

impl ColorProjection<PolarCoord> for StableLightnessProjection {
    fn color_at(&self, coord: PolarCoord) -> ColorSample {
        oklch_to_color32(Oklch {
            l: self.lightness,
            c: coord.radius * 0.325,
            h: coord.angle,
        })
    }
}
