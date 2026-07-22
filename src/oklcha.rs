use crate::conversions::*;
use crate::oklaba::*;
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
pub struct Oklcha {
    pub l: f32,
    pub c: f32,
    pub h: f32,
    pub alpha: f32,
}

impl Oklcha {
    #[inline]
    pub fn new(l: f32, c: f32, h: f32, alpha: f32) -> Self {
        Self { l, c, h, alpha }
    }

    //------------------------------------------------------------------------
    // Constructors
    //------------------------------------------------------------------------

    #[inline]
    pub fn from_oklaba(oklaba: Oklaba) -> Self {
        oklab_to_oklch(oklaba)
    }

    #[inline]
    pub fn to_oklaba(self) -> Oklaba {
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
        Self::from(Oklaba::from_rgb(rgb))
    }

    #[inline]
    pub fn from_srgb(rgb: [u8; 3]) -> Self {
        Self::from(Oklaba::from_srgb(rgb))
    }

    #[inline]
    pub fn from_rgba_unmultiplied(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self::from(Oklaba::from_rgba_unmultiplied(r, g, b, alpha))
    }

    #[inline]
    pub fn from_rgba_premultiplied(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self::from(Oklaba::from_rgba_premultiplied(r, g, b, alpha))
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
        Self::from(Oklaba::from_additive_rgb(rgb))
    }

    #[inline]
    pub fn from_additive_srgb(rgb: [u8; 3]) -> Self {
        Self::from(Oklaba::from_additive_srgb(rgb))
    }
}

impl From<Oklaba> for Oklcha {
    #[inline]
    fn from(oklaba: Oklaba) -> Self {
        Self::from_oklaba(oklaba)
    }
}

impl From<Oklcha> for Oklaba {
    #[inline]
    fn from(oklcha: Oklcha) -> Self {
        oklcha.to_oklaba()
    }
}

impl From<Oklcha> for Rgba {
    #[inline]
    fn from(oklcha: Oklcha) -> Self {
        Oklaba::from(oklcha).into()
    }
}

impl From<Rgba> for Oklcha {
    #[inline]
    fn from(rgba: Rgba) -> Self {
        Oklaba::from(rgba).into()
    }
}

impl From<Oklcha> for Color32 {
    #[inline]
    fn from(oklcha: Oklcha) -> Self {
        Oklaba::from(oklcha).into()
    }
}

impl From<Color32> for Oklcha {
    #[inline]
    fn from(color: Color32) -> Self {
        Oklaba::from(color).into()
    }
}
