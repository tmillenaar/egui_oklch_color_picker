use crate::text_fields::numeric_field::*;
use crate::ui_extras::copy_icon::*;
use egui::*;

fn parse_rgb(text: &str) -> Option<[u8; 3]> {
    // Remove common surrounding delimiters.
    let text = text
        .trim()
        .trim_matches(|c| matches!(c, '(' | ')' | '[' | ']' | '{' | '}'));

    // Split on commas and/or whitespace.
    let parts: Vec<_> = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() != 3 {
        return None;
    }

    let floats: Vec<f32> = parts
        .iter()
        .map(|s| s.parse::<f32>())
        .collect::<Result<_, _>>()
        .ok()?;

    let normalized = floats.iter().all(|v| (0.0..=1.0).contains(v));

    let mut rgb = [0; 3];

    for (dst, value) in rgb.iter_mut().zip(floats) {
        let value = if normalized { value * 255.0 } else { value };

        *dst = value.round().clamp(0.0, 255.0) as u8;
    }

    Some(rgb)
}

pub struct RgbEdit<'a> {
    pub rgb: &'a mut [u8; 3],
}

impl<'a> RgbEdit<'a> {
    pub fn new(rgb: &'a mut [u8; 3]) -> Self {
        Self { rgb }
    }
}

impl<'a> RgbEdit<'a> {
    pub fn ui(&mut self, ui: &mut Ui, width: f32) -> bool {
        let mut changed = false;

        ui.set_width(width);

        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.label("RGB:");

                // Width of the right-hand group:
                //
                // R label + edit
                // G label + edit
                // B label + edit
                // copy button
                //
                // Adjust this once until it looks right.
                const RIGHT_GROUP_WIDTH: f32 = 200.0;

                let spacer = (ui.available_width() - RIGHT_GROUP_WIDTH).max(0.0);
                ui.add_space(spacer);

                changed |= self.edit_component(ui, "R", 0);
                changed |= self.edit_component(ui, "G", 1);
                changed |= self.edit_component(ui, "B", 2);

                if copy_icon(ui, vec2(18.0, 18.0))
                    .on_hover_text("Click to copy RGB values")
                    .clicked()
                {
                    ui.ctx()
                        .copy_text(format!("{}, {}, {}", self.rgb[0], self.rgb[1], self.rgb[2]));
                }
            });
        });

        changed
    }

    fn edit_component(&mut self, ui: &mut Ui, label: &str, index: usize) -> bool {
        let id = ui.id().with(label);

        let (_, edit) = numeric_field(ui, id, label, self.rgb[index].to_string());

        match edit {
            NumericEdit::None | NumericEdit::Cancel => false,

            NumericEdit::Commit(text) => {
                if let Some(rgb) = parse_rgb(&text) {
                    *self.rgb = rgb;
                    return true;
                }

                if let Ok(v) = text.trim().parse::<f32>() {
                    self.rgb[index] = v.round().clamp(0.0, 255.0) as u8;
                    return true;
                }

                false
            }
        }
    }
}
