use egui::*;

use crate::Oklab;

use crate::color32_to_hsv;
use crate::color32_to_oklch;
use crate::hsv_to_color32;
use crate::oklab_to_color32;
use crate::oklch_to_color32;

use crate::text_fields::hex_text_field::*;
use crate::text_fields::hsv_text_fields::*;
use crate::text_fields::oklab_text_fields::*;
use crate::text_fields::oklch_text_fields::*;
use crate::text_fields::rgb_text_fields::*;

pub fn color_text_fields(ui: &mut Ui, color: Color32, widget_width: f32) -> (bool, Color32) {
    let mut changed = false;
    let mut new_color = color.clone();

    // Hex text field
    let mut srgba = color.to_srgba_unmultiplied();
    let mut srgb = [srgba[0], srgba[1], srgba[2]];
    let alpha = srgba[3];
    ui.horizontal(|ui| {
        if HexEdit::new(&mut srgb).ui(ui, widget_width + 34.0) {
            new_color = Color32::from_rgba_unmultiplied(srgb[0], srgb[1], srgb[2], alpha);
            changed = true;
        }
    });

    // RGB text fields
    ui.horizontal(|ui| {
        if RgbEdit::new(&mut srgb).ui(ui, widget_width + 34.) {
            new_color = Color32::from_rgba_unmultiplied(srgb[0], srgb[1], srgb[2], alpha);
            changed = true;
        }
    });

    // HSV text fields
    ui.horizontal(|ui| {
        let mut hsv = color32_to_hsv(color.to_opaque());
        if HsvEdit::new(&mut hsv).ui(ui, widget_width + 34.) {
            let rgba = hsv_to_color32(hsv).to_array();
            new_color = Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], alpha);
            changed = true;
        }
    });

    // Oklab text fields
    ui.horizontal(|ui| {
        let mut oklab = Oklab::from(color.to_opaque());
        if OklabEdit::new(&mut oklab).ui(ui, widget_width + 34.) {
            let color = oklab_to_color32(oklab);
            let rgba = if oklab.is_valid_srgb() {
                color.to_srgba_unmultiplied()
            } else {
                // TODO: fallback? Or implement an oklab_to_color32_with_chroma_fallback?
                color.to_srgba_unmultiplied()
            };
            new_color = Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], alpha);
            changed = true;
        }
    });

    // Oklch text fields
    ui.horizontal(|ui| {
        let mut oklch = color32_to_oklch(color.to_opaque());

        if OklchEdit::new(&mut oklch).ui(ui, widget_width + 34.) {
            let color = oklch_to_color32(oklch);
            let rgba = if oklch.is_valid_srgb() {
                color.to_srgba_unmultiplied()
            } else {
                // TODO: fallback? Or implement an oklab_to_color32_with_chroma_fallback?
                color.to_srgba_unmultiplied()
            };
            new_color = Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], color.a());
            changed = true;
        }
    });

    (changed, new_color)
}
