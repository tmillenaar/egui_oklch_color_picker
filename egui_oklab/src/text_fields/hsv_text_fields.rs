use crate::conversions::*;
use crate::oklch::Oklch;
use crate::supporting_widgets::copy_icon::*;
use crate::text_fields::editable_text_field::*;

use egui::*;

fn parse_hsv(text: &str) -> Option<Hsv> {
    let text = text
        .trim()
        .trim_matches(|c| matches!(c, '(' | ')' | '[' | ']' | '{' | '}'));

    let parts: Vec<_> = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_end_matches(|c| matches!(c, '%' | '°')))
        .collect();

    if parts.len() != 3 {
        return None;
    }

    let mut hsv = [0.0; 3];

    for (i, (dst, part)) in hsv.iter_mut().zip(parts.clone()).enumerate() {
        let value = part.parse::<f32>().ok()?;

        *dst = if i == 0 {
            value.rem_euclid(360.0)
        } else {
            value.clamp(0.0, 100.0)
        };
    }

    Some(Hsv {
        h: hsv[0],
        s: hsv[1],
        v: hsv[2],
    })
}
pub struct HsvEdit<'a> {
    pub hsv: &'a mut Hsv,
}

impl<'a> HsvEdit<'a> {
    pub fn new(hsv: &'a mut Hsv) -> Self {
        Self { hsv }
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
            ui.horizontal(|ui| {
                ui.label("HSV:");

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

                changed |= self.edit_component(ui, "H", 0, ui_identifier);
                changed |= self.edit_component(ui, "S", 1, ui_identifier);
                changed |= self.edit_component(ui, "V", 2, ui_identifier);

                if copy_icon(ui, vec2(18.0, 18.0))
                    .on_hover_text("Click to copy HSV values")
                    .clicked()
                {
                    ui.ctx().copy_text(format!(
                        "{:.1}, {:.1}, {:.1}",
                        self.hsv.h, self.hsv.s, self.hsv.v
                    ));
                }
            });
        });
        changed
    }

    fn edit_component(
        &mut self,
        ui: &mut Ui,
        label: &str,
        index: usize,
        ui_identifier: *const Oklch, // use pointer to color in RAM as identifier to allow for multiple color panels. They may share state if they refer to the same color
    ) -> bool {
        let id = ui.id().with(("HSV", label, ui_identifier));

        let mut component = match index {
            0 => &mut self.hsv.h,
            1 => &mut self.hsv.s,
            2 => &mut self.hsv.v,
            _ => unreachable!("HSV color does not have a 4rth component"),
        };

        let (_, edit) = editable_text_field(ui, id, label, format!("{:.1}", component), 38.);

        match edit {
            NumericEdit::None | NumericEdit::Cancel => false,

            NumericEdit::Commit(text) => {
                if let Some(hsv) = parse_hsv(&text) {
                    *self.hsv = hsv;
                    return true;
                }

                if let Ok(v) = text
                    .trim()
                    .trim_end_matches(|c| matches!(c, '%' | '°'))
                    .parse::<f32>()
                {
                    *component = if index == 0 {
                        v.rem_euclid(360.0)
                    } else {
                        v.clamp(0.0, 100.0)
                    };

                    return true;
                }

                false
            }
        }
    }
}
