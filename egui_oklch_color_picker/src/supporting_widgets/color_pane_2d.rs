use egui::*;

use crate::color_projection::*;
use crate::coordinate_transforms::*;
use crate::picker_2d::*;
use crate::supporting_widgets::markers::*;

pub fn color_slider_2d<T, P>(
    ui: &mut Ui,
    coord: &mut T::Coord,
    projection: P,
    ui_identifiable_label: impl egui::AsIdSalt,
) -> (Response, bool)
where
    T: CoordinateTransform + Send + Sync + 'static,
    T::Coord: Clone + Copy + PartialEq,
    P: ColorProjection<T::Coord> + Clone + PartialEq + Send + Sync + 'static,
{
    let mut color_changed = false;

    let desired_size = Vec2::splat(ui.spacing().slider_width + 64.);

    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    let picker_id = ui.id().with(ui_identifiable_label);

    let mut picker = ui
        .data(|d| d.get_temp::<ColorPicker2d<T, P>>(picker_id))
        .unwrap_or_else(|| ColorPicker2d::new(ui, rect.size(), projection.clone()));

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
        let oklch = picker.projection.color_at(*coord);
        color_is_valid = oklch.is_valid_srgb();

        // Draw marker
        let (indicator_color, marker_angle) = if color_is_valid {
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
        if color_is_valid {
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

    // FIXME: On mutlitouch devices, we need to delay the selection of the color to
    //        prevent the user from accidentally selecting colors when panning/zooming
    let is_multitouch = response.ctx.input(|i| i.multi_touch().is_some());
    if !is_multitouch
        && response
            .ctx
            .input(|i| i.pointer.button_down(PointerButton::Primary))
    {
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
