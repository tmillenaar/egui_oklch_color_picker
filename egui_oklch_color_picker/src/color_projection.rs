use crate::conversions::*;
use crate::coordinate_transforms::*;
use crate::oklab::*;
use crate::oklch::*;
use egui::*;

pub trait ColorProjection<C> {
    fn color_at(&self, coord: C) -> Oklch;
}

#[derive(Clone, Debug, PartialEq)]
pub struct StableHueProjection {
    pub hue: f32,
}

impl ColorProjection<CartesianCoord> for StableHueProjection {
    fn color_at(&self, coord: CartesianCoord) -> Oklch {
        Oklch {
            l: coord.y,
            c: coord.x * 0.325,
            h: self.hue,
            alpha: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StableChromaProjection {
    pub chroma: f32,
}

impl ColorProjection<PolarCoord> for StableChromaProjection {
    fn color_at(&self, coord: PolarCoord) -> Oklch {
        Oklch {
            l: 1.0 - coord.radius,
            c: self.chroma * 0.325,
            h: coord.angle,
            alpha: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StableLightnessProjection {
    pub lightness: f32,
}

impl ColorProjection<PolarCoord> for StableLightnessProjection {
    fn color_at(&self, coord: PolarCoord) -> Oklch {
        Oklch {
            l: self.lightness,
            c: coord.radius * 0.325,
            h: coord.angle,
            alpha: 1.0,
        }
    }
}
