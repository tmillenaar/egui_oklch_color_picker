use egui::*;

pub fn copy_icon(ui: &mut Ui, size: Vec2) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        if response.hovered() {
            ui.painter()
                .rect_filled(rect, visuals.corner_radius, visuals.bg_fill);
        }

        let stroke = visuals.fg_stroke;
        let rounding = CornerRadius::same(2);

        // Square "pages"
        let side = size.x.min(size.y) * 0.50;
        let offset = side * 0.15;

        let center = rect.center();

        // Back page (top-right)
        let back = Rect::from_center_size(center + vec2(offset, -offset), vec2(side, side));

        // Front page (bottom-left)
        let front = Rect::from_center_size(center - vec2(offset, -offset), vec2(side, side));

        // Draw back page first.
        ui.painter()
            .rect_stroke(back, rounding, stroke, StrokeKind::Middle);

        // Cover the overlap so the front page appears on top.
        ui.painter()
            .rect_filled(front, rounding, ui.visuals().panel_fill);

        ui.painter()
            .rect_stroke(front, rounding, stroke, StrokeKind::Middle);
    }

    response
}
