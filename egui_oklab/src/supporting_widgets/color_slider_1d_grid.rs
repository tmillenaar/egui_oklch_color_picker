use egui::epaint::RectShape;
use egui::*;

use crate::oklch::*;
use crate::supporting_widgets::background_checkers::*;

fn contrast_color(color: impl Into<Oklch>) -> Color32 {
    if color.into().l < 0.45 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

pub fn color_slider_1d_grid(
    ui: &mut Ui,
    slider_value: &mut f32,
    slider_value_nearest_color: f32,
    nearest_color: Color32,
    color_at: impl Fn(f32) -> Oklch,
    background_color_at: Option<impl Fn(f32) -> Color32>,
    axis_name: &str,
    ui_identifier: impl std::hash::Hash,
) -> Response {
    let desired_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    let mut changed = false;
    if let Some(mpos) = response.interact_pointer_pos() {
        changed = true;
        *slider_value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let tex_id = ui
            .id()
            .with(("color_slider_tex_1d", axis_name, ui_identifier));

        let mut tex = ui
            .data(|d| d.get_temp::<Option<egui::TextureHandle>>(tex_id))
            .unwrap_or(None);

        let nx = rect.width().max(1.0).round() as usize;
        let ny = rect.height().max(1.0).round() as usize;

        // TODO: rebuild when needed. Also dependent on change in l,c,h values. For now just always rebuild
        // let rebuild = changed || tex.is_none() || tex.as_ref().unwrap().size() != [nx, ny] || slider_value != prev_slider_value;
        let rebuild = true;

        background_checkers(ui.painter(), rect, 4);

        if rebuild {
            let mut pixels = Vec::with_capacity(nx * ny * 4);

            // Row-major order as expected by ColorImage
            for _y in 0..ny {
                for x in 0..nx {
                    let x_normalized = if nx > 1 {
                        x as f32 / (nx as f32 - 1.0)
                    } else {
                        0.0
                    };

                    let oklch = color_at(x_normalized);
                    if oklch.is_valid_srgb() {
                        let color = Color32::from(oklch);
                        pixels.push(color.r());
                        pixels.push(color.g());
                        pixels.push(color.b());
                        pixels.push(color.a());
                    } else {
                        match background_color_at {
                            None => {
                                pixels.push(0);
                                pixels.push(0);
                                pixels.push(0);
                                pixels.push(0);
                            }
                            Some(ref bg_color_at) => {
                                let color = bg_color_at(x_normalized);
                                pixels.push(color.r());
                                pixels.push(color.g());
                                pixels.push(color.b());
                                pixels.push(color.a());
                            }
                        }
                    };
                }
            }

            let image = egui::ColorImage::from_rgba_premultiplied([nx, ny], &pixels);

            match tex.as_mut() {
                Some(existing) => {
                    existing.set(image, egui::TextureOptions::LINEAR);
                }
                None => {
                    tex = Some(ui.ctx().load_texture(
                        "color_slider_1d",
                        image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }

            ui.data_mut(|d| {
                d.insert_temp(tex_id, tex.clone());
            });
        }

        // Actually draw the texture
        if let Some(tex) = &tex {
            ui.painter().image(
                tex.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        ui.painter()
            .rect_stroke(rect, 0.0, visuals.bg_stroke, StrokeKind::Inside);

        // Show current slider location with marker

        // If marker is not at valid color, show where the projected color would be on the slider
        let x = lerp(rect.left()..=rect.right(), slider_value_nearest_color);
        let oklch = color_at(*slider_value);
        if !oklch.is_valid_srgb() {
            ui.painter().add(Shape::Rect(RectShape::new(
                Rect::from_center_size(
                    Pos2 {
                        x,
                        y: rect.center().y,
                    },
                    Vec2 {
                        x: 6.,
                        y: rect.height(),
                    },
                ),
                0,
                nearest_color,
                Stroke::new(visuals.fg_stroke.width, contrast_color(nearest_color)),
                StrokeKind::Outside,
            )));
        }
        // Show current location with white marker
        let r = rect.height() / 3.0;
        let x = lerp(rect.left()..=rect.right(), *slider_value);
        ui.painter().add(Shape::convex_polygon(
            vec![
                pos2(x + r / 2., rect.top() - 2.),
                pos2(x, rect.top() + r - 2.),
                pos2(x - r / 2., rect.top() - 2.),
            ],
            Color32::WHITE,
            Stroke::new(visuals.fg_stroke.width, Color32::BLACK),
        ));
        ui.painter().add(Shape::convex_polygon(
            vec![
                pos2(x + r / 2., rect.bottom() + 2.),
                pos2(x, rect.bottom() - r + 2.),
                pos2(x - r / 2., rect.bottom() + 2.),
            ],
            Color32::WHITE,
            Stroke::new(visuals.fg_stroke.width, Color32::BLACK),
        ));
    }

    response
}
