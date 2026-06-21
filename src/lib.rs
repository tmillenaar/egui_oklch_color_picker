//! Color picker widgets.

// use egui::util::fixed_cache::FixedCache;
mod conversions;
use std::f32;
use std::sync::Arc;

use crate::conversions::*;
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

/// Number of vertices per dimension in the color sliders.
/// We need at least 6 for hues, and more for smooth 2D areas.
/// Should always be a multiple of 6 to hit the peak hues in HSV/HSL (every 60°).
const N: u32 = 6 * 6;

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
            if modulus(i_x as f32 + i_y as f32, 2.) == 0. {
                i_x += 1; // Offset cell
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
pub fn show_color(ui: &mut Ui, color: impl Into<Color32>, desired_size: Vec2) -> Response {
    show_color32(ui, color.into(), desired_size)
}

fn show_color32(ui: &mut Ui, color: Color32, desired_size: Vec2) -> Response {
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
    if ui.is_rect_visible(rect) {
        show_color_at(ui.painter(), color, rect);
    }
    response
}

/// Show a color with background checkers to demonstrate transparency (if any).
pub fn show_color_at(painter: &Painter, color: Color32, rect: Rect) {
    if color.is_opaque() {
        painter.rect_filled(rect, 0.0, color);
    } else {
        // Transparent: how both the transparent and opaque versions of the color
        background_checkers(painter, rect, 4);

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

fn color_slider_1d(ui: &mut Ui, value: &mut f32, color_at: impl Fn(f32) -> Color32) -> Response {
    #![expect(clippy::identity_op)]

    let desired_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    if let Some(mpos) = response.interact_pointer_pos() {
        *value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        background_checkers(ui.painter(), rect, 4); // for alpha:

        {
            // fill color:
            let mut mesh = Mesh::default();
            for i in 0..=N {
                let t = i as f32 / (N as f32);
                let color = color_at(t);
                let x = lerp(rect.left()..=rect.right(), t);
                mesh.colored_vertex(pos2(x, rect.top()), color);
                mesh.colored_vertex(pos2(x, rect.bottom()), color);
                if i < N {
                    mesh.add_triangle(2 * i + 0, 2 * i + 1, 2 * i + 2);
                    // mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
                }
            }
            ui.painter().add(Shape::mesh(mesh));
        }

        ui.painter()
            .rect_stroke(rect, 0.0, visuals.bg_stroke, StrokeKind::Inside); // outline

        {
            // Show where the slider is at:
            let x = lerp(rect.left()..=rect.right(), *value);
            let r = rect.height() / 4.0;
            let picked_color = color_at(*value);
            ui.painter().add(Shape::convex_polygon(
                vec![
                    pos2(x, rect.center().y),   // tip
                    pos2(x + r, rect.bottom()), // right bottom
                    pos2(x - r, rect.bottom()), // left bottom
                ],
                picked_color,
                Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
            ));
        }
    }

    response
}

#[derive(Debug, Clone)]
struct OklchUiState {
    // normalized 0..1
    l: f32,
    c: f32,
    h: f32,
    previous_color: Oklch,
}

fn color_slider_1d_grid(
    ui: &mut Ui,
    slider_value: &mut f32,
    slider_value_nearest_color: f32,
    nearest_color: Color32,
    color_at: impl Fn(f32) -> (bool, Color32),
    background_color_at: Option<impl Fn(f32) -> Color32>,
    axis_name: &str,
) -> Response {
    let desired_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    // let state_id = ui.id().with(("oklch_ui_state", axis_name));

    // // Todo: base default on color value
    // // Todo: also store the previous color. If the color changed beneath us, reset state to match new color
    // let mut state: OklchUiState =
    //     ui.data(|d| d.get_temp::<OklchUiState>(state_id))
    //         .unwrap_or(OklchUiState {
    //             l: 0.5,
    //             c: 0.5,
    //             h: 0.5,
    //             previous_color: color_at(*value),
    //         });

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

                    let (is_valid, color) = color_at(t);
                    if is_valid {
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
        let (is_valid, _) = color_at(*slider_value);
        if !is_valid {
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
        // ui.painter().add(Shape::convex_polygon(
        //     vec![
        //         pos2(x + r / 2., rect.top()),
        //         pos2(x, rect.top() + r),
        //         pos2(x - r / 2., rect.top()),
        //     ],
        //     current_color,
        //     Stroke::new(visuals.fg_stroke.width, contrast_color(current_color)),
        // ));
        // ui.painter().add(Shape::convex_polygon(
        //     vec![
        //         pos2(x + r / 2., rect.bottom()),
        //         pos2(x, rect.bottom() - r),
        //         pos2(x - r / 2., rect.bottom()),
        //     ],
        //     current_color,
        //     Stroke::new(visuals.fg_stroke.width, contrast_color(current_color)),
        // ));
    }

    response
}

/// # Arguments
/// * `x_value` - X axis, either saturation or value (0.0-1.0).
/// * `y_value` - Y axis, either saturation or value (0.0-1.0).
/// * `color_at` - A function that dictates how the mix of saturation and value will be displayed in the 2d slider.
///
/// e.g.: `|x_value, y_value| HsvaGamma { h: 1.0, s: x_value, v: y_value, a: 1.0 }.into()` displays the colors as follows:
/// * top-left: white `[s: 0.0, v: 1.0]`
/// * top-right: fully saturated color `[s: 1.0, v: 1.0]`
/// * bottom-right: black `[s: 0.0, v: 1.0].`
fn color_slider_2d(
    ui: &mut Ui,
    x_value: &mut f32,
    y_value: &mut f32,
    color_at: impl Fn(f32, f32) -> Color32,
) -> Response {
    let desired_size = Vec2::splat(ui.spacing().slider_width);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    if let Some(mpos) = response.interact_pointer_pos() {
        *x_value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
        *y_value = remap_clamp(mpos.y, rect.bottom()..=rect.top(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let mut mesh = Mesh::default();

        for xi in 0..=N {
            for yi in 0..=N {
                let xt = xi as f32 / (N as f32);
                let yt = yi as f32 / (N as f32);
                let color = color_at(xt, yt);
                let x = lerp(rect.left()..=rect.right(), xt);
                let y = lerp(rect.bottom()..=rect.top(), yt);
                mesh.colored_vertex(pos2(x, y), color);

                if xi < N && yi < N {
                    let x_offset = 1;
                    let y_offset = N + 1;
                    let tl = yi * y_offset + xi;
                    mesh.add_triangle(tl, tl + x_offset, tl + y_offset);
                    mesh.add_triangle(tl + x_offset, tl + y_offset, tl + y_offset + x_offset);
                }
            }
        }
        ui.painter().add(Shape::mesh(mesh)); // fill

        ui.painter()
            .rect_stroke(rect, 0.0, visuals.bg_stroke, StrokeKind::Inside); // outline

        // Show where the slider is at:
        let x = lerp(rect.left()..=rect.right(), *x_value);
        let y = lerp(rect.bottom()..=rect.top(), *y_value);
        let picked_color = color_at(*x_value, *y_value);
        ui.painter().add(epaint::CircleShape {
            center: pos2(x, y),
            radius: rect.width() / 12.0,
            fill: picked_color,
            stroke: Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        });
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

// fn oklch_to_color32(c: Oklch) -> Color32 {
//     let srgb = linear_srgb_to_srgb(oklab_to_linear_srgb(oklch_to_oklab(c)));

//     Color32::from_rgb(
//         (srgb.r * 255.) as u8,
//         (srgb.g * 255.) as u8,
//         (srgb.b * 255.) as u8,
//     )
// }

// fn color_picker_hsvag_2d(ui: &mut Ui, hsvag: &mut HsvaGamma, alpha: Alpha) {
//     use egui::style::NumericColorSpace;

//     let alpha_control = if is_additive_alpha(hsvag.a) {
//         Alpha::Opaque // no alpha control for additive colors
//     } else {
//         alpha
//     };

//     match ui.style().visuals.numeric_color_space {
//         NumericColorSpace::GammaByte => {
//             let mut srgba_unmultiplied = Hsva::from(*hsvag).to_srgba_unmultiplied();
//             // Only update if changed to avoid rounding issues.
//             if srgba_edit_ui(ui, &mut srgba_unmultiplied, alpha_control) {
//                 if is_additive_alpha(hsvag.a) {
//                     let alpha = hsvag.a;

//                     *hsvag = HsvaGamma::from(Hsva::from_additive_srgb([
//                         srgba_unmultiplied[0],
//                         srgba_unmultiplied[1],
//                         srgba_unmultiplied[2],
//                     ]));

//                     // Don't edit the alpha:
//                     hsvag.a = alpha;
//                 } else {
//                     // Normal blending.
//                     *hsvag = HsvaGamma::from(Hsva::from_srgba_unmultiplied(srgba_unmultiplied));
//                 }
//             }
//         }

//         NumericColorSpace::Linear => {
//             let mut rgba_unmultiplied = Hsva::from(*hsvag).to_rgba_unmultiplied();
//             // Only update if changed to avoid rounding issues.
//             if rgba_edit_ui(ui, &mut rgba_unmultiplied, alpha_control) {
//                 if is_additive_alpha(hsvag.a) {
//                     let alpha = hsvag.a;

//                     *hsvag = HsvaGamma::from(Hsva::from_rgb([
//                         rgba_unmultiplied[0],
//                         rgba_unmultiplied[1],
//                         rgba_unmultiplied[2],
//                     ]));

//                     // Don't edit the alpha:
//                     hsvag.a = alpha;
//                 } else {
//                     // Normal blending.
//                     *hsvag = HsvaGamma::from(Hsva::from_rgba_unmultiplied(
//                         rgba_unmultiplied[0],
//                         rgba_unmultiplied[1],
//                         rgba_unmultiplied[2],
//                         rgba_unmultiplied[3],
//                     ));
//                 }
//             }
//         }
//     }

//     let current_color_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
//     show_srgba_unmultiplied(
//         ui,
//         Hsva::from(*hsvag).to_srgba_unmultiplied(),
//         current_color_size,
//     )
//     .on_hover_text("Selected color");

//     if alpha == Alpha::BlendOrAdditive {
//         let a = &mut hsvag.a;
//         let mut additive = is_additive_alpha(*a);
//         ui.horizontal(|ui| {
//             ui.label("Blending:");
//             ui.radio_value(&mut additive, false, "Normal");
//             ui.radio_value(&mut additive, true, "Additive");

//             if additive {
//                 *a = -a.abs();
//             }

//             if !additive {
//                 *a = a.abs();
//             }
//         });
//     }

//     let opaque = HsvaGamma { a: 1.0, ..*hsvag };

//     let HsvaGamma { h, s, v, a: _ } = hsvag;

//     if false {
//         color_slider_1d(ui, s, |s| HsvaGamma { s, ..opaque }.into()).on_hover_text("Saturation");
//     }

//     if false {
//         color_slider_1d(ui, v, |v| HsvaGamma { v, ..opaque }.into()).on_hover_text("Value");
//     }

//     // color_slider_2d(ui, s, v, |s, v| HsvaGamma { s, v, ..opaque }.into());
//     color_slider_2d_grid(ui, s, v, |s, v| HsvaGamma { s, v, ..opaque }.into());

//     color_slider_1d_grid(ui, h, |h| {
//         oklch_to_color32(
//             Oklch {
//                 l: 0.75,
//                 c: 0.15,
//                 h: h * std::f32::consts::TAU,
//             },
//             "osgsoe".into(),
//         )
//     })
//     .on_hover_text("Hue");

//     let additive = is_additive_alpha(hsvag.a);

//     if alpha == Alpha::Opaque {
//         hsvag.a = 1.0;
//     } else {
//         let a = &mut hsvag.a;

//         if alpha == Alpha::OnlyBlend {
//             if is_additive_alpha(*a) {
//                 *a = 0.5; // was additive, but isn't allowed to be
//             }
//             color_slider_1d(ui, a, |a| HsvaGamma { a, ..opaque }.into()).on_hover_text("Alpha");
//         } else if !additive {
//             color_slider_1d(ui, a, |a| HsvaGamma { a, ..opaque }.into()).on_hover_text("Alpha");
//         }
//     }
// }
//
//
//

fn color_slider_2d_grid_oklch(
    ui: &mut Ui,
    l_value: &mut f32,
    c_value: &mut f32,
    hue: f32,
    color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
) -> Response {
    let desired_size = Vec2::splat(ui.spacing().slider_width);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    // ----------------------------
    // Interaction (x = chroma, y = lightness)
    // ----------------------------
    if let Some(mpos) = response.interact_pointer_pos() {
        *c_value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
        *l_value = remap_clamp(mpos.y, rect.bottom()..=rect.top(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let size = rect.size();
        let tex_id = ui.id().with(("oklch_2d_tex", (hue * 1000.0) as u32));

        let mut tex = ui
            .data(|d| d.get_temp::<Option<egui::TextureHandle>>(tex_id))
            .unwrap_or(None);

        let pixels_w = size.x as usize;
        let pixels_h = size.y as usize;

        // Fixme: Also dependent on change in l,c,h values, forn now just always rebuild
        // let rebuild = tex.is_none() || tex.as_ref().unwrap().size_vec2() != size;
        let rebuild = true;

        if rebuild {
            let mut pixels = Vec::with_capacity(pixels_w * pixels_h * 4);

            for y in (0..pixels_h).rev() {
                let lt = y as f32 / (pixels_h as f32 - 1.0); // lightness

                for x in 0..pixels_w {
                    let ct = x as f32 / (pixels_w as f32 - 1.0); // chroma

                    let (is_valid, color) = color_at(lt, ct, hue);

                    let (r, g, b, a) = color.to_tuple();

                    pixels.push(r);
                    pixels.push(g);
                    pixels.push(b);
                    pixels.push(is_valid as u8);
                }
            }

            let image = egui::ColorImage::from_rgba_unmultiplied([pixels_w, pixels_h], &pixels);

            if tex.is_none() {
                tex = Some(ui.ctx().load_texture(
                    "oklch_2d_slider",
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
        // Draw texture
        // ----------------------------
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        // ----------------------------
        // Border
        // ----------------------------
        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

        // ----------------------------
        // Handle
        // ----------------------------
        let x = lerp(rect.left()..=rect.right(), *c_value);
        let y = lerp(rect.bottom()..=rect.top(), *l_value);

        draw_inward_triangles(
            ui.painter(),
            Pos2::new(x, y),
            4.0,  // radius
            4.0,  // size
            4,    // count
            45.0, // radial-offset
            Color32::WHITE,
            Stroke {
                width: 0.5,
                color: Color32::BLACK,
            },
        );
    }

    response
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

#[derive(Clone, Copy, Debug)]
struct RadialCoordinate {
    radius: f32,
    angle: f32,
}

// #[derive(Clone, Copy, Hash, PartialEq, Eq)]
// struct RadialCoordinateCacheKey {
//     width: usize,
//     height: usize,
// }

// #[derive(Default)]
// struct RadialCoordinateCache;

// impl ComputerMut<RadialCoordinateCacheKey, Arc<[RadialCoordinate]>> for RadialCoordinateCache {
//     fn compute(&mut self, key: RadialCoordinateCacheKey) -> Arc<[RadialCoordinate]> {
//         let pixels_w = key.width;
//         let pixels_h = key.height;

//         let mut coords: Vec<RadialCoordinate> = Vec::with_capacity(pixels_w * pixels_h);

//         let radius_px = pixels_w.min(pixels_h) as f32 * 0.5;
//         let cx = pixels_w as f32 * 0.5;
//         let cy = pixels_h as f32 * 0.5;
//         for i_y in 0..pixels_h {
//             for i_x in 0..pixels_w {
//                 let dx = (i_x as f32 + 0.5) - cx;
//                 let dy = (i_y as f32 + 0.5) - cy;

//                 let x_normalized = dx / radius_px;
//                 let y_normalized = dy / radius_px;

//                 let r_squared = x_normalized * x_normalized + y_normalized * y_normalized;
//                 let r = r_squared.sqrt();

//                 let mut angle = (-y_normalized).atan2(x_normalized);
//                 if angle < 0.0 {
//                     angle += std::f32::consts::TAU;
//                 }

//                 coords.push(RadialCoordinate { radius: r, angle });
//             }
//         }
//         Arc::from(coords)
//     }
// }

#[derive(Clone)]
struct CachedPicker {
    stable_value_bits: u32,
    size: Vec2,
    viewport_bounds: [f32; 4],
    texture: egui::TextureHandle,
    polar_coordinates: Arc<[RadialCoordinate]>,
    border_positions: Arc<[Pos2]>,
    prev_fallback_color_pos: Option<Pos2>,
}

impl CachedPicker {
    fn new(
        ui: &mut egui::Ui,
        size: Vec2,
        stable_value: f32,
        invert_radial_value: bool,
        color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
    ) -> CachedPicker {
        let pixels_w = size.x as usize;
        let pixels_h = size.y as usize;
        let viewport_bounds = [-1., -1., 1., 1.];
        let polar_coordinates = CachedPicker::calculate_polar_coordinates(size, viewport_bounds);
        let pixels = CachedPicker::calculate_color_per_pixel(
            &polar_coordinates,
            &size,
            stable_value,
            invert_radial_value,
            color_at,
        );
        let image = egui::ColorImage::from_rgba_unmultiplied([pixels_w, pixels_h], &pixels);
        let texture =
            ui.ctx()
                .load_texture("oklch_radial_fixed", image, egui::TextureOptions::LINEAR);
        let border_positions = CachedPicker::calculate_border_from_pixels(size, &pixels);
        CachedPicker {
            stable_value_bits: (stable_value * 10000.) as u32,
            size: size,
            viewport_bounds,
            texture: texture,
            polar_coordinates: Arc::from(polar_coordinates),
            border_positions: Arc::from(border_positions),
            prev_fallback_color_pos: None,
        }
    }

    fn update_polar_coordinates(&mut self, size: Vec2) {
        let coords = CachedPicker::calculate_polar_coordinates(size, self.viewport_bounds);
        self.polar_coordinates = Arc::from(coords);
    }

    fn update_pixel_values_and_border_positions(
        &mut self,
        stable_value: f32,
        invert_radial_value: bool,
        color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
    ) {
        self.stable_value_bits = (stable_value * 10000.) as u32;

        let pixels_w = self.size.x as usize;
        let pixels_h = self.size.y as usize;
        let polar_coordinates = self.polar_coordinates.clone();
        let mut pixels = CachedPicker::calculate_color_per_pixel(
            &polar_coordinates,
            &self.size,
            stable_value,
            invert_radial_value,
            color_at,
        );

        let border_positions = CachedPicker::calculate_border_from_pixels(self.size, &pixels);
        self.border_positions = Arc::from(border_positions);

        let image = egui::ColorImage::from_rgba_unmultiplied([pixels_w, pixels_h], &pixels);
        self.texture.set(image, egui::TextureOptions::LINEAR);
    }

    fn update_viewport_bounds(
        response: &egui::Response,
        ui_rect: egui::Rect,
        viewport_bounds: &mut [f32; 4],
    ) -> bool {
        let width = viewport_bounds[2] - viewport_bounds[0];
        let height = viewport_bounds[3] - viewport_bounds[1];

        let mut viewport_changed = false;
        //
        // Pan
        //
        let is_pan_button_down = response.ctx.input(|i| {
            i.pointer.button_down(PointerButton::Secondary)
                || i.pointer.button_down(PointerButton::Middle)
        });
        if is_pan_button_down && response.dragged() {
            viewport_changed = true;

            let drag = response.drag_delta();

            let dx_world = drag.x / ui_rect.width() * width;
            let dy_world = drag.y / ui_rect.height() * height;

            viewport_bounds[0] -= dx_world;
            viewport_bounds[2] -= dx_world;

            viewport_bounds[1] -= dy_world;
            viewport_bounds[3] -= dy_world;
        }

        //
        // Zoom
        //
        let scroll = response.ctx.input(|i| i.raw_scroll_delta.y);

        if scroll.abs() > 0.0 {
            if let Some(mouse_pos) = response.hover_pos() {
                let nx = (mouse_pos.x - ui_rect.left()) / ui_rect.width();
                let ny = (mouse_pos.y - ui_rect.top()) / ui_rect.height();

                let mouse_world_x =
                    viewport_bounds[0] + nx * (viewport_bounds[2] - viewport_bounds[0]);

                let mouse_world_y =
                    viewport_bounds[1] + ny * (viewport_bounds[3] - viewport_bounds[1]);

                {
                    // Do not zoom if we are already zoomed in rather far
                    let zoom_factor = if scroll > 0.0 { 0.9 } else { 1.1 };

                    let width = viewport_bounds[2] - viewport_bounds[0];
                    let height = viewport_bounds[3] - viewport_bounds[1];

                    const MAX_ZOOM_FACTOR: f32 = 7.0;
                    const WORLD_SIZE: f32 = 2.0; // [-1, 1]

                    let min_width = WORLD_SIZE / MAX_ZOOM_FACTOR;
                    let min_height = WORLD_SIZE / MAX_ZOOM_FACTOR;

                    let new_width = width * zoom_factor;
                    let new_height = height * zoom_factor;

                    // Prevent further zoom-in once maximum zoom is reached.
                    if scroll > 0.0 && (new_width < min_width || new_height < min_height) {
                        // early return to guard against very deep zooms, viewport not changed
                        return viewport_changed;
                    }
                }
                viewport_changed = true;

                let zoom_factor = if scroll > 0.0 { 0.9 } else { 1.1 };

                viewport_bounds[0] =
                    mouse_world_x + (viewport_bounds[0] - mouse_world_x) * zoom_factor;

                viewport_bounds[2] =
                    mouse_world_x + (viewport_bounds[2] - mouse_world_x) * zoom_factor;

                viewport_bounds[1] =
                    mouse_world_y + (viewport_bounds[1] - mouse_world_y) * zoom_factor;

                viewport_bounds[3] =
                    mouse_world_y + (viewport_bounds[3] - mouse_world_y) * zoom_factor;
            }
        }
        if viewport_changed {
            CachedPicker::clamp_viewport(viewport_bounds)
        }
        viewport_changed
    }

    fn clamp_viewport(viewport: &mut [f32; 4]) {
        let min_world = -1.0;
        let max_world = 1.0;

        let mut min_x = viewport[0];
        let mut min_y = viewport[1];
        let mut max_x = viewport[2];
        let mut max_y = viewport[3];

        let width = max_x - min_x;
        let height = max_y - min_y;

        // -------------------------
        // 1. Clamp zoom FIRST (uniform bounds)
        // -------------------------
        let max_width = max_world - min_world;
        let max_height = max_world - min_world;

        let width = width.min(max_width);
        let height = height.min(max_height);

        // compute center BEFORE correction
        let cx = (min_x + max_x) * 0.5;
        let cy = (min_y + max_y) * 0.5;

        // rebuild viewport around center (this fixes drift)
        min_x = cx - width * 0.5;
        max_x = cx + width * 0.5;
        min_y = cy - height * 0.5;
        max_y = cy + height * 0.5;

        // -------------------------
        // 2. Clamp position (shift as a block)
        // -------------------------
        let dx_left = min_world - min_x;
        let dx_right = max_world - max_x;

        if dx_left > 0.0 {
            min_x += dx_left;
            max_x += dx_left;
        } else if dx_right < 0.0 {
            min_x += dx_right;
            max_x += dx_right;
        }

        let dy_bottom = min_world - min_y;
        let dy_top = max_world - max_y;

        if dy_bottom > 0.0 {
            min_y += dy_bottom;
            max_y += dy_bottom;
        } else if dy_top < 0.0 {
            min_y += dy_top;
            max_y += dy_top;
        }

        viewport[0] = min_x;
        viewport[1] = min_y;
        viewport[2] = max_x;
        viewport[3] = max_y;
    }

    fn calculate_polar_coordinates(size: Vec2, viewport_bounds: [f32; 4]) -> Vec<RadialCoordinate> {
        let pixels_w = size.x as usize;
        let pixels_h = size.y as usize;
        let mut coords: Vec<RadialCoordinate> = Vec::with_capacity(pixels_w * pixels_h);

        for i_y in 0..pixels_h {
            for i_x in 0..pixels_w {
                let polar_coord =
                    pixel_to_polar(i_x as f32, i_y as f32, pixels_w, pixels_h, viewport_bounds);
                coords.push(polar_coord);
            }
        }
        coords
    }

    fn calculate_border_from_pixels(size: Vec2, pixels: &Vec<u8>) -> Vec<Pos2> {
        let pixels_w = size.x as usize;
        let pixels_h = size.y as usize;
        let mut border_positions = Vec::new();
        for y in 1..pixels_h - 1 {
            for x in 1..pixels_w - 1 {
                let i = (y * pixels_w + x) * 4;

                let valid = pixels[i + 3] != 0;
                if !valid {
                    continue;
                }

                let mut neighbor_invalid = false;

                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;

                        let ni = (ny * pixels_w + nx) * 4;

                        if pixels[ni + 3] == 0 {
                            neighbor_invalid = true;
                            break;
                        }
                    }
                    if neighbor_invalid {
                        break;
                    }
                }

                if neighbor_invalid {
                    border_positions.push(Pos2 {
                        x: x as f32,
                        y: y as f32,
                    });
                }
            }
        }
        border_positions
    }

    fn calculate_color_per_pixel(
        polar_coordinates: &[RadialCoordinate],
        size: &Vec2,
        stable_value: f32,
        invert_radial_value: bool,
        color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
    ) -> Vec<u8> {
        let pixels_w = size.x as usize;
        let pixels_h = size.y as usize;

        let mut pixels = vec![0 as u8; pixels_w * pixels_h * 4];

        for i_y in 0..pixels_h {
            for i_x in 0..pixels_w {
                let i = i_y * pixels_w + i_x;
                let polar_coord = polar_coordinates[i];

                if polar_coord.radius > 1.0 {
                    continue;
                }

                let radius = if invert_radial_value {
                    (1.0 - polar_coord.radius).clamp(0.0, 1.0)
                } else {
                    polar_coord.radius.clamp(0.0, 1.0)
                };
                let (is_valid, color) = color_at(
                    polar_coord.angle / std::f32::consts::TAU,
                    radius,
                    stable_value,
                );

                if is_valid {
                    let i = (i_y * pixels_w + i_x) * 4;
                    pixels[i] = color.r();
                    pixels[i + 1] = color.g();
                    pixels[i + 2] = color.b();
                    pixels[i + 3] = color.a();
                }
            }
        }
        pixels
    }

    fn nearest_valid_border_coord(&self, radial_value: f32, angle_value: f32) -> Option<Pos2> {
        let (start_x, start_y) = polar_to_pixel(
            angle_value,
            radial_value,
            self.size.x as usize,
            self.size.y as usize,
            self.viewport_bounds,
        );
        let mut min_distance = f32::MAX;
        let mut min_distance_i = 0;
        let mut found_nothing = true;
        let border_positions = self.border_positions.clone();
        for (i, border_coord) in border_positions.iter().enumerate() {
            let distance =
                ((start_x - border_coord.x).powi(2) + (start_y - border_coord.y).powi(2)).sqrt();
            if distance < min_distance {
                min_distance_i = i;
                min_distance = distance;
                found_nothing = false;
            }
        }
        if found_nothing {
            return None;
        } else {
            return Some(border_positions[min_distance_i]);
        }
    }

    fn nearest_valid_color(
        &self,
        angle: f32,
        radius: f32,
        invert_radial_value: bool,
        color_at: impl Fn(f32, f32, f32) -> (bool, Color32), // TODO: move to self as function pointer
    ) -> Color32 {
        // Here we could sample the pixels vec, but we can also determine the color
        // based on coordinates. This makes the function less pleasant to use, as we
        // have to supply the color_at closure, but we would then not need to save
        // a copy of the texture in RAM.
        let stable_value = self.stable_value_bits as f32 / 10000.;

        // If color exists at provided values, return the color
        let r = if invert_radial_value {
            (1.0 - radius).clamp(0.0, 1.0)
        } else {
            radius.clamp(0.0, 1.0)
        };
        let (is_valid, color) = color_at(angle / std::f32::consts::TAU, r, stable_value);
        if is_valid {
            return color;
        }

        // If color does not exist, return nearest color
        let sample_pos = self.nearest_valid_border_coord(radius, angle);
        let fallback_color = Color32::BLACK; // Take a look at the values where no possibilities left, what is the last color to disappear? Use that!
        let color = match sample_pos {
            None => fallback_color,
            Some(sample_pos) => {
                let polar_coord = pixel_to_polar(
                    sample_pos.x,
                    sample_pos.y,
                    self.size.x as usize,
                    self.size.y as usize,
                    self.viewport_bounds,
                );
                let radius = if invert_radial_value {
                    (1.0 - polar_coord.radius).clamp(0.0, 1.0)
                } else {
                    polar_coord.radius.clamp(0.0, 1.0)
                };

                let (_, color) = color_at(
                    polar_coord.angle / std::f32::consts::TAU,
                    radius,
                    stable_value,
                );
                color
            }
        };
        color
    }
}

// #[derive(Clone, Copy, Hash, PartialEq, Eq)]
// struct RadialBorderCoordinateCacheKey {
//     width: usize,
//     height: usize,
//     stable_value: u32,
//     ui_identifyable_label: &str,
// }

// #[derive(Default)]
// struct RadialBorderCoordinateCache;

// impl ComputerMut<RadialBorderCoordinateCacheKey, Arc<[RadialCoordinate]>> for RadialBorderCoordinateCache {
//     fn compute(&mut self, key: RadialBorderCoordinateCacheKey) -> Arc<[RadialCoordinate]> {
//         let pixels_w = key.width;
//         let pixels_h = key.height;
//         let stable_value = key.stable_value as f32 / 10000.;

//         // TIMO LEFT HERE:
//         // I somehow need to have access to pixels. This is stored as tex handle,
//         // not as vec. I can try to also store a vec, but even then I'd need to access
//         // the cache. Maybe consider a OnceCell or other global?

//         let mut border_positions = Vec::new();
//         for y in 1..pixels_h - 1 {
//             for x in 1..pixels_h - 1 {
//                 let i = (y * pixels_w + x) * 4;

//                 let valid = pixels[i + 3] != 0;
//                 if !valid {
//                     continue;
//                 }

//                 let mut neighbor_invalid = false;

//                 for dy in -1i32..=1 {
//                     for dx in -1i32..=1 {
//                         if dx == 0 && dy == 0 {
//                             continue;
//                         }

//                         let nx = (x as i32 + dx) as usize;
//                         let ny = (y as i32 + dy) as usize;

//                         let ni = (ny * pixels_w + nx) * 4;

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
//                     border_positions.push((x,y));
//                 }
//             }
//         }
//         Arc::from(border_positions)
//     }
// }

pub fn polar_to_pixel(
    angle: f32,
    radius: f32,
    width: usize,
    height: usize,
    viewport_bounds: [f32; 4], // [min_x, min_y, max_x, max_y]
) -> (f32, f32) {
    // -----------------------------
    // 1. polar → WORLD space
    // -----------------------------
    let world_x = radius * angle.cos();
    let world_y = -radius * angle.sin();

    // -----------------------------
    // 2. WORLD → normalized viewport space [0,1]
    // -----------------------------
    let vx = (world_x - viewport_bounds[0]) / (viewport_bounds[2] - viewport_bounds[0]);

    let vy = (world_y - viewport_bounds[1]) / (viewport_bounds[3] - viewport_bounds[1]);

    // -----------------------------
    // 3. normalized → pixel space
    // -----------------------------
    let x = vx * width as f32 - 0.5;
    let y = vy * height as f32 - 0.5;

    (x, y)
}

pub fn pixel_to_polar(
    x: f32,
    y: f32,
    width: usize,
    height: usize,
    viewport_bounds: [f32; 4], // [min_x, min_y, max_x, max_y]
) -> RadialCoordinate {
    let nx = (x + 0.5) / width as f32;
    let ny = (y + 0.5) / height as f32;

    let world_x = viewport_bounds[0] + nx * (viewport_bounds[2] - viewport_bounds[0]);

    let world_y = viewport_bounds[1] + ny * (viewport_bounds[3] - viewport_bounds[1]);

    let r_squared = world_x * world_x + world_y * world_y;
    let r = r_squared.sqrt();

    // Invert Y for correct hue rotation direction
    let mut angle = fast_math::atan2(-world_y, world_x);
    if angle < 0.0 {
        angle += std::f32::consts::TAU;
    }

    RadialCoordinate { radius: r, angle }
}

// pub fn nearest_lch_color(
//     pixels: &[u8],
//     width: usize,
//     height: usize,
//     start_x: usize,
//     start_y: usize,
//     mut to_lch: impl Fn(usize, usize) -> Oklch,
// ) -> Option<Oklch> {
//     // Helper: check if pixel is valid
//     let is_valid = |x: usize, y: usize| -> bool {
//         let i = (y * width + x) * 4;
//         pixels[i + 3] != 0
//     };

//     // Clamp start position
//     let start_x = start_x.min(width.saturating_sub(1));
//     let start_y = start_y.min(height.saturating_sub(1));

//     // If starting point is valid, return immediately
//     if is_valid(start_x, start_y) {
//         return Some(to_lch(start_x, start_y));
//     }

//     // Ring search outward
//     let max_radius = width.max(height); // Anythin outside r==1 is anyway not valid color space

//     for r in 1..max_radius {
//         let x0 = start_x.saturating_sub(r);
//         let x1 = (start_x + r).min(width - 1);
//         let y0 = start_y.saturating_sub(r);
//         let y1 = (start_y + r).min(height - 1);

//         // top + bottom edges
//         for x in x0..=x1 {
//             if y0 < height && is_valid(x, y0) {
//                 return Some(to_lch(x, y0));
//             }
//             if y1 < height && is_valid(x, y1) {
//                 return Some(to_lch(x, y1));
//             }
//         }

//         // left + right edges (excluding corners already checked)
//         for y in (y0 + 1)..y1 {
//             if x0 < width && is_valid(x0, y) {
//                 return Some(to_lch(x0, y));
//             }
//             if x1 < width && is_valid(x1, y) {
//                 return Some(to_lch(x1, y));
//             }
//         }
//     }

//     None
// }

struct Pos {
    x: usize,
    y: usize,
}

pub fn nearest_valid_coord(
    pixels: &[u8],
    width: usize,
    height: usize,
    radial_value: f32,
    angle_value: f32,
    viewport_bounds: [f32; 4],
) -> Option<Pos> {
    let (start_x, start_y) =
        polar_to_pixel(angle_value, radial_value, width, height, viewport_bounds);
    let start_x = start_x as usize;
    let start_y = start_y as usize;

    let idx = |x: usize, y: usize| (y * width + x) * 4;

    let is_valid = |x: usize, y: usize| -> bool { pixels[idx(x, y) + 3] != 0 };

    let start_x = start_x.min(width).max(0);
    let start_y = start_y.min(height).max(0);

    // early exit
    if is_valid(start_x, start_y) {
        return Some(Pos {
            x: start_x,
            y: start_y,
        });
    }

    // Diagonal distance as max search distance
    let max_r = (width * width + height * height).isqrt();

    for r in 1..max_r {
        let x0 = start_x.saturating_sub(r);
        let x1 = (start_x + r).min(width - 1);
        let y0 = start_y.saturating_sub(r);
        let y1 = (start_y + r).min(height - 1);

        // top + bottom edges
        for x in x0..=x1 {
            if y0 < height && is_valid(x, y0) {
                return Some(Pos { x, y: y0 });
            }
            if y1 < height && is_valid(x, y1) {
                return Some(Pos { x, y: y1 });
            }
        }

        // left + right edges
        for y in (y0 + 1)..y1 {
            if x0 < width && is_valid(x0, y) {
                return Some(Pos { x: x0, y });
            }
            if x1 < width && is_valid(x1, y) {
                return Some(Pos { x: x1, y });
            }
        }
    }

    None
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

    // interpret (u,v) as your 2D parameter space
    // NOTE: this assumes:
    // u -> angle
    // v -> radius (or vice versa depending on your design)

    let angle = u;
    let radius = if invert_radial_value { 1.0 - v } else { v };

    to_lch(angle, radius, stable_value)
}

fn color_slider_2d_radial(
    ui: &mut Ui,
    stable_value: f32,
    angle_value: &mut f32,
    radial_value: &mut f32,
    invert_radial_value: bool,
    color_at: impl Fn(f32, f32, f32) -> (bool, Color32),
    ui_identifyable_label: &str,
) -> (Response, bool) {
    let desired_size = Vec2::splat(ui.spacing().slider_width);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    let size = response.rect.size();
    let ui_center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.5;

    // Get picker info
    let picker_id = ui.id().with(ui_identifyable_label);
    let mut picker_cache = ui.data(|d| d.get_temp::<CachedPicker>(picker_id));

    let mut picker_info = match picker_cache {
        None => CachedPicker::new(ui, size, stable_value, invert_radial_value, &color_at),
        Some(picker_info) => picker_info,
    };

    // Mouse pointer interaction
    let is_lmb_down = response
        .ctx
        .input(|i| i.pointer.button_down(PointerButton::Primary));
    if is_lmb_down {
        if let Some(mpos) = response.interact_pointer_pos() {
            // 1. normalize mouse in UI space [0,1]
            let nx = (mpos.x - response.rect.left()) / size.x;
            let ny = (mpos.y - response.rect.top()) / size.y;

            // 2. convert to WORLD space using viewport
            let viewport_bounds = picker_info.viewport_bounds;
            let vx = viewport_bounds[0] + nx * (viewport_bounds[2] - viewport_bounds[0]);

            let vy = viewport_bounds[1] + ny * (viewport_bounds[3] - viewport_bounds[1]);

            // 3. convert world → polar
            let r = (vx * vx + vy * vy).sqrt().clamp(0.0, 1.0);

            let angle = (-vy).atan2(vx);

            let mut h = angle / std::f32::consts::TAU;
            if h < 0.0 {
                h += 1.0;
            }

            *angle_value = h.clamp(0.0, 1.0);
            *radial_value = if invert_radial_value { 1.0 - r } else { r };
        }
    }

    let mut viewport_contains_picked_color = false; // default in case rect not visible

    if ui.is_rect_visible(rect) {
        let size = rect.size();

        // Radial lines on background
        let painter = ui.painter().with_clip_rect(response.rect);
        let viewport_bounds = picker_info.viewport_bounds;
        let cx = response.rect.left()
            + ((0.0 - viewport_bounds[0]) / (viewport_bounds[2] - viewport_bounds[0]))
                * response.rect.width();

        let cy = response.rect.top()
            + ((0.0 - viewport_bounds[1]) / (viewport_bounds[3] - viewport_bounds[1]))
                * response.rect.height();

        let world_to_screen = response.rect.width() / (viewport_bounds[2] - viewport_bounds[0]);

        for i in 0..6 {
            let world_r = 0.01 + 0.15 * i as f32;
            let screen_r = world_r * world_to_screen;

            painter.circle(
                Pos2::new(cx, cy),
                screen_r,
                Color32::TRANSPARENT,
                egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(200, 200, 200, 100)),
            );
        }

        let stable_value_bits = (stable_value * 10000.) as u32;
        let stable_value_changed = picker_info.stable_value_bits != stable_value_bits;
        let size_changed = picker_info.size != size;
        if size_changed {
            picker_info.update_polar_coordinates(size);
        }
        if size_changed || stable_value_changed {
            picker_info.update_pixel_values_and_border_positions(
                stable_value,
                invert_radial_value,
                &color_at,
            );
        }

        let viewport_changed =
            CachedPicker::update_viewport_bounds(&response, rect, &mut picker_info.viewport_bounds);
        if viewport_changed {
            // picker_info.viewport_bounds = new_viewport_bounds;
            picker_info.update_polar_coordinates(size);
            picker_info.update_pixel_values_and_border_positions(
                stable_value,
                invert_radial_value,
                &color_at,
            );
        }

        let sample_radius = if invert_radial_value {
            (1.0 - *radial_value).clamp(0.0, 1.0)
        } else {
            (*radial_value).clamp(0.0, 1.0)
        };

        let tex = picker_info.texture.clone();
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        let visuals = ui.style().interact(&response);
        painter.rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

        // Handle
        let angle = *angle_value * std::f32::consts::TAU;

        let r = if invert_radial_value {
            1.0 - *radial_value
        } else {
            *radial_value
        };

        // -----------------------------
        // 1. Polar → WORLD space
        // -----------------------------
        let wx = angle.cos() * r;
        let wy = -angle.sin() * r;

        // -----------------------------
        // 2. WORLD → SCREEN space
        // -----------------------------
        let size = ui_center * 2.0; // or response.rect.size()

        let viewport_bounds = picker_info.viewport_bounds;
        let nx = (wx - viewport_bounds[0]) / (viewport_bounds[2] - viewport_bounds[0]);

        let ny = (wy - viewport_bounds[1]) / (viewport_bounds[3] - viewport_bounds[1]);

        let x = response.rect.left() + nx * response.rect.width();
        let y = response.rect.top() + ny * response.rect.height();

        let marker_pos = Pos2::new(x, y);
        viewport_contains_picked_color = response.rect.contains(marker_pos);

        // -----------------------------
        // 3. Color sampling MUST also use world-space mapping
        // -----------------------------
        let (is_valid, color) =
            color_at(angle / std::f32::consts::TAU, *radial_value, stable_value);

        let (indicator_color, marker_angle) = if is_valid {
            (Color32::WHITE, 45.)
        } else {
            (Color32::WHITE, 0.)
        };

        draw_inward_triangles(
            &painter,
            marker_pos,
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
        // Draw indicator at loction of fallback color
        if !is_valid {
            let fallback_color_pos = if viewport_contains_picked_color {
                let fallback_color_pos = picker_info.nearest_valid_border_coord(r, angle);
                picker_info.prev_fallback_color_pos = fallback_color_pos;
                dbg!(fallback_color_pos);
                fallback_color_pos
            } else {
                dbg!(picker_info.prev_fallback_color_pos);
                picker_info.prev_fallback_color_pos
            };
            // let color = picker_info.nearest_valid_color(angle, r, invert_radial_value, &color_at);
            if let Some(pos) = fallback_color_pos {
                draw_inward_triangles(
                    &painter,
                    pos + rect.min.to_vec2(),
                    3.0,  // radius
                    3.0,  // size
                    4,    // count
                    45.0, // radial-offset
                    Color32::WHITE,
                    Stroke {
                        width: 0.5,
                        color: Color32::BLACK,
                    },
                );
            }
        }

        ui.data_mut(|d| {
            d.insert_temp(picker_id, picker_info.clone());
        });
    }

    if response.is_pointer_button_down_on() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::None);
    }

    (response, viewport_contains_picked_color)
}

#[derive(Debug, Clone)]
enum OKLabPlotType {
    StableLightness,
    StableChroma,
    StableHue,
}

fn color_picker_oklch_2d(ui: &mut Ui, lch: &mut Oklch, alpha: &mut f32) -> (bool, Color32) {
    let current_color_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    let mut marker_in_viewport = false;

    let state_id = ui.id().with("oklch_ui_state");

    // Todo: base default on color value
    // Todo: If the color changed beneath us, reset state to match new color
    let mut state: OklchUiState =
        ui.data(|d| d.get_temp::<OklchUiState>(state_id))
            .unwrap_or(OklchUiState {
                l: 0.5,
                c: 0.5,
                h: 0.5,
                previous_color: lch.clone(),
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
            // state.l = lch.l;
            // state.c = lch.c / 0.325;
            // state.h = lch.h / std::f32::consts::TAU;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableChroma), "Hue/Lightness")
            .clicked()
        {
            tab = OKLabPlotType::StableChroma;
            tab_switched = true;
            // state.l = lch.l;
            // state.c = lch.c / 0.325;
            // state.h = lch.h / std::f32::consts::TAU;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableLightness), "Hue/Chroma")
            .clicked()
        {
            tab = OKLabPlotType::StableLightness;
            tab_switched = true;
            // state.l = lch.l;
            // state.c = lch.c / 0.325;
            // state.h = lch.h / std::f32::consts::TAU;
        }
    });

    ui.horizontal(|ui| {
        input_type_button_ui(ui);

        // TODO: get nearest color! nocheckin
        let (is_valid, color) = oklch_to_color32(Oklch {
            l,
            c: c * 0.325,
            h: h * std::f32::consts::TAU,
        });

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

    let (_, display_color) = oklch_to_color32(lch.clone()); // technically the color from previous frame, but I want to display this above the 2d picker
    show_srgba_unmultiplied(ui, display_color.to_array(), current_color_size)
        .on_hover_text("Selected color");

    let new_color = match tab {
        OKLabPlotType::StableHue => {
            color_slider_2d_grid_oklch(
                ui,
                &mut state.l,
                &mut state.c,
                state.h * std::f32::consts::TAU,
                |l, c, h| {
                    oklch_to_color32(Oklch {
                        l,
                        c: c * 0.325, // match your chroma scaling
                        h,
                    })
                },
            );
            Color32::CYAN
        }
        OKLabPlotType::StableChroma => {
            let ui_identifyable_label = "StableChromeColorPicker";
            color_slider_2d_radial(
                ui,
                state.c,
                &mut state.h,
                &mut state.l,
                true,
                |angle, radian, stable| {
                    oklch_to_color32(Oklch {
                        l: radian,
                        c: stable * 0.325,
                        h: angle * std::f32::consts::TAU,
                    })
                },
                ui_identifyable_label,
            );
            let picker_id = ui.id().with(ui_identifyable_label);
            let picker_info = ui.data(|d| d.get_temp::<CachedPicker>(picker_id).unwrap()); // Has to have been set by color_slider_2d_radial.
            let picked_color = picker_info.nearest_valid_color(
                state.h * std::f32::consts::TAU,
                (1.0 - state.l).clamp(0.0, 1.0),
                true,
                |angle, radian, stable| {
                    oklch_to_color32(Oklch {
                        l: radian,
                        c: stable * 0.325,
                        h: angle * std::f32::consts::TAU,
                    })
                },
            );
            picked_color
        }
        OKLabPlotType::StableLightness => {
            let ui_identifyable_label = "StableLightnessColorPicker";
            let (_response, marker_in_viewport_) = color_slider_2d_radial(
                ui,
                state.l,
                &mut state.h,
                &mut state.c,
                false,
                |angle, radian, stable| {
                    oklch_to_color32(Oklch {
                        l: stable,
                        c: radian * 0.325,
                        h: angle * std::f32::consts::TAU,
                    })
                },
                ui_identifyable_label,
            );
            marker_in_viewport = marker_in_viewport_;
            let picker_id = ui.id().with(ui_identifyable_label);
            let picker_info = ui.data(|d| d.get_temp::<CachedPicker>(picker_id).unwrap()); // Has to have been set by color_slider_2d_radial.
            // if marker_in_viewport {
            let picked_color = picker_info.nearest_valid_color(
                state.h * std::f32::consts::TAU,
                (state.c).clamp(0.0, 1.0),
                false,
                |angle, radian, stable| {
                    oklch_to_color32(Oklch {
                        l: stable,
                        c: radian * 0.325,
                        h: angle * std::f32::consts::TAU,
                    })
                },
            );
            picked_color
            // } else {
            //     println!("OUTSIDE!");
            //     // Color outside of viewport, return previous color
            //     // let (_, picled_color) = oklch_to_color32(prev_lch);
            //     picled_color
            // }
        }
    };
    // show_srgba_unmultiplied(ui, new_color.to_array(), current_color_size)
    //     .on_hover_text("Selected color");

    // --- Lightness slider (0..1) ---
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.l,
            lch.l,
            display_color,
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
            display_color,
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
        let (_, bg_color) = oklch_to_color32(Oklch {
            l: 0.750,
            c: 0.127,
            h: h_mut * std::f32::consts::TAU,
        });
        let bg_color = bg_color.blend(Color32::from_black_alpha(40));
        let color = Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 210);
        color
    };
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.h,
            lch.h / std::f32::consts::TAU,
            display_color,
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

    (marker_in_viewport, new_color)
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

// /// Shows a color picker where the user can change the given [`Hsva`] color.
// ///
// /// Returns `true` on change.
// pub fn color_picker_hsva_2d(ui: &mut Ui, hsva: &mut Hsva, alpha: Alpha) -> bool {
//     let mut hsvag = HsvaGamma::from(*hsva);
//     ui.vertical(|ui| {
//         color_picker_hsvag_2d(ui, &mut hsvag, alpha);
//     });
//     let new_hasva = Hsva::from(hsvag);
//     if *hsva == new_hasva {
//         false
//     } else {
//         *hsva = new_hasva;
//         true
//     }
// }

// /// Shows a color picker where the user can change the given [`Color32`] color.
// ///
// /// Returns `true` on change.
// pub fn color_picker_color32(ui: &mut Ui, srgba: &mut Color32, alpha: Alpha) -> bool {
//     // let mut hsva = color_cache_get(ui.ctx(), *srgba);
//     let mut hsva: Hsva =
//         Hsva::from_srgba_premultiplied([srgba.r(), srgba.g(), srgba.b(), srgba.a()]);
//     let changed = color_picker_hsva_2d(ui, &mut hsva, alpha);
//     *srgba = Color32::from(hsva);
//     changed
// }

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
    let prev_color = color.clone();
    let mut new_lch = color32_to_oklch(*color);
    let mut new_alpha = color.a() as f32 / 255.0;

    let (marker_in_viewport, new_color) = color_picker_oklch_2d(ui, &mut new_lch, &mut new_alpha);

    if marker_in_viewport {
        *color = new_color;
    }
    // *color = Color32::from_rgba_unmultiplied(
    //     (srgb.r * 255.0).round() as u8,
    //     (srgb.g * 255.0).round() as u8,
    //     (srgb.b * 255.0).round() as u8,
    //     (new_alpha * 255.0).round() as u8,
    // );
    if *color == prev_color { false } else { true }
}

// pub fn color_edit_button_hsva(ui: &mut Ui, hsva: &mut Hsva, alpha: Alpha) -> Response {
//     let popup_id = ui.auto_id_with("popup");
//     let open = Popup::is_id_open(ui.ctx(), popup_id);
//     let mut button_response = color_button(ui, hsva.to_srgba_unmultiplied(), open);
//     if ui.style().explanation_tooltips {
//         button_response = button_response.on_hover_text("Click to edit color");
//     }

//     const COLOR_SLIDER_WIDTH: f32 = 275.0;

//     Popup::menu(&button_response)
//         .id(popup_id)
//         .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
//         .show(|ui| {
//             ui.spacing_mut().slider_width = COLOR_SLIDER_WIDTH;
//             if color_picker_hsva_2d(ui, hsva, alpha) {
//                 button_response.mark_changed();
//             }
//         });

//     button_response
// }

// /// Shows a button with the given color.
// /// If the user clicks the button, a full color picker is shown.
// pub fn color_edit_button_srgba(ui: &mut Ui, srgba: &mut Color32, alpha: Alpha) -> Response {
//     let mut hsva: Hsva =
//         Hsva::from_srgba_premultiplied([srgba.r(), srgba.g(), srgba.b(), srgba.a()]);
//     let response = color_edit_button_hsva(ui, &mut hsva, alpha);
//     *srgba = Color32::from(hsva);
//     response
// }

// /// Shows a button with the given color.
// /// If the user clicks the button, a full color picker is shown.
// /// The given color is in `sRGB` space.
// pub fn color_edit_button_srgb(ui: &mut Ui, srgb: &mut [u8; 3]) -> Response {
//     let mut srgba = Color32::from_rgb(srgb[0], srgb[1], srgb[2]);
//     let response = color_edit_button_srgba(ui, &mut srgba, Alpha::Opaque);
//     srgb[0] = srgba[0];
//     srgb[1] = srgba[1];
//     srgb[2] = srgba[2];
//     response
// }

// /// Shows a button with the given color.
// /// If the user clicks the button, a full color picker is shown.
// pub fn color_edit_button_rgba(ui: &mut Ui, rgba: &mut Rgba, alpha: Alpha) -> Response {
//     let mut hsva: Hsva = Hsva::from_srgba_premultiplied([srgba.r(), srgba.g(), srgba.b(), srgba.a()]);
//     let response = color_edit_button_hsva(ui, &mut hsva, alpha);
//     *rgba = Rgba::from(hsva);
//     response
// }

// /// Shows a button with the given color.
// /// If the user clicks the button, a full color picker is shown.
// pub fn color_edit_button_rgb(ui: &mut Ui, rgb: &mut [f32; 3]) -> Response {
//     let mut rgba = Rgba::from_rgb(rgb[0], rgb[1], rgb[2]);
//     let response = color_edit_button_rgba(ui, &mut rgba, Alpha::Opaque);
//     rgb[0] = rgba[0];
//     rgb[1] = rgba[1];
//     rgb[2] = rgba[2];
//     response
// }

// // To ensure we keep hue slider when `srgba` is gray we store the full [`Hsva`] in a cache:
// fn color_cache_get(ctx: &Context, rgba: impl Into<Rgba>) -> Hsva {
//     let rgba = rgba.into();
//     use_color_cache(ctx, |cc| cc.get(&rgba).copied()).unwrap_or_else(|| Hsva::from(rgba))
// }

// // To ensure we keep hue slider when `srgba` is gray we store the full [`Hsva`] in a cache:
// fn color_cache_set(ctx: &Context, rgba: impl Into<Rgba>, hsva: Hsva) {
//     let rgba = rgba.into();
//     use_color_cache(ctx, |cc| cc.set(rgba, hsva));
// }

// To ensure we keep hue slider when `srgba` is gray we store the full [`Hsva`] in a cache:
// fn use_color_cache<R>(ctx: &Context, f: impl FnOnce(&mut FixedCache<Rgba, Hsva>) -> R) -> R {
//     ctx.data_mut(|d| f(d.get_temp_mut_or_default(Id::NULL)))
// }
