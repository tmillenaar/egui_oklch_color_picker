use egui::*;

use crate::oklch::Oklch;
use crate::supporting_widgets::background_checkers::*;

fn contrast_color(color: impl Into<Oklch>) -> Color32 {
    if color.into().l < 0.45 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

/// Show a color with background checkers to demonstrate transparency (if any).
/// Based on https://github.com/emilk/egui/pull/8208/changes
pub fn show_srgba_unmultiplied(
    ui: &mut Ui,
    srgba: [u8; 4],
    desired_size: Vec2,
    nr_checkers_y: u32,
    overlay_text: Option<impl ToString>,
) -> Response {
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover() | Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);

        show_srgba_unmultiplied_at(ui.painter(), srgba, rect.shrink(1.0), nr_checkers_y);

        let painter = ui.painter();
        painter.rect_stroke(
            rect,
            visuals.corner_radius.at_most(2),
            Stroke::new(1.5, visuals.bg_fill),
            StrokeKind::Inside,
        );

        if let Some(text) = overlay_text {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::TextStyle::Button.resolve(&painter.ctx().style()),
                contrast_color(Color32::from_rgb(srgba[0], srgba[1], srgba[2])),
            );
        }
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
