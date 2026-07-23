use egui::*;

use crate::supporting_widgets::background_checkers::*;

/// Show a color with background checkers to demonstrate transparency (if any).
/// Based on https://github.com/emilk/egui/pull/8208/changes
pub fn show_srgba_unmultiplied(
    ui: &mut Ui,
    srgba: [u8; 4],
    desired_size: Vec2,
    nr_checkers_y: u32,
) -> Response {
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
    if ui.is_rect_visible(rect) {
        show_srgba_unmultiplied_at(ui.painter(), srgba, rect, nr_checkers_y);
    }
    response
}

/// Show a color with background checkers to demonstrate transparency (if any).
/// Based on https://github.com/emilk/egui/pull/8208/changes
pub fn show_srgba_unmultiplied_at(
    painter: &Painter,
    [r, g, b, a]: [u8; 4],
    rect: Rect,
    nr_checkers_y: u32,
) {
    if a == 255 {
        painter.rect_filled(rect, 0.0, Color32::from_rgb(r, g, b));
    } else {
        background_checkers(painter, rect, nr_checkers_y);
        let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
        let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
        painter.rect_filled(left, 0.0, Color32::from_rgba_unmultiplied(r, g, b, a));
        painter.rect_filled(right, 0.0, Color32::from_rgb(r, g, b));
    }
}
