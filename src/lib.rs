//! Color picker widgets.

// use egui::util::fixed_cache::FixedCache;
mod color_projection;
mod conversions;
mod coordinate_transforms;
mod picker_2d;

use crate::color_projection::*;
use crate::conversions::*;
use crate::coordinate_transforms::*;
use crate::picker_2d::*;

use std::f32;
use std::sync::Arc;

use egui::Order::Background;
use egui::epaint::RectShape;
use egui::{
    Context, DragValue, Id, Painter, PointerButton, Popup, PopupCloseBehavior, Pos2, Response,
    Sense, TextureHandle, Ui, Widget as _, WidgetInfo, WidgetType, epaint, lerp, remap_clamp,
};
use egui::{cache::*, viewport};
use epaint::{
    Mesh, Rect, Shape, Stroke, StrokeKind, Vec2,
    ecolor::{Color32, Hsva, HsvaGamma, Rgba},
    pos2, vec2,
};

fn contrast_color(color: impl Into<Rgba>) -> Color32 {
    if color.into().intensity() < 0.5 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

/// In rust, & is the remainder, not the modulus
fn modulus(val: f32, modulus: f32) -> f32 {
    ((val % modulus) + modulus) % modulus
}

fn background_checkers(painter: &Painter, rect: Rect, ny: u32) {
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

/// Show a color with background checkers to demonstrate transparency (if any).
fn show_srgba_unmultiplied(ui: &mut Ui, srgba: [u8; 4], desired_size: Vec2) -> Response {
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
    if ui.is_rect_visible(rect) {
        show_srgba_unmultiplied_at(ui.painter(), srgba, rect);
    }
    response
}

/// Show a color with background checkers to demonstrate transparency (if any).
fn show_srgba_unmultiplied_at(painter: &Painter, [r, g, b, a]: [u8; 4], rect: Rect) {
    if a == 255 {
        painter.rect_filled(rect, 0.0, Color32::from_rgb(r, g, b));
    } else {
        background_checkers(painter, rect, 4);
        let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
        let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
        painter.rect_filled(left, 0.0, Color32::from_rgba_unmultiplied(r, g, b, a));
        painter.rect_filled(right, 0.0, Color32::from_rgb(r, g, b));
    }
}

fn color_button(ui: &mut Ui, srgba: [u8; 4], open: bool) -> Response {
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
        show_srgba_unmultiplied_at(ui.painter(), srgba, rect.shrink(stroke_width));

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

#[derive(Debug, Clone)]
struct OklchUiState {
    // normalized 0..1
    l: f32,
    c: f32,
    h: f32,
}

fn color_slider_1d_grid(
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

    if let Some(mpos) = response.interact_pointer_pos() {
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

        // Fixme: Also dependent on change in l,c,h values, forn now just always rebuild
        // let rebuild = tex.is_none() || tex.as_ref().unwrap().size() != [nx, ny];
        let rebuild = true;

        background_checkers(ui.painter(), rect, 4);

        if rebuild {
            let mut pixels = Vec::with_capacity(nx * ny * 4);

            // Row-major order as expected by ColorImage
            for _y in 0..ny {
                for x in 0..nx {
                    let t = if nx > 1 {
                        x as f32 / (nx as f32 - 1.0)
                    } else {
                        0.0
                    };

                    let color_sample = color_at(t);
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
                                let color = bg_color_at(t);
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

/// CPU-rasterized 2D color slider using egui textures.
fn color_slider_2d_grid(
    ui: &mut Ui,
    x_value: &mut f32,
    y_value: &mut f32,
    color_at: impl Fn(f32, f32) -> Color32,
) -> Response {
    let desired_size = Vec2::splat(ui.spacing().slider_width);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    // Handle interaction
    if let Some(mpos) = response.interact_pointer_pos() {
        *x_value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
        *y_value = remap_clamp(mpos.y, rect.bottom()..=rect.top(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let size = rect.size();

        // ----------------------------
        // 1. Retrieve or create texture
        // ----------------------------
        let tex_id = egui::Id::new("color_slider_tex");

        let mut tex = ui
            .data(|d| d.get_temp::<Option<egui::TextureHandle>>(tex_id))
            .unwrap_or(None);

        let pixels_w = size.x as usize;
        let pixels_h = size.y as usize;

        let rebuild = tex.is_none() || tex.as_ref().unwrap().size_vec2() != size;

        if rebuild {
            let mut pixels = Vec::with_capacity(pixels_w * pixels_h * 4);

            for y in 0..pixels_h {
                let yt = y as f32 / (pixels_h as f32 - 1.0);

                for x in 0..pixels_w {
                    let xt = x as f32 / (pixels_w as f32 - 1.0);

                    let color = color_at(xt, yt);

                    pixels.push(color.r());
                    pixels.push(color.g());
                    pixels.push(color.b());
                    pixels.push(color.a());
                }
            }

            let image = egui::ColorImage::from_rgba_unmultiplied([pixels_w, pixels_h], &pixels);

            if tex.is_none() {
                tex = Some(ui.ctx().load_texture(
                    "color_slider_2d",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }

            ui.data_mut(|d| {
                d.insert_temp(tex_id, tex.clone());
            });
        }

        let tex = tex.as_ref().unwrap();

        // ----------------------------
        // 2. Paint image
        // ----------------------------
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        // ----------------------------
        // 3. Outline
        // ----------------------------
        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

        // ----------------------------
        // 4. Handle indicator
        // ----------------------------
        let x = lerp(rect.left()..=rect.right(), *x_value);
        let y = lerp(rect.bottom()..=rect.top(), *y_value);
        let picked_color = color_at(*x_value, *y_value);

        ui.painter().circle(
            Pos2::new(x, y),
            rect.width() / 12.0,
            picked_color,
            egui::Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        );
    }

    response
}

/// We use a negative alpha for additive colors within this file (a bit ironic).
///
/// We use alpha=0 to mean "transparent".
fn is_additive_alpha(a: f32) -> bool {
    a < 0.0
}

/// What options to show for alpha
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Alpha {
    /// Set alpha to 1.0, and show no option for it.
    Opaque,

    /// Only show normal blend options for alpha.
    OnlyBlend,

    /// Show both blend and additive options.
    BlendOrAdditive,
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
    let desired_size = Vec2::splat(ui.spacing().slider_width);

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
    if CachedPicker::<T, P>::update_viewport_bounds(&response, rect, &mut picker.viewport) {
        picker.update_coordinates();
        picker.update_texture();
    }

    let mut color_is_valid = false;
    let mut viewport_contains_picked_color = false;

    if ui.is_rect_visible(rect) {
        let painter = ui.painter().with_clip_rect(rect);

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
        let marker_pos = picker.coord_to_pixel(*coord);
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
        }
    }

    ui.data_mut(|d| {
        d.insert_temp(picker_id, picker);
    });

    (response, viewport_contains_picked_color || color_is_valid)
}

// fn color_slider_2d_grid_oklch(
//     ui: &mut Ui,
//     x_value: &mut f32,
//     y_value: &mut f32,
//     stable_value: f32,
//     color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
//     ui_identifyable_label: &str,
// ) -> (Response, bool) {
//     let desired_size = Vec2::splat(ui.spacing().slider_width);
//     let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

//     let size = response.rect.size();
//     let ui_center = rect.center();

//     // Get picker info
//     let picker_id = ui.id().with(ui_identifyable_label);
//     let mut picker_cache = ui.data(|d| d.get_temp::<CachedPickerCartesian>(picker_id));

//     let mut picker_info = match picker_cache {
//         None => CachedPickerCartesian::new(ui, size, stable_value, &color_at),
//         Some(picker_info) => picker_info,
//     };

//     let viewport_bounds = picker_info.viewport_bounds;

//     // Mouse pointer interaction
//     let is_lmb_down = response
//         .ctx
//         .input(|i| i.pointer.button_down(PointerButton::Primary));
//     if is_lmb_down {
//         if let Some(mpos) = response.interact_pointer_pos() {
//             // *x_value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
//             // *y_value = 1.0 - remap_clamp(mpos.y, rect.bottom()..=rect.top(), 0.0..=1.0);

//             let x_normalized = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
//             let y_normalized = remap_clamp(mpos.y, rect.bottom()..=rect.top(), 0.0..=1.0);
//             // dbg!((x_normalized, y_normalized));
//             // let mouse_coord = pixel_to_coordinates(Pos2 { x, y }, size, viewport_bounds);
//             // *x_value = mouse_coord.x;
//             // *y_value = mouse_coord.y;

//             // // 1. normalize mouse in UI space [0,1]
//             // let nx = (mpos.x - response.rect.left());
//             // let ny = (mpos.y - response.rect.top());

//             // 2. convert to WORLD space using viewport
//             // let viewport_bounds = picker_info.viewport_bounds;
//             let x = viewport_bounds[0] + x_normalized * (viewport_bounds[2] - viewport_bounds[0]);
//             let y = viewport_bounds[1] + y_normalized * (viewport_bounds[3] - viewport_bounds[1]);
//             // let mouse_coord = pixel_to_coordinates(
//             //     Pos2 {
//             //         x: x_normalized,
//             //         y: y_normalized,
//             //     },
//             //     size,
//             //     viewport_bounds,
//             // );
//             *x_value = x;
//             *y_value = y;
//         }
//     }

//     let mut viewport_contains_picked_color = false; // default in case rect not visible
//     let mut color_is_valid = false; // default in case rect not visible

//     if ui.is_rect_visible(rect) {
//         let size = rect.size();
//         let painter = ui.painter().with_clip_rect(response.rect);

//         let stable_value_bits = (stable_value * 10000.) as u32;
//         let stable_value_changed = picker_info.stable_value_bits != stable_value_bits;
//         let size_changed = picker_info.size != size;
//         if size_changed {
//             picker_info.update_coordinates(size);
//         }
//         if size_changed || stable_value_changed {
//             picker_info.update_pixel_values_and_border_positions_px(stable_value, &color_at);
//         }

//         // Maybe unify with conditions above
//         let viewport_changed = CachedPickerCartesian::update_viewport_bounds(
//             &response,
//             rect,
//             &mut picker_info.viewport_bounds,
//         );
//         if viewport_changed {
//             picker_info.update_coordinates(size);
//             picker_info.update_pixel_values_and_border_positions_px(stable_value, &color_at);
//         }

//         let tex = picker_info.texture.clone();
//         painter.image(
//             tex.id(),
//             rect,
//             Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
//             Color32::WHITE,
//         );

//         let visuals = ui.style().interact(&response);
//         painter.rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

//         // let nx = (i_x as f32 + 0.5) / pixels_w as f32;
//         // let ny = (i_y as f32 + 0.5) / pixels_h as f32;
//         // let x = viewport_bounds[0] + nx * (viewport_bounds[2] - viewport_bounds[0]);
//         // let y = viewport_bounds[1] + ny * (viewport_bounds[3] - viewport_bounds[1]);
//         // let x = *x_value * (viewport_bounds[2] - viewport_bounds[0]) + viewport_bounds[0];
//         // let y = *y_value * (viewport_bounds[3] - viewport_bounds[1]) + viewport_bounds[1];
//         // let x = viewport_bounds[0] + *x_value / (viewport_bounds[2] - viewport_bounds[0]);
//         // let y = viewport_bounds[1] + *y_value / (viewport_bounds[3] - viewport_bounds[1]);
//         let x = (*x_value - viewport_bounds[0]) / (viewport_bounds[2] - viewport_bounds[0]);
//         // let y = 1.0 - ((*y_value - viewport_bounds[1]) / (viewport_bounds[3] - viewport_bounds[1]));
//         let y =
//             (((1.0 - *y_value) - viewport_bounds[1]) / (viewport_bounds[3] - viewport_bounds[1]));
//         // let marker_pos = Pos2::new(*x_value * size.x, *y_value * size.y);
//         let marker_pos = Pos2::new(x * size.x, y * size.y);

//         let (is_valid, color) = color_at(*x_value, *y_value, stable_value);
//         color_is_valid = is_valid;
//         dbg!(&color_is_valid);

//         let (indicator_color, marker_angle) = if is_valid {
//             (Color32::WHITE, 45.)
//         } else {
//             (Color32::WHITE, 0.)
//         };

//         let viewport_bounds = picker_info.viewport_bounds;
//         let nx = (x - viewport_bounds[0]) / (viewport_bounds[2] - viewport_bounds[0]);
//         let ny = (y - viewport_bounds[1]) / (viewport_bounds[3] - viewport_bounds[1]);
//         let x_ = response.rect.left() + nx * response.rect.width();
//         let y_ = response.rect.top() + ny * response.rect.height();

//         let marker_pos_ = Pos2::new(x_, y_);
//         viewport_contains_picked_color = response.rect.contains(marker_pos_);

//         draw_inward_triangles(
//             &painter,
//             marker_pos + rect.min.to_vec2(),
//             4.0,
//             4.0,
//             4,
//             marker_angle,
//             indicator_color,
//             Stroke {
//                 width: 0.5,
//                 color: Color32::BLACK,
//             },
//         );
//         // if false {
//         if is_valid {
//             picker_info.prev_fallback_color_pos = Some(Pos2 { x, y });
//         } else {
//             let fallback_color_pos = if viewport_contains_picked_color {
//                 // let fallback_color_pos = picker_info.nearest_valid_border_coord(marker_pos);
//                 dbg!((x, y));
//                 let coord_px = coordinates_to_pixel(Pos2 { x, y }, size, viewport_bounds);

//                 let fallback_color_pos = picker_info.nearest_valid_border_pixel(coord_px);
//                 // let fallback_color_pos = picker_info.nearest_valid_border_pixel(Pos2 {
//                 //     // x: *x_value * 0.325, // FIXME: don't hardcode this
//                 //     // y: 1.0 - *y_value,
//                 //     // x: *x_value,
//                 //     // y: 1.0 - *y_value,
//                 //     x: x + size.x,
//                 //     y: y + size.y,
//                 // });
//                 dbg!(&fallback_color_pos);
//                 picker_info.prev_fallback_color_pos = fallback_color_pos;
//                 fallback_color_pos
//             } else {
//                 picker_info.prev_fallback_color_pos
//             };
//             // let color = picker_info.nearest_valid_color(angle, r, invert_radial_value, &color_at);
//             if let Some(pos) = fallback_color_pos {
//                 let pos_px = Pos2 {
//                     x: pos.x + size.x,
//                     y: pos.y + size.y,
//                 };
//                 draw_inward_triangles(
//                     &painter,
//                     pos + rect.min.to_vec2(),
//                     3.0,  // radius
//                     3.0,  // size
//                     4,    // count
//                     45.0, // radial-offset
//                     Color32::WHITE,
//                     Stroke {
//                         width: 0.5,
//                         color: Color32::BLACK,
//                     },
//                 );
//             }
//         }
//         ui.data_mut(|d| {
//             d.insert_temp(picker_id, picker_info.clone());
//         });
//     }

//     if response.is_pointer_button_down_on() {
//         ui.ctx().set_cursor_icon(egui::CursorIcon::None);
//     }

//     (response, viewport_contains_picked_color || color_is_valid)
// }

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
        let inward = -outward;

        // Inner tip sits exactly radius units from center.
        let tip = center + outward * radius;

        let base_center = tip + outward * height;

        let perp = Vec2::new(-outward.y, outward.x);

        let left = base_center + perp * (size * 0.5);
        let right = base_center - perp * (size * 0.5);

        painter.add(Shape::convex_polygon(vec![tip, left, right], fill, stroke));
    }
}

#[derive(Clone)]
struct CachedPickerCartesian {
    stable_value_bits: u32,
    size: Vec2,
    viewport_bounds: [f32; 4],
    texture: egui::TextureHandle,
    coordinates: Arc<[Pos2]>,
    border_positions_px: Arc<[Pos2]>,
    prev_fallback_color_pos: Option<Pos2>,
}

// impl CachedPickerCartesian {
//     fn new(
//         ui: &mut egui::Ui,
//         size: Vec2,
//         stable_value: f32,
//         color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
//     ) -> CachedPickerCartesian {
//         let pixels_w = size.x as usize;
//         let pixels_h = size.y as usize;
//         let viewport_bounds = [0., 0., 1., 1.];
//         // let viewport_bounds = [-2., -2., 2., 2.];
//         // let viewport_bounds = [-1., 0., 1., 1.]; // TODO: ALso create a mode for this, quite nice :)
//         let coordinates = CachedPickerCartesian::calculate_coordinates(size, viewport_bounds);
//         let pixels = CachedPickerCartesian::calculate_color_per_pixel(
//             &coordinates,
//             &size,
//             stable_value,
//             color_at,
//         );
//         let image = egui::ColorImage::from_rgba_unmultiplied([pixels_w, pixels_h], &pixels);
//         let texture =
//             ui.ctx()
//                 .load_texture("oklch_radial_fixed", image, egui::TextureOptions::LINEAR);
//         let border_positions_px =
//             CachedPickerCartesian::calculate_border_from_pixels(size, &pixels);
//         CachedPickerCartesian {
//             stable_value_bits: (stable_value * 10000.) as u32,
//             size: size,
//             viewport_bounds,
//             texture: texture,
//             coordinates: Arc::from(coordinates),
//             border_positions_px: Arc::from(border_positions_px),
//             prev_fallback_color_pos: None,
//         }
//     }

//     fn calculate_coordinates(size: Vec2, viewport_bounds: [f32; 4]) -> Vec<Pos2> {
//         let pixels_w = size.x as usize;
//         let pixels_h = size.y as usize;
//         let mut coordinates: Vec<Pos2> = Vec::with_capacity(pixels_w * pixels_h);

//         for i_y in 0..pixels_h {
//             for i_x in 0..pixels_w {
//                 // let nx = (i_x as f32 + 0.5) / pixels_w as f32;
//                 // let ny = (i_y as f32 + 0.5) / pixels_h as f32;
//                 // let x = viewport_bounds[0] + nx * (viewport_bounds[2] - viewport_bounds[0]);
//                 // let y = 1.0 - (viewport_bounds[1] + ny * (viewport_bounds[3] - viewport_bounds[1]));
//                 let coords = pixel_to_coordinates(
//                     Pos2 {
//                         x: i_x as f32,
//                         y: (pixels_h - i_y) as f32,
//                         // y: i_y as f32,
//                     },
//                     size,
//                     viewport_bounds,
//                 );
//                 coordinates.push(coords);
//             }
//         }
//         coordinates
//     }

//     fn update_coordinates(&mut self, size: Vec2) {
//         let coords = CachedPickerCartesian::calculate_coordinates(size, self.viewport_bounds);
//         self.coordinates = Arc::from(coords);
//     }

//     fn update_pixel_values_and_border_positions_px(
//         &mut self,
//         stable_value: f32,
//         color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
//     ) {
//         self.stable_value_bits = (stable_value * 10000.) as u32;

//         let pixels_w = self.size.x as usize;
//         let pixels_h = self.size.y as usize;
//         let coordinates = self.coordinates.clone();
//         let pixels = CachedPickerCartesian::calculate_color_per_pixel(
//             &coordinates,
//             &self.size,
//             stable_value,
//             color_at,
//         );

//         let border_positions_px =
//             CachedPickerCartesian::calculate_border_from_pixels(self.size, &pixels);
//         self.border_positions_px = Arc::from(border_positions_px);

//         let image = egui::ColorImage::from_rgba_unmultiplied([pixels_w, pixels_h], &pixels);
//         self.texture.set(image, egui::TextureOptions::LINEAR);
//     }

//     fn update_viewport_bounds(
//         response: &egui::Response,
//         ui_rect: egui::Rect,
//         viewport_bounds: &mut [f32; 4],
//     ) -> bool {
//         let width = viewport_bounds[2] - viewport_bounds[0];
//         let height = viewport_bounds[3] - viewport_bounds[1];

//         let mut viewport_changed = false;
//         //
//         // Pan
//         //
//         let is_pan_button_down = response.ctx.input(|i| {
//             i.pointer.button_down(PointerButton::Secondary)
//                 || i.pointer.button_down(PointerButton::Middle)
//         });
//         if is_pan_button_down && response.dragged() {
//             viewport_changed = true;

//             let drag = response.drag_delta();

//             let dx_world = drag.x / ui_rect.width() * width;
//             let dy_world = drag.y / ui_rect.height() * height;

//             // Note: y-axis is negative
//             viewport_bounds[0] -= dx_world;
//             viewport_bounds[1] += dy_world;
//             viewport_bounds[2] -= dx_world;
//             viewport_bounds[3] += dy_world;
//         }

//         //
//         // Zoom
//         //
//         let scroll = response.ctx.input(|i| i.raw_scroll_delta.y);

//         if scroll.abs() > 0.0 {
//             if let Some(mouse_pos) = response.hover_pos() {
//                 let nx = (mouse_pos.x - ui_rect.left()) / ui_rect.width();
//                 let ny = (mouse_pos.y - ui_rect.top()) / ui_rect.height();

//                 let mouse_world_x =
//                     viewport_bounds[0] + nx * (viewport_bounds[2] - viewport_bounds[0]);

//                 let mouse_world_y =
//                     viewport_bounds[1] + ny * (viewport_bounds[3] - viewport_bounds[1]);

//                 {
//                     // Do not zoom if we are already zoomed in rather far
//                     let zoom_factor = if scroll > 0.0 { 0.9 } else { 1.1 };

//                     let width = viewport_bounds[2] - viewport_bounds[0];
//                     let height = viewport_bounds[3] - viewport_bounds[1];

//                     const MAX_ZOOM_FACTOR: f32 = 7.0;
//                     const WORLD_SIZE: f32 = 4.0; // [0, 1]

//                     let min_width = WORLD_SIZE / MAX_ZOOM_FACTOR;
//                     let min_height = WORLD_SIZE / MAX_ZOOM_FACTOR;

//                     let new_width = width * zoom_factor;
//                     let new_height = height * zoom_factor;

//                     // Prevent further zoom-in once maximum zoom is reached.
//                     if scroll > 0.0 && (new_width < min_width || new_height < min_height) {
//                         // early return to guard against very deep zooms, viewport not changed
//                         return viewport_changed;
//                     }
//                 }
//                 viewport_changed = true;

//                 let zoom_factor = if scroll > 0.0 { 0.9 } else { 1.1 };

//                 viewport_bounds[0] =
//                     mouse_world_x + (viewport_bounds[0] - mouse_world_x) * zoom_factor;

//                 viewport_bounds[2] =
//                     mouse_world_x + (viewport_bounds[2] - mouse_world_x) * zoom_factor;

//                 viewport_bounds[1] =
//                     mouse_world_y + (viewport_bounds[1] - mouse_world_y) * zoom_factor;

//                 viewport_bounds[3] =
//                     mouse_world_y + (viewport_bounds[3] - mouse_world_y) * zoom_factor;
//             }
//         }
//         if viewport_changed {
//             CachedPickerCartesian::clamp_viewport(viewport_bounds)
//         }
//         viewport_changed
//     }

//     fn clamp_viewport(viewport: &mut [f32; 4]) {
//         let min_world = 0.0;
//         let max_world = 1.0;

//         let mut min_x = viewport[0];
//         let mut min_y = viewport[1];
//         let mut max_x = viewport[2];
//         let mut max_y = viewport[3];

//         let width = max_x - min_x;
//         let height = max_y - min_y;

//         // -------------------------
//         // 1. Clamp zoom FIRST (uniform bounds)
//         // -------------------------
//         let max_width = max_world - min_world;
//         let max_height = max_world - min_world;

//         let width = width.min(max_width);
//         let height = height.min(max_height);

//         // compute center BEFORE correction
//         let cx = (min_x + max_x) * 0.5;
//         let cy = (min_y + max_y) * 0.5;

//         // rebuild viewport around center (this fixes drift)
//         min_x = cx - width * 0.5;
//         max_x = cx + width * 0.5;
//         min_y = cy - height * 0.5;
//         max_y = cy + height * 0.5;

//         // -------------------------
//         // 2. Clamp position (shift as a block)
//         // -------------------------
//         let dx_left = min_world - min_x;
//         let dx_right = max_world - max_x;

//         if dx_left > 0.0 {
//             min_x += dx_left;
//             max_x += dx_left;
//         } else if dx_right < 0.0 {
//             min_x += dx_right;
//             max_x += dx_right;
//         }

//         let dy_bottom = min_world - min_y;
//         let dy_top = max_world - max_y;

//         if dy_bottom > 0.0 {
//             min_y += dy_bottom;
//             max_y += dy_bottom;
//         } else if dy_top < 0.0 {
//             min_y += dy_top;
//             max_y += dy_top;
//         }
//         viewport[0] = min_x;
//         viewport[1] = min_y;
//         viewport[2] = max_x;
//         viewport[3] = max_y;
//     }

//     fn calculate_border_from_pixels(size: Vec2, pixels: &Vec<u8>) -> Vec<Pos2> {
//         // Note: same as polar version, consider unifying
//         let pixels_w = size.x as usize;
//         let pixels_h = size.y as usize;
//         let mut border_positions_px = Vec::new();
//         for y in 1..pixels_h - 1 {
//             for x in 1..pixels_w - 1 {
//                 let i = (y * pixels_w + x) * 4;

//                 let valid = pixels[i + 3] != 0;
//                 if !valid {
//                     continue;
//                 }

//                 let mut neighbor_invalid = false;

//                 let mut ni: usize = 0;
//                 for dy in -1..=1 {
//                     for dx in -1..=1 {
//                         if dx == 0 && dy == 0 {
//                             continue;
//                         }

//                         let nx = (x as i32 + dx) as usize;
//                         let ny = (y as i32 + dy) as usize;

//                         ni = (ny * pixels_w + nx) * 4;

//                         if pixels[ni + 3] == 0 {
//                             neighbor_invalid = true;
//                             break;
//                         }
//                     }
//                     if neighbor_invalid {
//                         break;
//                     }
//                 }

//                 if neighbor_invalid {
//                     // pixels[ni + 0] = 200;
//                     // pixels[ni + 1] = 0;
//                     // pixels[ni + 2] = 0;
//                     // pixels[ni + 3] = 255;
//                     border_positions_px.push(Pos2 {
//                         x: x as f32,
//                         y: y as f32,
//                     });
//                 }
//             }
//         }
//         border_positions_px
//     }

//     fn calculate_color_per_pixel(
//         coordinates: &[Pos2],
//         size: &Vec2,
//         stable_value: f32,
//         color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
//     ) -> Vec<u8> {
//         let pixels_w = size.x as usize;
//         let pixels_h = size.y as usize;

//         let mut pixels = vec![0 as u8; pixels_w * pixels_h * 4];
//         for i_y in 0..pixels_h {
//             for i_x in 0..pixels_w {
//                 let coord_i = i_y * pixels_w + i_x;

//                 let coord = coordinates[coord_i];

//                 let (is_valid, color) = color_at(coord.x, coord.y, stable_value);

//                 if is_valid {
//                     let px_i = (i_y * pixels_w + i_x) * 4;
//                     pixels[px_i] = color.r();
//                     pixels[px_i + 1] = color.g();
//                     pixels[px_i + 2] = color.b();
//                     pixels[px_i + 3] = color.a();
//                 }
//             }
//         }
//         // dbg!([minx, miny, maxx, maxy]);
//         pixels
//     }

//     fn nearest_valid_border_pixel(&self, coord_px: Pos2) -> Option<Pos2> {
//         let mut min_distance = f32::MAX;
//         let mut min_distance_i = 0;
//         let mut found_nothing = true;
//         let border_positions_px = self.border_positions_px.clone();
//         let viewport_bounds = self.viewport_bounds;
//         for (i, border_px) in border_positions_px.iter().enumerate() {
//             let distance =
//                 ((coord_px.x - border_px.x).powi(2) + (coord_px.y - border_px.y).powi(2)).sqrt();
//             if distance < min_distance {
//                 min_distance_i = i;
//                 min_distance = distance;
//                 found_nothing = false;
//             }
//         }
//         if found_nothing {
//             return None;
//         } else {
//             let lab = border_positions_px[min_distance_i];
//             let bla = pixel_to_coordinates(lab, self.size, viewport_bounds);
//             println!("Coord {:?} nearby: {:?}", coord_px, bla);
//             return Some(border_positions_px[min_distance_i]);
//         }
//     }

//     fn nearest_valid_color(
//         &self,
//         coord: Pos2,
//         color_at: impl Fn(f32, f32, f32) -> (bool, Color32), // TODO: move to self as function pointer
//     ) -> Color32 {
//         // Here we could sample the pixels vec, but we can also determine the color
//         // based on coordinates. This makes the function less pleasant to use, as we
//         // have to supply the color_at closure, but we would then not need to save
//         // a copy of the texture in RAM.
//         let stable_value = self.stable_value_bits as f32 / 10000.;

//         // If color exists at provided values, return the color
//         let (is_valid, color) = color_at(coord.x, coord.y, stable_value);
//         if is_valid {
//             return color;
//         }

//         // If color does not exist, return nearest color
//         println!("Fallback color coord: {:?}", &coord);
//         let coord_px = coordinates_to_pixel(
//             Pos2 {
//                 x: coord.x,
//                 y: 1.0 - coord.y,
//             },
//             self.size,
//             self.viewport_bounds,
//         );
//         let sample_pixel = self.nearest_valid_border_pixel(coord_px);
//         let fallback_color = Color32::BLACK; // Take a look at the values where no possibilities left, what is the last color to disappear? Use that!
//         let color = match sample_pixel {
//             None => fallback_color,
//             Some(sample_px) => {
//                 let sample_pos = pixel_to_coordinates(sample_px, self.size, self.viewport_bounds);
//                 let (_, color) = color_at(sample_pos.x, 1.0 - sample_pos.y, stable_value);
//                 color
//             }
//         };
//         color
//     }
// }

struct Pos {
    x: usize,
    y: usize,
}

pub fn tex_coord_to_lch(
    coord: Pos,
    width: usize,
    height: usize,
    stable_value: f32,
    invert_radial_value: bool,
    // your existing mapping function reused here
    to_lch: impl Fn(f32, f32, f32) -> Option<Oklch>,
) -> Option<Oklch> {
    let x = coord.x;
    let y = coord.y;

    // normalize to 0..1 texture space
    let u = x as f32 / (width as f32 - 1.0);
    let v = y as f32 / (height as f32 - 1.0);

    let angle = u;
    let radius = if invert_radial_value { 1.0 - v } else { v };

    to_lch(angle, radius, stable_value)
}

// fn color_slider_2d_radial(
//     ui: &mut Ui,
//     stable_value: f32,
//     angle_value: &mut f32,
//     radial_value: &mut f32,
//     invert_radial_value: bool,
//     color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
//     ui_identifyable_label: &str,
// ) -> (Response, bool) {
//     let desired_size = Vec2::splat(ui.spacing().slider_width);
//     let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

//     let size = response.rect.size();
//     let ui_center = rect.center();
//     let radius = rect.width().min(rect.height()) * 0.5;

//     // Get picker info
//     let picker_id = ui.id().with(ui_identifyable_label);
//     let mut picker_cache = ui.data(|d| d.get_temp::<CachedPicker>(picker_id));

//     let mut picker_info = match picker_cache {
//         None => CachedPicker::new(ui, size, stable_value, invert_radial_value, &color_at),
//         Some(picker_info) => picker_info,
//     };

//     // Mouse pointer interaction
//     let is_lmb_down = response
//         .ctx
//         .input(|i| i.pointer.button_down(PointerButton::Primary));
//     if is_lmb_down {
//         if let Some(mpos) = response.interact_pointer_pos() {
//             // FIXME: selected position makes oval shape when hue is dragged!

//             // 1. normalize mouse in UI space [0,1]
//             let nx = (mpos.x - response.rect.left()) / size.x;
//             let ny = (mpos.y - response.rect.top()) / size.y;

//             // 2. convert to WORLD space using viewport
//             let viewport_bounds = picker_info.viewport;
//             let vx = viewport_bounds[0] + nx * (viewport_bounds[2] - viewport_bounds[0]);

//             let vy = viewport_bounds[1] + ny * (viewport_bounds[3] - viewport_bounds[1]);

//             // 3. convert world → polar
//             let r = (vx * vx + vy * vy).sqrt().clamp(0.0, 1.0);

//             let angle = (-vy).atan2(vx);

//             let mut h = angle / std::f32::consts::TAU;
//             if h < 0.0 {
//                 h += 1.0;
//             }

//             *angle_value = h.clamp(0.0, 1.0);
//             *radial_value = if invert_radial_value { 1.0 - r } else { r };
//         }
//     }

//     let mut viewport_contains_picked_color = false; // default in case rect not visible
//     let mut color_is_valid = false; // default in case rect not visible

//     if ui.is_rect_visible(rect) {
//         let size = rect.size();

//         // Radial lines on background
//         let painter = ui.painter().with_clip_rect(response.rect);
//         let viewport_bounds = picker_info.viewport_bounds;
//         let cx = response.rect.left()
//             + ((0.0 - viewport_bounds[0]) / (viewport_bounds[2] - viewport_bounds[0]))
//                 * response.rect.width();

//         let cy = response.rect.top()
//             + ((0.0 - viewport_bounds[1]) / (viewport_bounds[3] - viewport_bounds[1]))
//                 * response.rect.height();

//         let world_to_screen = response.rect.width() / (viewport_bounds[2] - viewport_bounds[0]);

//         for i in 0..6 {
//             let world_r = 0.01 + 0.15 * i as f32;
//             let screen_r = world_r * world_to_screen;

//             painter.circle(
//                 Pos2::new(cx, cy),
//                 screen_r,
//                 Color32::TRANSPARENT,
//                 egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(200, 200, 200, 100)),
//             );
//         }

//         let stable_value_bits = (stable_value * 10000.) as u32;
//         let stable_value_changed = picker_info.stable_value_bits != stable_value_bits;
//         let size_changed = picker_info.size != size;
//         if size_changed {
//             picker_info.update_polar_coordinates(size);
//         }
//         if size_changed || stable_value_changed {
//             picker_info.update_pixel_values_and_border_positions_px(
//                 stable_value,
//                 invert_radial_value,
//                 &color_at,
//             );
//         }

//         // Maybe unify with conditions above
//         let viewport_changed =
//             CachedPicker::update_viewport_bounds(&response, rect, &mut picker_info.viewport_bounds);
//         if viewport_changed {
//             picker_info.update_polar_coordinates(size);
//             picker_info.update_pixel_values_and_border_positions_px(
//                 stable_value,
//                 invert_radial_value,
//                 &color_at,
//             );
//         }

//         let sample_radius = if invert_radial_value {
//             (1.0 - *radial_value).clamp(0.0, 1.0)
//         } else {
//             (*radial_value).clamp(0.0, 1.0)
//         };

//         let tex = picker_info.texture.clone();
//         painter.image(
//             tex.id(),
//             rect,
//             Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
//             Color32::WHITE,
//         );

//         let visuals = ui.style().interact(&response);
//         painter.rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

//         // Handle
//         let angle = *angle_value * std::f32::consts::TAU;

//         let r = if invert_radial_value {
//             1.0 - *radial_value
//         } else {
//             *radial_value
//         };

//         // -----------------------------
//         // 1. Polar → WORLD space
//         // -----------------------------
//         let wx = angle.cos() * r;
//         let wy = -angle.sin() * r;

//         // -----------------------------
//         // 2. WORLD → SCREEN space
//         // -----------------------------
//         let size = ui_center * 2.0; // or response.rect.size()

//         let viewport_bounds = picker_info.viewport_bounds;
//         let nx = (wx - viewport_bounds[0]) / (viewport_bounds[2] - viewport_bounds[0]);

//         let ny = (wy - viewport_bounds[1]) / (viewport_bounds[3] - viewport_bounds[1]);

//         let x = response.rect.left() + nx * response.rect.width();
//         let y = response.rect.top() + ny * response.rect.height();

//         let marker_pos = Pos2::new(x, y);
//         viewport_contains_picked_color = response.rect.contains(marker_pos);

//         // -----------------------------
//         // 3. Color sampling MUST also use world-space mapping
//         // -----------------------------
//         let (is_valid, color) =
//             color_at(angle / std::f32::consts::TAU, *radial_value, stable_value);
//         color_is_valid = is_valid;

//         let (indicator_color, marker_angle) = if is_valid {
//             (Color32::WHITE, 45.)
//         } else {
//             (Color32::WHITE, 0.)
//         };

//         draw_inward_triangles(
//             &painter,
//             marker_pos,
//             4.0,
//             4.0,
//             4,
//             marker_angle,
//             indicator_color,
//             Stroke {
//                 width: 0.5,
//                 color: Color32::BLACK,
//             },
//         );
//         // Draw indicator at loction of fallback color
//         if is_valid {
//             picker_info.prev_fallback_color_pos = Some(PolarCoordinate {
//                 radius: sample_radius,
//                 // radius: *radial_value,
//                 angle,
//             });
//         } else {
//             let fallback_color_pos = if viewport_contains_picked_color {
//                 let fallback_color_pos = picker_info.nearest_valid_border_coord(r, angle);
//                 let fallback_color_pos_polar = match fallback_color_pos {
//                     Some(pos) => Some(pixel_to_polar(
//                         pos.x,
//                         pos.y,
//                         picker_info.size.x as usize,
//                         picker_info.size.y as usize,
//                         picker_info.viewport_bounds,
//                     )),
//                     None => None,
//                 };
//                 picker_info.prev_fallback_color_pos = fallback_color_pos_polar;
//                 fallback_color_pos // Return xy, not polar
//             } else {
//                 let fallback_color_pos_polar = picker_info.prev_fallback_color_pos;
//                 let fallback_color_pos = match fallback_color_pos_polar {
//                     Some(polar_pos) => {
//                         let (x, y) = polar_to_pixel(
//                             polar_pos.angle,
//                             polar_pos.radius,
//                             picker_info.size.x as usize,
//                             picker_info.size.y as usize,
//                             picker_info.viewport_bounds,
//                         );
//                         Some(Pos2 { x, y })
//                     }
//                     None => None,
//                 };
//                 fallback_color_pos
//             };
//             // let color = picker_info.nearest_valid_color(angle, r, invert_radial_value, &color_at);
//             if let Some(pos) = fallback_color_pos {
//                 draw_inward_triangles(
//                     &painter,
//                     pos + rect.min.to_vec2(),
//                     3.0,  // radius
//                     3.0,  // size
//                     4,    // count
//                     45.0, // radial-offset
//                     Color32::WHITE,
//                     Stroke {
//                         width: 0.5,
//                         color: Color32::BLACK,
//                     },
//                 );
//             }
//         }

//         ui.data_mut(|d| {
//             d.insert_temp(picker_id, picker_info.clone());
//         });
//     }

//     if response.is_pointer_button_down_on() {
//         ui.ctx().set_cursor_icon(egui::CursorIcon::None);
//     }

//     (response, viewport_contains_picked_color || color_is_valid)
// }

#[derive(Debug, Clone)]
enum OKLabPlotType {
    StableLightness,
    StableChroma,
    StableHue,
}

fn color_picker_oklch_2d(ui: &mut Ui, lch: &mut Oklch, alpha: &mut f32) -> (bool, Color32) {
    let current_color_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    let mut update_color = false;

    let state_id = ui.id().with("oklch_ui_state");

    // Todo: base default on color value
    // Todo: If the color changed beneath us, reset state to match new color
    let mut state: OklchUiState =
        ui.data(|d| d.get_temp::<OklchUiState>(state_id))
            .unwrap_or(OklchUiState {
                l: 0.5,
                c: 0.5,
                h: 0.5,
            });

    let l = state.l;
    let c = state.c * 0.325;
    let h = state.h * std::f32::consts::TAU;

    let mut tab = ui.data(|d| {
        d.get_temp::<OKLabPlotType>(ui.id())
            .unwrap_or(OKLabPlotType::StableHue)
    });
    let mut tab_switched = false;
    ui.horizontal(|ui| {
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableHue), "Lightness/Chroma")
            .clicked()
        {
            tab = OKLabPlotType::StableHue;
            tab_switched = true;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableChroma), "Hue/Lightness")
            .clicked()
        {
            tab = OKLabPlotType::StableChroma;
            tab_switched = true;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableLightness), "Hue/Chroma")
            .clicked()
        {
            tab = OKLabPlotType::StableLightness;
            tab_switched = true;
        }
    });

    ui.horizontal(|ui| {
        input_type_button_ui(ui);

        // TODO: get nearest color! nocheckin
        let color_sample = oklch_to_color32(Oklch {
            l,
            c: c * 0.325,
            h: h * std::f32::consts::TAU,
        });
        // TODO: what about color_sample.valid?
        let color = color_sample.color;

        if ui
            .button("📋")
            .on_hover_text("Click to copy color values")
            .clicked()
        {
            ui.ctx()
                .copy_text(format!("{}, {}, {}", color.r(), color.g(), color.b()));
        }
        ui.label(format!(
            "R {}    G {}    B {}",
            color.r(),
            color.g(),
            color.b()
        ))
    });

    ui.data_mut(|d| {
        d.insert_temp(ui.id(), tab.clone());
    });

    if tab_switched {
        state.l = lch.l;
        state.c = lch.c / 0.325;
        state.h = lch.h / std::f32::consts::TAU;
    }

    let color_sample = oklch_to_color32(lch.clone()); // technically the color from previous frame, but I want to display this above the 2d picker
    show_srgba_unmultiplied(ui, color_sample.color.to_array(), current_color_size)
        .on_hover_text("Selected color");

    let (new_color, update_color) = match tab {
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

            state.c = coord.x;
            state.l = coord.y;

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

            state.h = coord.angle / std::f32::consts::TAU;
            state.l = 1.0 - coord.radius;

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

            state.h = coord.angle / std::f32::consts::TAU;
            state.c = coord.radius;

            let picker_id = ui.id().with("StableLightnessColorPicker");

            let picker = ui
                .data(|d| d.get_temp::<StableLightnessPicker>(picker_id))
                .unwrap();

            (picker.nearest_valid_color(coord), update_color)
        }
    };

    if update_color {
        *lch = color32_to_oklch(new_color);
    }

    // --- Lightness slider (0..1) ---
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.l,
            lch.l,
            color_sample.color,
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

    // --- Chroma slider ---
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.c,
            lch.c / 0.325,
            color_sample.color,
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

    // --- Hue slider ---
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
            color_sample.color,
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

    ui.data_mut(|d| {
        d.insert_temp(state_id, state);
    });

    // TODO: Add alpha slider?

    (update_color, new_color)
}

fn input_type_button_ui(ui: &mut Ui) {
    let mut input_type = ui.style().visuals.numeric_color_space;
    if input_type.toggle_button_ui(ui).changed() {
        ui.ctx().all_styles_mut(|s| {
            s.visuals.numeric_color_space = input_type;
        });
    }
}

/// Shows 4 `DragValue` widgets to be used to edit the RGBA u8 values.
/// Alpha's `DragValue` is hidden when `Alpha::Opaque`.
///
/// Returns `true` on change.
fn srgba_edit_ui(ui: &mut Ui, [r, g, b, a]: &mut [u8; 4], alpha: Alpha) -> bool {
    let mut edited = false;

    ui.horizontal(|ui| {
        input_type_button_ui(ui);

        if ui
            .button("📋")
            .on_hover_text("Click to copy color values")
            .clicked()
        {
            if alpha == Alpha::Opaque {
                ui.ctx().copy_text(format!("{r}, {g}, {b}"));
            } else {
                ui.ctx().copy_text(format!("{r}, {g}, {b}, {a}"));
            }
        }
        edited |= DragValue::new(r).speed(0.5).prefix("R ").ui(ui).changed();
        edited |= DragValue::new(g).speed(0.5).prefix("G ").ui(ui).changed();
        edited |= DragValue::new(b).speed(0.5).prefix("B ").ui(ui).changed();
        if alpha != Alpha::Opaque {
            edited |= DragValue::new(a).speed(0.5).prefix("A ").ui(ui).changed();
        }
    });

    edited
}

/// Shows 4 `DragValue` widgets to be used to edit the RGBA f32 values.
/// Alpha's `DragValue` is hidden when `Alpha::Opaque`.
///
/// Returns `true` on change.
fn rgba_edit_ui(ui: &mut Ui, [r, g, b, a]: &mut [f32; 4], alpha: Alpha) -> bool {
    fn drag_value(ui: &mut Ui, prefix: &str, value: &mut f32) -> Response {
        DragValue::new(value)
            .speed(0.003)
            .prefix(prefix)
            .range(0.0..=1.0)
            .custom_formatter(|n, _| format!("{n:.03}"))
            .ui(ui)
    }

    let mut edited = false;

    ui.horizontal(|ui| {
        input_type_button_ui(ui);

        if ui
            .button("📋")
            .on_hover_text("Click to copy color values")
            .clicked()
        {
            if alpha == Alpha::Opaque {
                ui.ctx().copy_text(format!("{r:.03}, {g:.03}, {b:.03}"));
            } else {
                ui.ctx()
                    .copy_text(format!("{r:.03}, {g:.03}, {b:.03}, {a:.03}"));
            }
        }

        edited |= drag_value(ui, "R ", r).changed();
        edited |= drag_value(ui, "G ", g).changed();
        edited |= drag_value(ui, "B ", b).changed();
        if alpha != Alpha::Opaque {
            edited |= drag_value(ui, "A ", a).changed();
        }
    });

    edited
}

pub fn color_edit_button_oklcha(ui: &mut Ui, color: &mut Color32) -> Response {
    let popup_id = ui.auto_id_with("popup");
    let open = Popup::is_id_open(ui.ctx(), popup_id);
    let mut button_response = color_button(ui, color.to_srgba_unmultiplied(), open);
    if ui.style().explanation_tooltips {
        button_response = button_response.on_hover_text("Click to edit color");
    }

    const COLOR_SLIDER_WIDTH: f32 = 275.0;
    // egui::Window::new("Debug: data tiles").movable(true).resizable(true).show(ctx, |ui| {
    Popup::menu(&button_response)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = COLOR_SLIDER_WIDTH;
            if color_picker_oklcha_2d(ui, color) {
                button_response.mark_changed();
            }
        });

    button_response
}

pub fn color_picker_oklcha_2d(ui: &mut Ui, color: &mut Color32) -> bool {
    let mut new_lch = color32_to_oklch(*color);
    let mut new_alpha = color.a() as f32 / 255.0;

    let (update_color, new_color) = color_picker_oklch_2d(ui, &mut new_lch, &mut new_alpha);

    if update_color {
        *color = new_color;
    }

    update_color
}
