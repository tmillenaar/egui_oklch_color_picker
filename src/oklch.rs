use crate::conversions::*;
use crate::oklab::*;
use egui::{Color32, Rgba};

/// Oklch + alpha.
///
/// L is normally in `[0,1]`.
/// C is unbounded (though values above ~0.4 are outside sRGB).
/// H is normalized to `[0,1)`, where `1.0 == 0.0`.
/// Alpha is in `[0,1]`.
/// A negative alpha signifies an additive color.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Oklch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
    pub alpha: f32,
}

impl Oklch {
    #[inline]
    pub fn new(l: f32, c: f32, h: f32, alpha: f32) -> Self {
        Self { l, c, h, alpha }
    }

    //------------------------------------------------------------------------
    // Constructors
    //------------------------------------------------------------------------

    #[inline]
    pub fn from_oklaba(oklab: Oklab) -> Self {
        oklab_to_oklch(oklab)
    }

    #[inline]
    pub fn to_oklaba(self) -> Oklab {
        oklch_to_oklab(self)
    }

    #[inline]
    pub fn to_opaque(self) -> Self {
        Self { alpha: 1.0, ..self }
    }

    //------------------------------------------------------------------------
    // RGB helpers
    //------------------------------------------------------------------------

    #[inline]
    pub fn to_rgb(self) -> [f32; 3] {
        self.to_oklaba().to_rgb()
    }

    #[inline]
    pub fn to_srgb(self) -> [u8; 3] {
        self.to_oklaba().to_srgb()
    }

    #[inline]
    pub fn to_rgba_unmultiplied(self) -> [f32; 4] {
        self.to_oklaba().to_rgba_unmultiplied()
    }

    #[inline]
    pub fn to_rgba_premultiplied(self) -> [f32; 4] {
        self.to_oklaba().to_rgba_premultiplied()
    }

    #[inline]
    pub fn to_srgba_unmultiplied(self) -> [u8; 4] {
        self.to_oklaba().to_srgba_unmultiplied()
    }

    #[inline]
    pub fn to_srgba_premultiplied(self) -> [u8; 4] {
        Color32::from(self).to_array()
    }

    #[inline]
    pub fn is_valid_srgb(&self) -> bool {
        self.to_oklaba().is_valid_srgb()
    }

    //------------------------------------------------------------------------
    // Constructors from RGB
    //------------------------------------------------------------------------

    #[inline]
    pub fn from_rgb(rgb: [f32; 3]) -> Self {
        Self::from(Oklab::from_rgb(rgb))
    }

    #[inline]
    pub fn from_srgb(rgb: [u8; 3]) -> Self {
        Self::from(Oklab::from_srgb(rgb))
    }

    #[inline]
    pub fn from_rgba_unmultiplied(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self::from(Oklab::from_rgba_unmultiplied(r, g, b, alpha))
    }

    #[inline]
    pub fn from_rgba_premultiplied(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self::from(Oklab::from_rgba_premultiplied(r, g, b, alpha))
    }

    #[inline]
    pub fn from_srgba_unmultiplied([r, g, b, a]: [u8; 4]) -> Self {
        Self::from(Color32::from_rgba_unmultiplied(r, g, b, a))
    }

    #[inline]
    pub fn from_srgba_premultiplied([r, g, b, a]: [u8; 4]) -> Self {
        Self::from(Color32::from_rgba_premultiplied(r, g, b, a))
    }

    #[inline]
    pub fn from_additive_rgb(rgb: [f32; 3]) -> Self {
        Self::from(Oklab::from_additive_rgb(rgb))
    }

    #[inline]
    pub fn from_additive_srgb(rgb: [u8; 3]) -> Self {
        Self::from(Oklab::from_additive_srgb(rgb))
    }
}

impl From<Oklab> for Oklch {
    #[inline]
    fn from(oklab: Oklab) -> Self {
        Self::from_oklaba(oklab)
    }
}

impl From<Oklch> for Oklab {
    #[inline]
    fn from(oklch: Oklch) -> Self {
        oklch.to_oklaba()
    }
}

impl From<Oklch> for Rgba {
    #[inline]
    fn from(oklch: Oklch) -> Self {
        Oklab::from(oklch).into()
    }
}

impl From<Rgba> for Oklch {
    #[inline]
    fn from(rgba: Rgba) -> Self {
        Oklab::from(rgba).into()
    }
}

impl From<Oklch> for Color32 {
    #[inline]
    fn from(oklch: Oklch) -> Self {
        Oklab::from(oklch).into()
    }
}

impl From<Color32> for Oklch {
    #[inline]
    fn from(color: Color32) -> Self {
        Oklab::from(color).into()
    }
}
