use crate::conversions::*;
use crate::oklaba::*;
use crate::oklcha::*;
use crate::coordinate_transforms::*;
use egui::*;

pub trait ColorProjection<C> {
    fn color_at(&self, coord: C) -> Oklcha;
}

#[derive(Clone, Debug, PartialEq)]
pub struct StableHueProjection {
    pub hue: f32,
}

impl ColorProjection<CartesianCoord> for StableHueProjection {
    fn color_at(&self, coord: CartesianCoord) -> Oklcha {
        Oklcha {
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
    fn color_at(&self, coord: PolarCoord) -> Oklcha {
        Oklcha {
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
    fn color_at(&self, coord: PolarCoord) -> Oklcha {
        Oklcha {
            l: self.lightness,
            c: coord.radius * 0.325,
            h: coord.angle,
            alpha: 1.0,
        }
    }
}
