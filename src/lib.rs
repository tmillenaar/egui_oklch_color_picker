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

use egui::*;
use egui::Order::Background;
use egui::epaint::RectShape;
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
        }
    }

    ui.data_mut(|d| {
        d.insert_temp(picker_id, picker);
    });

    (response, viewport_contains_picked_color || color_is_valid)
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

#[derive(Clone, Default)]
struct NumericFieldState {
    text: String,
    select_all_next_frame: bool,
}

pub enum NumericEdit {
    None,
    Commit(String),
    Cancel,
}

pub fn numeric_field(
    ui: &mut Ui,
    id: Id,
    prefix: &str,
    value: String,
) -> (Response, NumericEdit) {
    let mut state = ui
        .data(|d| d.get_temp::<NumericFieldState>(id))
        .unwrap_or_else(|| NumericFieldState {
            text: value.clone(),
            select_all_next_frame: false,
        });

    // If we're not editing, keep the displayed text in sync.
    if !ui.memory(|m| m.has_focus(id)) {
        state.text = value.clone();
    }

    // 1. Was the flag already set from the previous frame?
    let select_all = state.select_all_next_frame;
    state.select_all_next_frame = false;

    // 2. Build the widget
    let mut output = ui
        .horizontal(|ui| {
            ui.label(prefix);

            egui::TextEdit::singleline(&mut state.text)
                .desired_width(25.0)
                .id(id)
                .show(ui)
        })
        .inner;

    let response = output.response;

    // 3. If requested, perform the selection now.
    if select_all {
        use egui::text::{CCursor, CCursorRange};

        output.state.cursor.set_char_range(Some(
            CCursorRange::two(
                CCursor::new(0),
                CCursor::new(state.text.chars().count()),
            ),
        ));

        output.state.store(ui.ctx(), response.id);
    }

    // 4. If we *just* gained focus, arm it for next frame.
    if response.gained_focus() {
        state.select_all_next_frame = true;
    }

    let pasted = response.has_focus()
        && ui.input(|i| {
            i.events.iter().any(|e| matches!(e, egui::Event::Paste(_)))
        });
    let enter = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    let escape = ui.input(|i| i.key_pressed(Key::Escape));

    let commit = response.lost_focus() || enter || pasted;

    let result = if escape {
        state.text = value;
        NumericEdit::Cancel
    } else if commit {
        NumericEdit::Commit(state.text.clone())
    } else {
        NumericEdit::None
    };

    // Drop focus on element after pasting
    if pasted {
        ui.memory_mut(|mem| mem.surrender_focus(id));
        return (response, NumericEdit::Commit(state.text.clone()));
    }

    ui.data_mut(|d| d.insert_temp(id, state));

    (response, result)
}

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
        let value = if normalized {
            value * 255.0
        } else {
            value
        };

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
    pub fn ui(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            changed |= self.edit_component(ui, "R", 0);
            changed |= self.edit_component(ui, "G", 1);
            changed |= self.edit_component(ui, "B", 2);
            if ui
                .button("📋")
                .on_hover_text("Click to copy RGB values")
                .clicked()
            {
                ui.ctx()
                    .copy_text(format!("{}, {}, {}", self.rgb[0], self.rgb[1], self.rgb[2]));
            }
        });

        changed
    }

    fn edit_component(
        &mut self,
        ui: &mut Ui,
        label: &str,
        index: usize,
    ) -> bool {
        let id = ui.id().with(label);

        let (_, edit) = numeric_field(
            ui,
            id,
            label,
            self.rgb[index].to_string(),
        );

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

#[derive(Debug, Clone)]
enum OKLabPlotType {
    StableLightness,
    StableChroma,
    StableHue,
}

fn color_picker_oklch_2d(ui: &mut Ui, lch: &mut Oklch, alpha: &mut f32) -> (bool, Color32) {
    let current_color_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);

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

    let mut color_sample = oklch_to_color32(lch.clone()); // technically the color from previous frame, but I want to display this above the 2d picker
    let mut update_color = false;

    let mut tab = ui.data(|d| {
        d.get_temp::<OKLabPlotType>(ui.id())
            .unwrap_or(OKLabPlotType::StableHue)
    });
    let mut tab_switched = false;
    ui.horizontal(|ui| {
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableHue), "Stable Hue")
            .clicked()
        {
            tab = OKLabPlotType::StableHue;
            tab_switched = true;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableChroma), "Stable Chroma")
            .clicked()
        {
            tab = OKLabPlotType::StableChroma;
            tab_switched = true;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableLightness), "Stable Lightness")
            .clicked()
        {
            tab = OKLabPlotType::StableLightness;
            tab_switched = true;
        }
    });

    ui.horizontal(|ui| {
        let color = color_sample.color;

        let mut r = color.r();
        let mut g = color.g();
        let mut b = color.b();
        let mut a = color.a();
        let mut rgb = [r, g, b];

        if RgbEdit::new(&mut rgb).ui(ui) {
            let new_color = Color32::from_rgba_unmultiplied(
                rgb[0],
                rgb[1],
                rgb[2],
                a
            );
            update_color = true;
            color_sample.color = new_color;
            *lch = color32_to_oklch(color_sample.color);
            state.l = lch.l;
            state.c = lch.c / 0.325;
            state.h = lch.h / std::f32::consts::TAU;
        }



        // ui.horizontal(|ui| {
        //     if ui
        //         .button("📋")
        //         .on_hover_text("Click to copy color values")
        //         .clicked()
        //     {
        //         ui.ctx()
        //             .copy_text(format!("{}, {}, {}", color.r(), color.g(), color.b()));
        //     }
        //     edit_color_field(ui, "R", &mut r, &mut rgba, 0, false);
        //     edit_color_field(ui, "G", &mut g, &mut rgba, 1, false);
        //     edit_color_field(ui, "B", &mut b, &mut rgba, 2, false);

        // });

        // let new_color = Color32::from_rgba_unmultiplied(r, g, b, a);
        // if new_color != color {
        //     update_color = true;
        //     color_sample.color = new_color;
        //     *lch = color32_to_oklch(color_sample.color);
        //     state.l = lch.l;
        //     state.c = lch.c / 0.325;
        //     state.h = lch.h / std::f32::consts::TAU;
        // }
    });

    ui.data_mut(|d| {
        d.insert_temp(ui.id(), tab.clone());
    });

    if tab_switched {
        state.l = lch.l;
        state.c = lch.c / 0.325;
        state.h = lch.h / std::f32::consts::TAU;
    }

    show_srgba_unmultiplied(ui, color_sample.color.to_array(), current_color_size)
        .on_hover_text("Selected color");

    let (new_color_, update_color_) = match tab {
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

    if update_color_ {
        update_color = true;
        color_sample.color = new_color_;
        *lch = color32_to_oklch(new_color_);
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

    (update_color, color_sample.color)
}

/// Shows 4 `DragValue` widgets to be used to edit the RGBA u8 values.
/// Alpha's `DragValue` is hidden when `Alpha::Opaque`.
///
/// Returns `true` on change.
fn srgba_edit_ui(ui: &mut Ui, [r, g, b, a]: &mut [u8; 4], alpha: Alpha) -> bool {
    let mut edited = false;

    ui.horizontal(|ui| {
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
