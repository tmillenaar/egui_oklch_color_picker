//! Color picker widgets.

// use egui::util::fixed_cache::FixedCache;
mod color_projection;
mod conversions;
mod coordinate_transforms;
mod oklaba;
mod oklcha;
mod picker_2d;
mod text_fields;
mod ui_extras;

use crate::color_projection::*;
use crate::conversions::*;
use crate::coordinate_transforms::*;
use crate::oklaba::*;
use crate::oklcha::*;
use crate::picker_2d::*;
use crate::text_fields::hex_text_field::*;
use crate::text_fields::hsv_text_fields::*;
use crate::text_fields::oklab_text_fields::*;
use crate::text_fields::oklch_text_fields::*;
use crate::text_fields::rgb_text_fields::*;

use std::f32;
use std::f32::consts::TAU;

// use egui::color_picker::{Alpha, show_color, show_color_at};
use egui::epaint::RectShape;
use egui::*;
use epaint::{
    Mesh, Rect, Shape, Stroke, StrokeKind, Vec2,
    ecolor::{Color32, Rgba},
    pos2, vec2,
};

fn contrast_color(color: impl Into<Rgba>) -> Color32 {
    if color.into().intensity() < 0.5 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

/// In rust, % is the remainder, not the modulus
fn modulus(val: f32, modulus: f32) -> f32 {
    ((val % modulus) + modulus) % modulus
}

pub fn background_checkers(painter: &Painter, rect: Rect, ny: u32) {
    let rect = rect.shrink(0.5); // Small hack to avoid the checkers from peeking through the sides

    let dark_color = Color32::from_gray(80);
    let bright_color = Color32::from_gray(180);

    // Adjust rect size and checker size to match
    let target_checker_size = rect.height() / ny as f32;
    let checker_size = target_checker_size.floor();
    let overshoot_y = modulus(rect.height(), checker_size);
    let overshoot_x = modulus(rect.width(), checker_size);
    let rect = rect.shrink2(Vec2::new(overshoot_x / 2., overshoot_y / 2.));

    if !rect.is_positive() {
        return;
    }

    let nx = (rect.width() / checker_size).round() as u32;

    let mut mesh = Mesh::default();
    mesh.add_colored_rect(rect, dark_color);

    let mut top = true;
    for i_y in 0..ny {
        for mut i_x in (0..nx).step_by(2) {
            // Offset every other cell
            if modulus(i_x as f32 + i_y as f32, 2.) == 0. {
                i_x += 1;
                if i_x >= nx {
                    break;
                }
            }
            let x = lerp(rect.left()..=rect.right(), i_x as f32 / nx as f32);
            let cell_top = rect.top() + i_y as f32 * checker_size;
            let cell_rect = Rect::from_min_size(pos2(x, cell_top), Vec2::splat(checker_size));
            mesh.add_colored_rect(cell_rect, bright_color);
            top = !top;
        }
    }
    painter.add(Shape::mesh(mesh));
}

fn color_button(ui: &mut Ui, color: Color32, open: bool) -> Response {
    let size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::new(WidgetType::ColorButton));

    if ui.is_rect_visible(rect) {
        let visuals = if open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&response)
        };
        let rect = rect.expand(visuals.expansion);

        let stroke_width = 1.0;
        show_color_at(ui.painter(), color, rect.shrink(stroke_width), 3);

        let corner_radius = visuals.corner_radius.at_most(2); // Can't do more rounding because the background grid doesn't do any rounding
        ui.painter().rect_stroke(
            rect,
            corner_radius,
            (stroke_width, visuals.bg_fill), // Using fill for stroke is intentional, because default style has no border
            StrokeKind::Inside,
        );
    }

    response
}

#[derive(Debug, Clone, PartialEq)]
struct OklchUiState {
    // normalized 0..1
    l: f32,
    c: f32,
    h: f32,
    a: f32,
}

