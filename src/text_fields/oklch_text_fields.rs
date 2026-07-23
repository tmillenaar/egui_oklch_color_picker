use crate::conversions::*;
use crate::oklch::*;
use crate::text_fields::editable_text_field::*;
use crate::ui_extras::copy_icon::*;

use egui::*;

fn parse_oklch(text: &str) -> Option<Oklch> {
    let text = text
        .trim()
        .trim_matches(|c| matches!(c, '(' | ')' | '[' | ']' | '{' | '}'));

    let parts: Vec<_> = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_end_matches(|c| matches!(c, '°')))
        .collect();

    if parts.len() != 3 {
        return None;
    }

    let l = parts[0].parse::<f32>().ok()?.clamp(0.0, 1.0);
    let c = parts[1].parse::<f32>().ok()?.max(0.0);
    let h = parts[2]
        .parse::<f32>()
        .ok()?
        .to_radians()
        .rem_euclid(std::f32::consts::TAU);

    Some(Oklch {
        l,
        c,
        h,
        alpha: 1.0,
    })
}

pub struct OklchEdit<'a> {
    pub oklch: &'a mut Oklch,
}

impl<'a> OklchEdit<'a> {
    pub fn new(oklch: &'a mut Oklch) -> Self {
        Self { oklch }
    }

    pub fn ui(&mut self, ui: &mut Ui, width: f32) -> bool {
        let mut changed = false;

        ui.set_width(width);

        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.label("Oklch:");

                const RIGHT_GROUP_WIDTH: f32 = 200.0;

                let spacer = (ui.available_width() - RIGHT_GROUP_WIDTH).max(0.0);
                ui.add_space(spacer);

                changed |= self.edit_component(ui, "L", 0);
                changed |= self.edit_component(ui, "C", 1);
                changed |= self.edit_component(ui, "H", 2);

                if copy_icon(ui, vec2(18.0, 18.0))
                    .on_hover_text("Click to copy Oklch values")
                    .clicked()
                {
                    ui.ctx().copy_text(format!(
                        "{:.4}, {:.4}, {:.1}",
                        self.oklch.l,
                        self.oklch.c,
                        self.oklch.h.to_degrees(),
                    ));
                }
            });
        });

        changed
    }

    fn edit_component(&mut self, ui: &mut Ui, label: &str, index: usize) -> bool {
        let id = ui.id().with(("Oklch", label));

        let text = match index {
            0 => format!("{:.3}", self.oklch.l.clone()),
            1 => format!("{:.3}", self.oklch.c.clone()),
            2 => format!("{:.1}", self.oklch.h.clone().to_degrees()),
            _ => unreachable!(),
        };

        let component = match index {
            0 => &mut self.oklch.l,
            1 => &mut self.oklch.c,
            2 => &mut self.oklch.h,
            _ => unreachable!("Oklch does not have a 4th component, alpha is treated separately"),
        };

        let (_, edit) = editable_text_field(ui, id, label, text, 38.);

        match edit {
            NumericEdit::None | NumericEdit::Cancel => false,

            NumericEdit::Commit(text) => {
                // Allow pasting "L, C, H"
                if let Some(oklch) = parse_oklch(&text) {
                    *self.oklch = oklch;
                    return true;
                }

                if let Ok(v) = text
                    .trim()
                    .trim_end_matches(|c| matches!(c, '°'))
                    .parse::<f32>()
                {
                    *component = match index {
                        0 => v.clamp(0.0, 1.0),
                        1 => v.max(0.0),
                        2 => v.to_radians().rem_euclid(std::f32::consts::TAU),
                        _ => unreachable!(),
                    };

                    return true;
                }

                false
            }
        }
    }
}
