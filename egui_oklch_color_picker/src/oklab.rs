use crate::conversions::*;
use crate::ecolor::{
    gamma_u8_from_linear_f32, linear_f32_from_gamma_u8, linear_u8_from_linear_f32,
};
use crate::{Color32, Rgba};

/// Oklab + alpha.
///
/// L is normally in `[0,1]`.
/// Alpha is in `[0,1]`.
/// A negative alpha signifies an additive color, matching `Hsva`.
///
/// To convert to an egui::Color32, use ```egui::Color32::from(oklab)```
///
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Default, PartialOrd, PartialEq)]
pub struct Oklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,

    /// Alpha in `[0,1]`.
    /// A negative value signifies an additive color.
    pub alpha: f32,
}

impl Oklab {
    /// Creates a new OKLab color with the given lightness, chroma components, and
    /// alpha value.
    #[inline]
    pub fn new(l: f32, a: f32, b: f32, alpha: f32) -> Self {
        Self { l, a, b, alpha }
    }

    //------------------------------------------------------------------------
    // Constructors
    //------------------------------------------------------------------------

    #[inline]
    /// Creates an `Oklab` color from an unmultiplied sRGBA color.
    pub fn from_srgba_unmultiplied([r, g, b, a]: [u8; 4]) -> Self {
        Self::from(Color32::from_rgba_unmultiplied(r, g, b, a))
    }

    /// Creates an `Oklab` color from a premultiplied sRGBA color.
    #[inline]
    pub fn from_srgba_premultiplied([r, g, b, a]: [u8; 4]) -> Self {
        Self::from(Color32::from_rgba_premultiplied(r, g, b, a))
    }

    /// Creates an opaque `Oklab` color from linear RGB components.
    #[inline]
    pub fn from_rgb(rgb: [f32; 3]) -> Self {
        let oklab = linear_srgb_to_oklab(LinearSrgb {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
        });

        Self {
            l: oklab.l,
            a: oklab.a,
            b: oklab.b,
            alpha: 1.0,
        }
    }

    /// Creates an opaque `Oklab` color from gamma-encoded sRGB components.
    #[inline]
    pub fn from_srgb([r, g, b]: [u8; 3]) -> Self {
        Self::from_rgb([
            linear_f32_from_gamma_u8(r),
            linear_f32_from_gamma_u8(g),
            linear_f32_from_gamma_u8(b),
        ])
    }

    /// Creates an additive `Oklab` color from linear RGB components.
    ///
    /// The resulting color has a negative alpha value, matching the additive color
    /// convention used by `egui`.
    #[inline]
    pub fn from_additive_rgb(rgb: [f32; 3]) -> Self {
        let mut out = Self::from_rgb(rgb);
        out.alpha = -0.5;
        out
    }

    /// Creates an additive `Oklab` color from gamma-encoded sRGB components.
    ///
    /// The resulting color has a negative alpha value, matching the additive color
    /// convention used by `egui`.
    #[inline]
    pub fn from_additive_srgb([r, g, b]: [u8; 3]) -> Self {
        Self::from_additive_rgb([
            linear_f32_from_gamma_u8(r),
            linear_f32_from_gamma_u8(g),
            linear_f32_from_gamma_u8(b),
        ])
    }


    /// Creates an `Oklab` color from unmultiplied linear RGBA components.
    #[inline]
    pub fn from_rgba_unmultiplied(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        let oklab = linear_srgb_to_oklab(LinearSrgb { r, g, b });

        Self {
            l: oklab.l,
            a: oklab.a,
            b: oklab.b,
            alpha,
        }
    }

    /// Creates an `Oklab` color from premultiplied linear RGBA components.
    ///
    /// If `alpha` is zero and the color channels are non-zero, the color is treated
    /// as additive.
    #[inline]
    pub fn from_rgba_premultiplied(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        if alpha <= 0.0 {
            if r == 0.0 && g == 0.0 && b == 0.0 {
                Self::default()
            } else {
                Self::from_additive_rgb([r, g, b])
            }
        } else {
            Self::from_rgba_unmultiplied(r / alpha, g / alpha, b / alpha, alpha)
        }
    }

    //------------------------------------------------------------------------
    // Conversion
    //------------------------------------------------------------------------

    /// Returns a copy of this color with its alpha channel set to `1.0`.
    #[inline]
    pub fn to_opaque(self) -> Self {
        Self { alpha: 1.0, ..self }
    }

    /// Converts this color to linear RGB.
    ///
    /// Colors outside the sRGB gamut are converted without clipping. Use
    /// [`Oklab::is_valid_srgb`] to determine whether the result lies within the
    /// displayable sRGB gamut.
    #[inline]
    pub fn to_rgb(self) -> [f32; 3] {
        let (is_valid, rgb) = oklab_to_linear_srgb(self);

        // TODO: add fallback on crhoma

        [rgb.r, rgb.g, rgb.b]
    }

    /// Converts this color to gamma-encoded sRGB.
    #[inline]
    pub fn to_srgb(self) -> [u8; 3] {
        let [r, g, b] = self.to_rgb();

        [
            gamma_u8_from_linear_f32(r),
            gamma_u8_from_linear_f32(g),
            gamma_u8_from_linear_f32(b),
        ]
    }

    /// Converts this color to unmultiplied linear RGBA.
    #[inline]
    pub fn to_rgba_unmultiplied(self) -> [f32; 4] {
        let [r, g, b] = self.to_rgb();

        [r, g, b, self.alpha]
    }


    /// Converts this color to premultiplied linear RGBA.
    ///
    /// Additive colors are returned with an alpha value of `0.0`.
    #[inline]
    pub fn to_rgba_premultiplied(self) -> [f32; 4] {
        let [r, g, b, alpha] = self.to_rgba_unmultiplied();

        if alpha < 0.0 {
            [r, g, b, 0.0]
        } else {
            [r * alpha, g * alpha, b * alpha, alpha]
        }
    }

    /// Converts this color to unmultiplied sRGBA.
    #[inline]
    pub fn to_srgba_unmultiplied(self) -> [u8; 4] {
        let [r, g, b, alpha] = self.to_rgba_unmultiplied();

        [
            gamma_u8_from_linear_f32(r),
            gamma_u8_from_linear_f32(g),
            gamma_u8_from_linear_f32(b),
            linear_u8_from_linear_f32(alpha.abs()),
        ]
    }

    /// Converts this color to premultiplied sRGBA.
    #[inline]
    pub fn to_srgba_premultiplied(self) -> [u8; 4] {
        Color32::from(self).to_array()
    }

    /// Returns `true` if this color lies within the displayable sRGB gamut.
    #[inline]
    pub fn is_valid_srgb(&self) -> bool {
        let (is_valid, rgb) = oklab_to_linear_srgb(*self);
        is_valid
    }
}

impl From<Oklab> for Rgba {
    #[inline]
    fn from(oklab: Oklab) -> Self {
        let rgba_u8 = oklab.to_rgba_premultiplied();
        Rgba::from_rgba_premultiplied(rgba_u8[0], rgba_u8[1], rgba_u8[2], rgba_u8[3])
    }
}

impl From<Rgba> for Oklab {
    #[inline]
    fn from(rgba: Rgba) -> Self {
        Self::from_rgba_premultiplied(rgba.r(), rgba.g(), rgba.b(), rgba.a())
    }
}

impl From<Oklab> for Color32 {
    #[inline]
    fn from(oklab: Oklab) -> Self {
        Self::from(Rgba::from(oklab))
    }
}

impl From<Color32> for Oklab {
    #[inline]
    fn from(color: Color32) -> Self {
        Self::from(Rgba::from(color))
    }
}