pub fn color_slider_1d_grid(
    ui: &mut Ui,
    slider_value: &mut f32,
    slider_value_nearest_color: f32,
    nearest_color: Color32,
    color_at: impl Fn(f32) -> ColorSample,
    background_color_at: Option<impl Fn(f32) -> Color32>,
    axis_name: &str,
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

        let tex_id = ui.id().with(("color_slider_tex_1d", axis_name));

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

                    let color_sample = color_at(x_normalized);
                    if color_sample.valid {
                        pixels.push(color_sample.color.r());
                        pixels.push(color_sample.color.g());
                        pixels.push(color_sample.color.b());
                        pixels.push(color_sample.color.a());
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

            let image = egui::ColorImage::from_rgba_unmultiplied([nx, ny], &pixels);

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
        let color_sample = color_at(*slider_value);
        if !color_sample.valid {
            // let (_, nearest_color) = color_at(slider_value_nearest_color);
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

pub fn color_slider_2d<T, P>(
    ui: &mut Ui,
    coord: &mut T::Coord,
    projection: P,
    ui_identifiable_label: &str,
) -> (Response, bool)
where
    T: CoordinateTransform + Send + Sync + 'static,
    T::Coord: Clone + Copy + PartialEq,
    P: ColorProjection<T::Coord> + Clone + PartialEq + Send + Sync + 'static,
{
    let mut color_changed = false;

    let desired_size = Vec2::splat(ui.spacing().slider_width + 34.);

    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    let picker_id = ui.id().with(ui_identifiable_label);

    let mut picker = ui
        .data(|d| d.get_temp::<CachedPicker<T, P>>(picker_id))
        .unwrap_or_else(|| CachedPicker::new(ui, rect.size(), projection.clone()));

    // Resize if necessary.
    picker.resize(rect.size());

    // Update projection if it changed.
    picker.set_projection(projection);

    // Update viewport (zoom/pan).
    if picker.update_viewport(&response, rect) {
        picker.update_coordinates();
        picker.update_texture();
    }

    let mut color_is_valid = false;
    let mut viewport_contains_picked_color = false;

    if ui.is_rect_visible(rect) {
        let painter = ui.painter().with_clip_rect(rect);

        T::draw_axes_on_background(
            &painter,
            |world| picker.world_to_pixel(world),
            rect,
            picker.viewport,
        );

        painter.image(
            picker.texture.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        let visuals = ui.style().interact(&response);

        painter.rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

        // Obtain color sample
        let sample = picker.projection.color_at(*coord);
        color_is_valid = sample.valid;

        // Draw marker
        let (indicator_color, marker_angle) = if sample.valid {
            (Color32::WHITE, 45.)
        } else {
            (Color32::WHITE, 0.)
        };

        let marker_pos = picker.coord_to_pixel(*coord);
        viewport_contains_picked_color = marker_pos.x >= 0.0
            && marker_pos.x <= rect.width()
            && marker_pos.y >= 0.0
            && marker_pos.y <= rect.height();

        draw_inward_triangles(
            &painter,
            marker_pos + rect.min.to_vec2(),
            4.0,
            4.0,
            4,
            marker_angle,
            indicator_color,
            Stroke {
                width: 0.5,
                color: Color32::BLACK,
            },
        );
        if sample.valid {
            picker.previous_fallback = Some(*coord);
        } else {
            let fallback_coord = if viewport_contains_picked_color {
                let fallback = picker.nearest_valid_border_coord(*coord);
                picker.previous_fallback = fallback;
                fallback
            } else {
                picker.previous_fallback
            };

            if let Some(coord) = fallback_coord {
                let pos = picker.coord_to_pixel(coord);

                draw_inward_triangles(
                    &painter,
                    pos + rect.min.to_vec2(),
                    3.0,
                    3.0,
                    4,
                    45.0,
                    Color32::WHITE,
                    Stroke {
                        width: 0.5,
                        color: Color32::BLACK,
                    },
                );
            }
        }
        // Draw arrow pointing to marker if outside of viewport
        if let Some((pos, angle)) = picker.offscreen_indicator(*coord) {
            draw_triangle(
                &painter,
                pos + rect.min.to_vec2(),
                angle,
                8.0,
                Color32::WHITE,
                Stroke {
                    width: 0.5,
                    color: Color32::BLACK,
                },
            );
        }
    }

    if response.is_pointer_button_down_on() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::None);
    }

    // Mouse interaction.
    if response
        .ctx
        .input(|i| i.pointer.button_down(PointerButton::Primary))
    {
        if let Some(mouse) = response.interact_pointer_pos() {
            let local = Pos2 {
                x: mouse.x - rect.left(),
                y: mouse.y - rect.top(),
            };

            *coord = picker.pixel_to_coord(local);
            color_changed = true;
        }
    }

    ui.data_mut(|d| {
        d.insert_temp(picker_id, picker);
    });

    // (response, viewport_contains_picked_color || color_is_valid)
    (response, color_changed)
}

pub fn draw_inward_triangles(
    painter: &Painter,
    center: Pos2,
    radius: f32, // distance from center to inner tip
    size: f32,   // side length of equilateral triangle
    count: usize,
    offset_deg: f32,
    fill: Color32,
    stroke: Stroke,
) {
    if count == 0 {
        return;
    }

    let height = size * (3.0_f32).sqrt() * 0.5;

    let base_angle = std::f32::consts::TAU * 0.75; // bottom
    let offset = offset_deg.to_radians();
    let step = std::f32::consts::TAU / count as f32;

    for i in 0..count {
        let angle = base_angle + offset + step * i as f32;

        let outward = Vec2::new(angle.cos(), angle.sin());

        // Inner tip sits exactly radius units from center.
        let tip = center + outward * radius;

        let base_center = tip + outward * height;

        let perp = Vec2::new(-outward.y, outward.x);

        let left = base_center + perp * (size * 0.5);
        let right = base_center - perp * (size * 0.5);

        painter.add(Shape::convex_polygon(vec![tip, left, right], fill, stroke));
    }
}

pub fn draw_triangle(
    painter: &Painter,
    tip: Pos2,
    angle: f32, // radians, 0 = +X
    size: f32,  // side length of equilateral triangle
    fill: Color32,
    stroke: Stroke,
) {
    let height = size * (3.0_f32).sqrt() * 0.5;

    // Direction the triangle points.
    let dir = Vec2::new(angle.cos(), angle.sin());

    // Perpendicular to the direction.
    let perp = Vec2::new(-dir.y, dir.x);

    // Base is behind the tip.
    let base_center = tip - dir * height;

    let left = base_center + perp * (size * 0.5);
    let right = base_center - perp * (size * 0.5);

    painter.add(Shape::convex_polygon(vec![tip, left, right], fill, stroke));
}

#[derive(Debug, Clone)]
enum OKLabPlotType {
    StableLightness,
    StableChroma,
    StableHue,
}

pub fn color_edit_button_oklcha(ui: &mut Ui, color: &mut Color32, widget_width: f32) -> Response {
    let popup_id = ui.auto_id_with("popup");
    let open = Popup::is_id_open(ui.ctx(), popup_id);
    let mut button_response = color_button(ui, *color, open);
    if ui.style().explanation_tooltips {
        button_response = button_response.on_hover_text("Click to edit color");
    }

    Popup::menu(&button_response)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = widget_width;
            if color_picker_oklcha_2d(ui, color, widget_width) {
                button_response.mark_changed();
            }
        });

    button_response
}

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
        let mut oklab = Oklaba::from(color.to_opaque());
        if OklabEdit::new(&mut oklab).ui(ui, widget_width + 34.) {
            let color_sample = oklab_to_color32(oklab);
            let rgba = if color_sample.valid {
                color_sample.color.to_srgba_unmultiplied()
            } else {
                // TODO: fallback? Or implement an oklab_to_color32_with_chroma_fallback?
                color_sample.color.to_srgba_unmultiplied()
            };
            new_color = Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], alpha);
            changed = true;
        }
    });

    // Oklch text fields
    ui.horizontal(|ui| {
        let mut oklch = color32_to_oklch(color.to_opaque());

        if OklchEdit::new(&mut oklch).ui(ui, widget_width + 34.) {
            let color_sample = oklch_to_color32(oklch);
            let rgba = if color_sample.valid {
                color_sample.color.to_srgba_unmultiplied()
            } else {
                // TODO: fallback? Or implement an oklab_to_color32_with_chroma_fallback?
                color_sample.color.to_srgba_unmultiplied()
            };
            new_color = Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], color.a());
            changed = true;
        }
    });

    (changed, new_color)
}

