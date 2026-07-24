use eframe::egui;
use egui_oklab::{color_popup_oklch_advanced, color_popup_oklcha_advanced, oklch::Oklch};

pub struct PopupDemo {
    top_left: Oklch,
    top_right: Oklch,
    bottom_left: Oklch,
    bottom_right: Oklch,
    background: Oklch,
}

impl Default for PopupDemo {
    fn default() -> Self {
        Self {
            top_left: egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128).into(),
            top_right: egui::Color32::from_rgba_unmultiplied(0, 255, 0, 128).into(),
            bottom_left: egui::Color32::from_rgba_unmultiplied(0, 128, 255, 128).into(),
            bottom_right: egui::Color32::from_rgba_unmultiplied(255, 255, 0, 128).into(),
            background: egui::Color32::from_rgb(128, 128, 128).into(),
        }
    }
}

fn editable_rect(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    color: &mut Oklch,
    picker_width: f32,
    id_source: impl std::hash::Hash,
    with_alpha: bool,
) {
    const CORNER_ROUNDING: u8 = 24;

    let response = ui.interact(rect, ui.make_persistent_id(id_source), egui::Sense::click());

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);

        ui.painter().rect_stroke(
            rect.expand(3.0),
            CORNER_ROUNDING,
            egui::Stroke::new(3.0, ui.visuals().selection.stroke.color),
            egui::StrokeKind::Outside,
        );
    }

    ui.painter().rect_filled(rect, CORNER_ROUNDING, *color);

    if with_alpha {
        color_popup_oklcha_advanced(
            &response,
            response.clicked(),
            response.clicked(),
            color,
            picker_width,
        );
    } else {
        color_popup_oklch_advanced(
            &response,
            response.clicked(),
            response.clicked(),
            color,
            picker_width,
        );
    }
}

impl PopupDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        const PICKER_WIDTH: f32 = 380.0;

        let rect = ui.available_rect_before_wrap();
        let center = rect.center();

        let bg_size = egui::vec2(420.0, 420.0);
        let fg_size = egui::vec2(180.0, 180.0);
        let offset = 70.0;

        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.put(
                egui::Rect::from_center_size(
                    rect.center_top() + egui::vec2(0.0, 30.0),
                    egui::vec2(500.0, 30.0),
                ),
                egui::Label::new(
                    egui::RichText::new("Click a colored shape to open its color picker")
                        .size(24.0)
                        .italics(),
                ),
            );
        });

        editable_rect(
            ui,
            egui::Rect::from_center_size(center, bg_size),
            &mut self.background,
            PICKER_WIDTH,
            "background_rect",
            false,
        );

        editable_rect(
            ui,
            egui::Rect::from_center_size(center + egui::vec2(-offset, -offset), fg_size),
            &mut self.top_left,
            PICKER_WIDTH,
            "top_left_rect",
            true,
        );

        editable_rect(
            ui,
            egui::Rect::from_center_size(center + egui::vec2(offset, -offset), fg_size),
            &mut self.top_right,
            PICKER_WIDTH,
            "top_right_rect",
            true,
        );

        editable_rect(
            ui,
            egui::Rect::from_center_size(center + egui::vec2(-offset, offset), fg_size),
            &mut self.bottom_left,
            PICKER_WIDTH,
            "bottom_left_rect",
            true,
        );

        editable_rect(
            ui,
            egui::Rect::from_center_size(center + egui::vec2(offset, offset), fg_size),
            &mut self.bottom_right,
            PICKER_WIDTH,
            "bottom_right_rect",
            true,
        );
    }
}
