use crate::conversions::*;
use crate::text_fields::numeric_field::*;
use crate::ui_extras::copy_icon::*;

use egui::*;

fn parse_oklab(text: &str) -> Option<Oklab> {
    let text = text
        .trim()
        .trim_matches(|c| matches!(c, '(' | ')' | '[' | ']' | '{' | '}'));

    let parts: Vec<_> = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() != 3 {
        return None;
    }

    let l = parts[0].parse::<f32>().ok()?.clamp(0.0, 1.0);
    let a = parts[1].parse::<f32>().ok()?;
    let b = parts[2].parse::<f32>().ok()?;

    Some(Oklab { l, a, b })
}

pub struct OklabEdit<'a> {
    pub oklab: &'a mut Oklab,
}

impl<'a> OklabEdit<'a> {
    pub fn new(oklab: &'a mut Oklab) -> Self {
        Self { oklab }
    }

    pub fn ui(&mut self, ui: &mut Ui, width: f32) -> bool {
        let mut changed = false;

        ui.set_width(width);

        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.label("Oklab:");

                const RIGHT_GROUP_WIDTH: f32 = 200.0;

                let spacer = (ui.available_width() - RIGHT_GROUP_WIDTH).max(0.0);
                ui.add_space(spacer);

                changed |= self.edit_component(ui, "L", 0);
                changed |= self.edit_component(ui, "A", 1);
                changed |= self.edit_component(ui, "B", 2);

                if copy_icon(ui, vec2(18.0, 18.0))
                    .on_hover_text("Click to copy Oklab values")
                    .clicked()
                {
                    ui.ctx().copy_text(format!(
                        "{:.3}, {:.3}, {:.3}",
                        self.oklab.l, self.oklab.a, self.oklab.b
                    ));
                }
            });
        });

        changed
    }

    fn edit_component(&mut self, ui: &mut Ui, label: &str, index: usize) -> bool {
        let id = ui.id().with(label);

        let component = match index {
            0 => &mut self.oklab.l,
            1 => &mut self.oklab.a,
            2 => &mut self.oklab.b,
            _ => unreachable!("Oklab does not have a 4th component"),
        };

        let (_, edit) = numeric_field(ui, id, label, format!("{:.3}", component));

        match edit {
            NumericEdit::None | NumericEdit::Cancel => false,

            NumericEdit::Commit(text) => {
                // Allow pasting "L, a, b"
                if let Some(oklab) = parse_oklab(&text) {
                    *self.oklab = oklab;
                    return true;
                }

                if let Ok(v) = text.trim().parse::<f32>() {
                    *component = if index == 0 { v.clamp(0.0, 1.0) } else { v };

                    return true;
                }

                false
            }
        }
    }
}