pub fn oklch_sliders(ui: &mut Ui, fallback_color: Color32) -> (bool, OklchUiState) {
    // Recover state from cache
    let state_id = ui.id().with("oklch_ui_state");
    let lch = color32_to_oklch(fallback_color);
    let mut state: OklchUiState =
        ui.data(|d| d.get_temp::<OklchUiState>(state_id))
            .unwrap_or(OklchUiState {
                l: lch.l,
                c: lch.c / 0.325,
                h: lch.h / std::f32::consts::TAU,
                a: 1.0,
            });
    let original_state = state.clone();
    let l = state.l;
    let c = state.c * 0.325;
    let h = state.h * TAU;

    // Lightness slider
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.l,
            lch.l,
            fallback_color,
            |l_mut| oklch_to_color32(Oklch { l: l_mut, c, h }),
            None::<fn(f32) -> Color32>,
            &"lightness",
        )
        .on_hover_text("Lightness");
        DragValue::new(&mut state.l)
            .prefix("L: ")
            .speed(0.003)
            .range(0.0..=1.0)
            .custom_formatter(|n, _| format!("{n:.03}"))
            .ui(ui)
            .on_hover_text("Lightness");
    });

    // Chroma slider
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.c,
            lch.c / 0.325,
            fallback_color,
            |c_mut| {
                oklch_to_color32(Oklch {
                    l,
                    c: c_mut * 0.325,
                    h,
                })
            },
            None::<fn(f32) -> Color32>,
            &"chroma",
        )
        .on_hover_text("Chroma");
        let mut chroma_val = state.c * 0.325;
        DragValue::new(&mut chroma_val)
            .prefix("C: ")
            .speed(0.001)
            .range(0.0..=0.325)
            .custom_formatter(|n, _| format!("{n:.03}"))
            .ui(ui)
            .on_hover_text("Chroma");
        state.c = chroma_val / 0.325;
    });

    // Hue slider
    let determine_hue_slider_colors = |h_mut| {
        // Define lightness and chroma that exist for all hues
        let bg_color_sample = oklch_to_color32(Oklch {
            l: 0.750,
            c: 0.127,
            h: h_mut * std::f32::consts::TAU,
        });
        let bg_color = bg_color_sample.color.blend(Color32::from_black_alpha(40));
        let color = Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 210);
        color
    };
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.h,
            lch.h / std::f32::consts::TAU,
            fallback_color,
            |h_mut| {
                oklch_to_color32(Oklch {
                    l: l,
                    c: c,
                    h: h_mut * std::f32::consts::TAU,
                })
            },
            Some(determine_hue_slider_colors),
            &"hue",
        )
        .on_hover_text("Hue");
        let mut hue_val = state.h * 360.;
        DragValue::new(&mut hue_val)
            .prefix("H: ")
            .speed(1.0)
            .range(0.0..=360.)
            .custom_formatter(|n, _| format!("{n:.1}"))
            .ui(ui)
            .on_hover_text("Hue");
        state.h = hue_val / 360.;
    });

    let with_alpha = true;
    if with_alpha {
        ui.horizontal(|ui| {
            color_slider_1d_grid(
                ui,
                &mut state.a,
                lch.h / std::f32::consts::TAU,
                fallback_color,
                |alpha| {
                    // let mut rgba = fallback_color.to_srgba_unmultiplied();
                    let mut rgba = fallback_color.to_array();
                    ColorSample {
                        color: Color32::from_rgba_premultiplied(
                            rgba[0],
                            rgba[1],
                            rgba[2],
                            (alpha * 255.) as u8,
                        ),
                        valid: true,
                    }
                },
                None::<fn(f32) -> Color32>,
                &"alpha",
            )
            .on_hover_text("Transparency (Alpha)");
            DragValue::new(&mut state.a)
                .prefix("A: ")
                .speed(1.0)
                .range(0.0..=1.0)
                .custom_formatter(|n, _| format!("{n:.3}"))
                .ui(ui)
                .on_hover_text("Transparency");
        });
    }

    let state_changed = state != original_state;

    ui.data_mut(|d| {
        d.insert_temp(state_id, state.clone());
    });

    (state_changed, state)
}

