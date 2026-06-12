//! Color picker widgets.

// use egui::util::fixed_cache::FixedCache;
mod conversions;
use crate::conversions::*;
use egui::{
    Context, DragValue, Id, Painter, Popup, PopupCloseBehavior, Pos2, Response, Sense,
    TextureHandle, Ui, Widget as _, WidgetInfo, WidgetType, epaint, lerp, remap_clamp,
};
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

fn background_checkers(painter: &Painter, rect: Rect) {
    let rect = rect.shrink(0.5); // Small hack to avoid the checkers from peeking through the sides
    if !rect.is_positive() {
        return;
    }

    let dark_color = Color32::from_gray(32);
    let bright_color = Color32::from_gray(128);

    let checker_size = Vec2::splat(rect.height() / 2.0);
    let n = (rect.width() / checker_size.x).round() as u32;

    let mut mesh = Mesh::default();
    mesh.add_colored_rect(rect, dark_color);

    let mut top = true;
    for i in 0..n {
        let x = lerp(rect.left()..=rect.right(), i as f32 / (n as f32));
        let small_rect = if top {
            Rect::from_min_size(pos2(x, rect.top()), checker_size)
        } else {
            Rect::from_min_size(pos2(x, rect.center().y), checker_size)
        };
        mesh.add_colored_rect(small_rect, bright_color);
        top = !top;
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
        background_checkers(painter, rect);

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
        background_checkers(painter, rect);
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

        background_checkers(ui.painter(), rect); // for alpha:

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
    color_value_normalized: f32,
    color_at: impl Fn(f32) -> Option<Color32>,
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

        background_checkers(ui.painter(), rect);

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

                    match color_at(t) {
                        None => {
                            pixels.push(0);
                            pixels.push(0);
                            pixels.push(0);
                            pixels.push(0);
                        }
                        Some(color) => {
                            pixels.push(color.r());
                            pixels.push(color.g());
                            pixels.push(color.b());
                            pixels.push(color.a());
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
        let r = rect.height() / 4.0;
        let picked_color = match color_at(*slider_value) {
            Some(color) => color,
            None => Color32::from_rgba_premultiplied(0, 0, 0, 0),
        };
        let x = lerp(rect.left()..=rect.right(), *slider_value);
        ui.painter().add(Shape::convex_polygon(
            vec![
                pos2(x, rect.center().y),
                pos2(x + r, rect.bottom()),
                pos2(x - r, rect.bottom()),
            ],
            picked_color,
            Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        ));
        // Show current color with marker (snap to nearest existing value)
        let x = lerp(
            rect.left()..=rect.right(),
            // color_value_normalized * (rect.right() - rect.left()),
            *slider_value * (rect.right() - rect.left()),
            // color_value_normalized,
        );
        ui.painter().add(Shape::convex_polygon(
            vec![
                pos2(x, rect.center().y),
                pos2(x + r, rect.bottom()),
                pos2(x - r, rect.bottom()),
            ],
            picked_color,
            Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        ));
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

fn oklch_to_color32(c: Oklch) -> Color32 {
    let srgb = linear_srgb_to_srgb(oklab_to_linear_srgb(oklch_to_oklab(c)));

    Color32::from_rgb(
        (srgb.r * 255.) as u8,
        (srgb.g * 255.) as u8,
        (srgb.b * 255.) as u8,
    )
}

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
    color_at: impl Fn(f32, f32, f32) -> Option<Color32>,
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

                    let color = color_at(lt, ct, hue);

                    let (r, g, b, a) = match color {
                        Some(c) => (c.r(), c.g(), c.b(), c.a()),
                        None => (0, 0, 0, 0), // invalid → fully transparent
                    };

                    pixels.push(r);
                    pixels.push(g);
                    pixels.push(b);
                    pixels.push(a);
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

        let picked_color = color_at(*l_value, *c_value, hue).unwrap_or(Color32::TRANSPARENT);

        ui.painter().circle(
            Pos2::new(x, y),
            rect.width() / 12.0,
            picked_color,
            egui::Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        );
    }

    response
}

fn color_slider_2d_grid_oklch_radial(
    ui: &mut Ui,
    l_value: &mut f32,
    hue_value: &mut f32, // 0..1
    chroma: f32,
    color_at: impl Fn(f32, f32, f32) -> Option<Color32>,
) -> Response {
    let desired_size = Vec2::splat(ui.spacing().slider_width);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.5;

    // ----------------------------
    // Interaction (FIXED polar mapping)
    // ----------------------------
    if let Some(mpos) = response.interact_pointer_pos() {
        let v = (mpos - center) / radius;

        let r = v.length().clamp(0.0, 1.0);

        // IMPORTANT: invert Y for correct hue rotation direction
        let angle = (-v.y).atan2(v.x);

        let mut h = angle / std::f32::consts::TAU;
        if h < 0.0 {
            h += 1.0;
        }

        *hue_value = h.clamp(0.0, 1.0);

        // center = light, edge = dark (as you requested)
        *l_value = (1.0 - r).clamp(0.0, 1.0);
    }

    if ui.is_rect_visible(rect) {
        let size = rect.size();
        let tex_id = ui
            .id()
            .with(("oklch_radial_tex_stable_chroma", (chroma * 1000.0) as u32));

        let mut tex = ui
            .data(|d| d.get_temp::<Option<egui::TextureHandle>>(tex_id))
            .unwrap_or(None);

        let pixels_w = size.x as usize;
        let pixels_h = size.y as usize;

        let rebuild = true;

        if rebuild {
            let mut pixels = Vec::with_capacity(pixels_w * pixels_h * 4);

            for y in 0..pixels_h {
                for x in 0..pixels_w {
                    let nx = (x as f32 / (pixels_w as f32 - 1.0)) * 2.0 - 1.0;
                    let ny = (y as f32 / (pixels_h as f32 - 1.0)) * 2.0 - 1.0;

                    let v = Vec2::new(nx, ny);
                    let r = v.length();

                    if r > 1.0 {
                        pixels.extend([0, 0, 0, 0]);
                        continue;
                    }

                    let mut angle = (-ny).atan2(nx);
                    if angle < 0.0 {
                        angle += std::f32::consts::TAU;
                    }

                    let h = angle / std::f32::consts::TAU;
                    let l = 1.0 - r;

                    let color = color_at(l, chroma, h);

                    let (r8, g8, b8, a8) = match color {
                        Some(c) => (c.r(), c.g(), c.b(), c.a()),
                        None => (0, 0, 0, 0),
                    };

                    pixels.push(r8);
                    pixels.push(g8);
                    pixels.push(b8);
                    pixels.push(a8);
                }
            }

            let image = egui::ColorImage::from_rgba_unmultiplied([pixels_w, pixels_h], &pixels);

            if tex.is_none() {
                tex = Some(ui.ctx().load_texture(
                    "oklch_radial_fixed",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }

            ui.data_mut(|d| {
                d.insert_temp(tex_id, tex.clone());
            });
        }

        let tex = tex.as_ref().unwrap();

        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

        // ----------------------------
        // Handle (FIXED)
        // ----------------------------
        let hue = *hue_value * std::f32::consts::TAU;

        let r = (1.0 - *l_value).clamp(0.0, 1.0);

        let x = center.x + hue.cos() * r * radius;
        let y = center.y - hue.sin() * r * radius;

        let picked_color = color_at(*l_value, chroma, *hue_value).unwrap_or(Color32::TRANSPARENT);

        ui.painter().circle(
            Pos2::new(x, y),
            rect.width() / 14.0,
            picked_color,
            egui::Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        );
    }

    response
}

fn color_slider_2d_grid_oklch_radial_2(
    ui: &mut Ui,
    c_value: &mut f32,
    hue_value: &mut f32, // 0..1
    lightness: f32,
    color_at: impl Fn(f32, f32, f32) -> Option<Color32>,
) -> Response {
    let desired_size = Vec2::splat(ui.spacing().slider_width);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.5;

    // ----------------------------
    // Interaction (FIXED polar mapping)
    // ----------------------------
    if let Some(mpos) = response.interact_pointer_pos() {
        let v = (mpos - center) / radius;

        let r = v.length().clamp(0.0, 1.0);

        // IMPORTANT: invert Y for correct hue rotation direction
        let angle = (-v.y).atan2(v.x);

        let mut h = angle / std::f32::consts::TAU;
        if h < 0.0 {
            h += 1.0;
        }

        *hue_value = h.clamp(0.0, 1.0);

        // center = light, edge = dark (as you requested)
        *c_value = r.clamp(0.0, 1.0);
    }

    if ui.is_rect_visible(rect) {
        let size = rect.size();
        let tex_id = ui.id().with((
            "oklch_radial_tex_stable_lightness",
            (lightness * 1000.0) as u32,
        ));

        let mut tex = ui
            .data(|d| d.get_temp::<Option<egui::TextureHandle>>(tex_id))
            .unwrap_or(None);

        let pixels_w = size.x as usize;
        let pixels_h = size.y as usize;

        let rebuild = true;

        if rebuild {
            let mut pixels = Vec::with_capacity(pixels_w * pixels_h * 4);

            for y in 0..pixels_h {
                for x in 0..pixels_w {
                    let nx = (x as f32 / (pixels_w as f32 - 1.0)) * 2.0 - 1.0;
                    let ny = (y as f32 / (pixels_h as f32 - 1.0)) * 2.0 - 1.0;

                    let v = Vec2::new(nx, ny);
                    let r = v.length();

                    if r > 1.0 {
                        pixels.extend([0, 0, 0, 0]);
                        continue;
                    }

                    let mut angle = (-ny).atan2(nx);
                    if angle < 0.0 {
                        angle += std::f32::consts::TAU;
                    }

                    let h = angle / std::f32::consts::TAU;
                    let c = r; // / 0.325;

                    let color = color_at(lightness, c, h);

                    let (r8, g8, b8, a8) = match color {
                        Some(c) => (c.r(), c.g(), c.b(), c.a()),
                        None => (0, 0, 0, 0),
                    };

                    pixels.push(r8);
                    pixels.push(g8);
                    pixels.push(b8);
                    pixels.push(a8);
                }
            }

            let image = egui::ColorImage::from_rgba_unmultiplied([pixels_w, pixels_h], &pixels);

            if tex.is_none() {
                tex = Some(ui.ctx().load_texture(
                    "oklch_radial_fixed",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }

            ui.data_mut(|d| {
                d.insert_temp(tex_id, tex.clone());
            });
        }

        let tex = tex.as_ref().unwrap();

        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

        // -------
        // Handle
        // -------
        let hue = *hue_value * std::f32::consts::TAU;

        // let r = (1.0 - (*c_value)).clamp(0.0, 1.0);
        let r = (*c_value).clamp(0.0, 1.0);
        // let r = (1.0 - (*c_value) / 0.325).clamp(0.0, 1.0);

        let x = center.x + hue.cos() * r * radius;
        let y = center.y - hue.sin() * r * radius;

        let picked_color =
            color_at(lightness, *c_value, *hue_value).unwrap_or(Color32::TRANSPARENT);

        ui.painter().circle(
            Pos2::new(x, y),
            rect.width() / 14.0,
            picked_color,
            egui::Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        );
    }

    response
}

#[derive(Debug, Clone)]
enum OKLabPlotType {
    StableLightness,
    StableChroma,
    StableHue,
}

fn color_picker_oklch_2d(ui: &mut Ui, lch: &mut Oklch, alpha: &mut f32) {
    let current_color_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);

    // let Oklch { l, c, h } = *lch;

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
    ui.horizontal(|ui| {
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableHue), "Lightness/Chroma")
            .clicked()
        {
            tab = OKLabPlotType::StableHue;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableChroma), "Hue/Lightness")
            .clicked()
        {
            tab = OKLabPlotType::StableChroma;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableLightness), "Hue/Chroma")
            .clicked()
        {
            tab = OKLabPlotType::StableLightness;
        }
    });

    ui.data_mut(|d| {
        d.insert_temp(ui.id(), tab.clone());
    });

    match tab {
        OKLabPlotType::StableHue => {
            color_slider_2d_grid_oklch(
                ui,
                &mut state.l,
                &mut state.c,
                state.h * std::f32::consts::TAU,
                |l, c, h| {
                    try_oklch_to_color32(Oklch {
                        l,
                        c: c * 0.325, // match your chroma scaling
                        h,
                    })
                },
            );
        }
        OKLabPlotType::StableChroma => {
            color_slider_2d_grid_oklch_radial(
                ui,
                &mut state.l,
                &mut state.h,
                state.c,
                |l, c, h| {
                    try_oklch_to_color32(Oklch {
                        l,
                        c: c * 0.325,
                        h: h * std::f32::consts::TAU,
                    })
                },
            );
        }
        OKLabPlotType::StableLightness => {
            color_slider_2d_grid_oklch_radial_2(
                ui,
                &mut state.c,
                &mut state.h,
                state.l,
                |l, c, h| {
                    try_oklch_to_color32(Oklch {
                        l: l,
                        c: c * 0.325,
                        h: h * std::f32::consts::TAU,
                    })
                },
            );
        }
    }

    ui.horizontal(|ui| {
        show_srgba_unmultiplied(ui, oklch_to_color32(*lch).to_array(), current_color_size)
            .on_hover_text("Selected color");
        // if ui
        //     .button("📋")
        //     .on_hover_text("Click to copy color values")
        //     .clicked()
        // {
        //     let srgb: Srgb =
        //         linear_srgb_to_srgb(oklab_to_linear_srgb(oklch_to_oklab(state.previous_color)));
        //     ui.copy_text(format!("{srgb.r}, {srgb.g}, {srgb.b}"));
        // };
    });

    // --- Lightness slider (0..1) ---
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.l,
            lch.l,
            |l_mut| try_oklch_to_color32(Oklch { l: l_mut, c, h }),
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
            |c_mut| {
                try_oklch_to_color32(Oklch {
                    l,
                    c: c_mut * 0.325,
                    h,
                })
            },
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
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.h,
            lch.h / std::f32::consts::TAU,
            |h_mut| {
                try_oklch_to_color32(Oklch {
                    l: l,
                    c: c,
                    h: h_mut * std::f32::consts::TAU,
                })
            },
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

    // let edited = drag_value(ui, "H ", &mut state.h).changed();

    lch.l = state.l;
    lch.c = state.c * 0.325;
    lch.h = state.h * std::f32::consts::TAU;

    ui.data_mut(|d| {
        d.insert_temp(state_id, state);
    });

    // TODO: Add alpha slider?
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
    // --- Convert incoming Color32 → OKLCh ---
    let srgb = Srgb {
        r: color.r() as f32 / 255.0,
        g: color.g() as f32 / 255.0,
        b: color.b() as f32 / 255.0,
    };

    let linear = srgb_to_linear_srgb(srgb);
    let lab = linear_srgb_to_oklab(linear);
    let mut new_lch = oklab_to_oklch(lab);

    let mut new_alpha = color.a() as f32 / 255.0;

    color_picker_oklch_2d(ui, &mut new_lch, &mut new_alpha);

    // --- Convert back OKLCh → Color32 ---
    let lab = oklch_to_oklab(new_lch);
    let linear = oklab_to_linear_srgb(lab);
    let srgb = linear_srgb_to_srgb(linear);

    let prev_color = color.clone();
    *color = Color32::from_rgba_unmultiplied(
        (srgb.r * 255.0).round() as u8,
        (srgb.g * 255.0).round() as u8,
        (srgb.b * 255.0).round() as u8,
        (new_alpha * 255.0).round() as u8,
    );
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
