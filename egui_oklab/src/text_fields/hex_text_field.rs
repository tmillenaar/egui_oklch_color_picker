use egui::*;

use crate::oklch::Oklch;
use crate::supporting_widgets::copy_icon::*;
use crate::text_fields::editable_text_field::{NumericEdit, editable_text_field};

pub struct HexEdit<'a> {
    pub rgb: &'a mut [u8; 3],
}

impl<'a> HexEdit<'a> {
    pub fn new(rgb: &'a mut [u8; 3]) -> Self {
        Self { rgb }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        width: f32,
        ui_identifier: *const Oklch, // use pointer to color in RAM as identifier to allow for multiple color panels. They may share state if they refer to the same color
    ) -> bool {
        let mut changed = false;

        ui.set_width(width);

        ui.horizontal(|ui| {
            ui.label("Hex:");

            const RIGHT_GROUP_WIDTH: f32 = 71.0;

            let spacer = (ui.available_width() - RIGHT_GROUP_WIDTH).max(0.0);
            ui.add_space(spacer);

            let id = ui.id().with(("Oklab", "Hex", ui_identifier));

            let text = format!("#{:02X}{:02X}{:02X}", self.rgb[0], self.rgb[1], self.rgb[2],);

            let (_, edit) = editable_text_field(ui, id, "", text, 56.);

            match edit {
                NumericEdit::None | NumericEdit::Cancel => {}

                NumericEdit::Commit(text) => {
                    if let Some(rgb) = parse_hex(&text) {
                        *self.rgb = rgb;
                        changed = true;
                    }
                }
            }

            if copy_icon(ui, vec2(18.0, 18.0))
                .on_hover_text("Copy hex value")
                .clicked()
            {
                ui.ctx().copy_text(format!(
                    "#{:02X}{:02X}{:02X}",
                    self.rgb[0], self.rgb[1], self.rgb[2],
                ));
            }
        });

        changed
    }
}

fn parse_hex(text: &str) -> Option<[u8; 3]> {
    let text = text.trim().trim_start_matches('#');

    match text.len() {
        6 => Some([
            u8::from_str_radix(&text[0..2], 16).ok()?,
            u8::from_str_radix(&text[2..4], 16).ok()?,
            u8::from_str_radix(&text[4..6], 16).ok()?,
        ]),

        3 => Some([
            u8::from_str_radix(&text[0..1].repeat(2), 16).ok()?,
            u8::from_str_radix(&text[1..2].repeat(2), 16).ok()?,
            u8::from_str_radix(&text[2..3].repeat(2), 16).ok()?,
        ]),

        _ => None,
    }
}