pub fn plot_type_selector(ui: &mut Ui) -> (bool, OKLabPlotType) {
    let mut tab_changed = false;

    let mut tab = ui.data(|d| {
        d.get_temp::<OKLabPlotType>(ui.id())
            .unwrap_or(OKLabPlotType::StableHue)
    });

    ui.horizontal(|ui| {
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableHue), "Stable Hue")
            .clicked()
        {
            tab_changed = true;
            tab = OKLabPlotType::StableHue;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableChroma), "Stable Chroma")
            .clicked()
        {
            tab_changed = true;
            tab = OKLabPlotType::StableChroma;
        }
        if ui
            .selectable_label(
                matches!(tab, OKLabPlotType::StableLightness),
                "Stable Lightness",
            )
            .clicked()
        {
            tab_changed = true;
            tab = OKLabPlotType::StableLightness;
        }
    });

    ui.data_mut(|d| {
        d.insert_temp(ui.id(), tab.clone());
    });

    (tab_changed, tab)
}

pub fn color_picker_panel_oklch_2d(ui: &mut Ui, plot_type: OKLabPlotType) -> (bool, Color32) {
    // Recover state from cache
    let state_id = ui.id().with("oklch_ui_state");
    let mut state: OklchUiState =
        ui.data(|d| d.get_temp::<OklchUiState>(state_id))
            .unwrap_or(OklchUiState {
                l: 0.5,
                c: 0.3,
                h: 0.0,
                a: 1.0,
            });

    let state_original = state.clone();

    let (mut new_color, color_changed) = match plot_type {
        OKLabPlotType::StableHue => {
            let projection = StableHueProjection {
                hue: state.h * std::f32::consts::TAU,
            };

            let mut coord = CartesianCoord {
                x: state.c,
                y: state.l,
            };

            let (_response, update_color) = color_slider_2d::<
                CartesianTransform,
                StableHueProjection,
            >(
                ui, &mut coord, projection, "StableHueColorPicker"
            );

            state.c = coord.x.min(1.0);
            state.l = coord.y.min(1.0);

            let picker_id = ui.id().with("StableHueColorPicker");

            let picker = ui
                .data(|d| d.get_temp::<StableHuePicker>(picker_id))
                .unwrap();

            (picker.nearest_valid_color(coord), update_color)
        }

        OKLabPlotType::StableChroma => {
            let projection = StableChromaProjection { chroma: state.c };

            let mut coord = PolarCoord {
                angle: state.h * std::f32::consts::TAU,
                radius: 1.0 - state.l,
            };

            let (_response, update_color) = color_slider_2d::<PolarTransform, StableChromaProjection>(
                ui,
                &mut coord,
                projection,
                "StableChromaColorPicker",
            );

            state.h = coord.angle.rem_euclid(360.0) / std::f32::consts::TAU;
            state.l = 1.0 - coord.radius.min(1.0);

            let picker_id = ui.id().with("StableChromaColorPicker");

            let picker = ui
                .data(|d| d.get_temp::<StableChromaPicker>(picker_id))
                .unwrap();

            (picker.nearest_valid_color(coord), update_color)
        }

        OKLabPlotType::StableLightness => {
            let projection = StableLightnessProjection { lightness: state.l };

            let mut coord = PolarCoord {
                angle: state.h * std::f32::consts::TAU,
                radius: state.c,
            };

            let (_response, update_color) =
                color_slider_2d::<PolarTransform, StableLightnessProjection>(
                    ui,
                    &mut coord,
                    projection,
                    "StableLightnessColorPicker",
                );

            state.h = (coord.angle / std::f32::consts::TAU).rem_euclid(360.0);
            state.c = coord.radius.min(1.0);

            let picker_id = ui.id().with("StableLightnessColorPicker");

            let picker = ui
                .data(|d| d.get_temp::<StableLightnessPicker>(picker_id))
                .unwrap();

            (picker.nearest_valid_color(coord), update_color)
        }
    };

    if color_changed {
        ui.data_mut(|d| {
            d.insert_temp(state_id, state);
        });
    } else {
        // Note: We put back the original state in case the user did not click the
        //       viewport to prevent rounding of state to the nearest pixel in case
        //       state was set before by a different widget.
        ui.data_mut(|d| {
            d.insert_temp(state_id, state_original);
        });
    }

    (color_changed, new_color)
}

pub fn color_picker_oklcha_2d(ui: &mut Ui, color: &mut Color32, widget_width: f32) -> bool {
    let color_opague = color.to_opaque();
    let (tab_changed, plot_type) = plot_type_selector(ui);

    let current_color_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    show_color(ui, *color, current_color_size).on_hover_text("Selected color");

    let (color_changed_in_2d_panel, new_color_opague) = color_picker_panel_oklch_2d(ui, plot_type);

    let (color_changed_by_sliders, new_state) = oklch_sliders(ui, new_color_opague);

    if color_changed_in_2d_panel | tab_changed {
        let rgba = new_color_opague.to_array();
        *color =
            Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], (new_state.a * 255.) as u8);
    }
    if color_changed_by_sliders {
        let color_sample = oklch_to_color32(Oklch {
            l: new_state.l,
            c: new_state.c * 0.325,
            h: new_state.h * std::f32::consts::TAU,
        });
        if color_sample.valid {
            *color = Color32::from_rgba_unmultiplied(
                color_sample.color.r(),
                color_sample.color.g(),
                color_sample.color.b(),
                (new_state.a * 255.) as u8,
            );
        } else {
            // Fallback color using 2d panel
            let rgba = new_color_opague.to_array();
            *color = Color32::from_rgba_unmultiplied(
                rgba[0],
                rgba[1],
                rgba[2],
                (new_state.a * 255.) as u8,
            );
        }
    }

    let (color_changed_by_text_field, new_color_from_text) =
        color_text_fields(ui, new_color_opague, widget_width);
    if color_changed_by_text_field {
        // Also update state indicators when color changed from direct text inserts
        let state_id = ui.id().with("oklch_ui_state");
        let lch = color32_to_oklch(new_color_from_text);
        let state = OklchUiState {
            l: lch.l,
            c: lch.c / 0.325,
            h: lch.h / std::f32::consts::TAU,
            a: new_state.a,
        };
        let rgba = new_color_from_text.to_array();
        *color =
            Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], (new_state.a * 255.) as u8);
        ui.data_mut(|d| {
            d.insert_temp(state_id, state);
        });
    };

    let color_changed = color_changed_in_2d_panel
        | color_changed_by_sliders
        | color_changed_by_text_field
        | tab_changed;

    color_changed
}


/// Show a color with background checkers to demonstrate transparency (if any).
pub fn show_color(ui: &mut Ui, color: impl Into<Color32>, desired_size: Vec2) -> Response {
    show_color32(ui, color.into(), desired_size)
}

fn show_color32(ui: &mut Ui, color: Color32, desired_size: Vec2) -> Response {
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
    if ui.is_rect_visible(rect) {
        show_color_at(ui.painter(), color, rect, 3);
    }
    response
}

/// Show a color with background checkers to demonstrate transparency (if any).
pub fn show_color_at(painter: &Painter, color: Color32, rect: Rect, nr_cells_y: u32) {
    if color.is_opaque() {
        painter.rect_filled(rect, 0.0, color);
    } else {
        // Transparent: how both the transparent and opaque versions of the color
        background_checkers(painter, rect, nr_cells_y);

        if color == Color32::TRANSPARENT {
            // There is no opaque version, so just show the background checkers
        } else {
            let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
            let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
            painter.rect_filled(left, 0.0, color);
            painter.rect_filled(right, 0.0, color.to_opaque());
        }
    }
}
