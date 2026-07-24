use eframe::egui;
use egui_oklab::{color_edit_button_oklch, color_edit_button_oklcha, oklch::Oklch};

pub struct ButtonDemo {
    top_left: Oklch,
    top_right: Oklch,
    bottom_left: Oklch,
    bottom_right: Oklch,
    background: Oklch,
}

impl Default for ButtonDemo {
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

impl ButtonDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        const PICKER_WIDTH: f32 = 380.0;
        const MARGIN: f32 = 10.0;

        let painter = ui.painter();

        let center = content_rect.center();

        let radius = 110.0;
        let offset = 70.0;

        painter.rect_filled(content_rect, 0.0, ui.visuals().panel_fill);

        painter.circle_filled(center, radius * 2.5, self.background);

        painter.circle_filled(center + egui::vec2(-offset, -offset), radius, self.top_left);

        painter.circle_filled(center + egui::vec2(offset, -offset), radius, self.top_right);

        painter.circle_filled(
            center + egui::vec2(-offset, offset),
            radius,
            self.bottom_left,
        );

        painter.circle_filled(
            center + egui::vec2(offset, offset),
            radius,
            self.bottom_right,
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
            ui.put(
                egui::Rect::from_center_size(
                    content_rect.center_top() + egui::vec2(0.0, 30.0),
                    egui::vec2(500.0, 30.0),
                ),
                egui::Label::new(
                    egui::RichText::new("Buttons around the edge open color pickers")
                        .size(24.0)
                        .italics(),
                ),
            );
        });

        egui::Area::new("top_left".into())
            .fixed_pos(content_rect.left_top() + egui::vec2(MARGIN, MARGIN))
            .show(ui.ctx(), |ui| {
                color_edit_button_oklcha(ui, &mut self.top_left, PICKER_WIDTH);
            });

        egui::Area::new("top_right".into())
            .fixed_pos(content_rect.right_top() + egui::vec2(-MARGIN, MARGIN))
            .show(ui.ctx(), |ui| {
                color_edit_button_oklcha(ui, &mut self.top_right, PICKER_WIDTH);
            });

        egui::Area::new("background_picker".into())
            .fixed_pos(content_rect.right_center() + egui::vec2(-MARGIN, 0.0))
            .show(ui.ctx(), |ui| {
                color_edit_button_oklch(ui, &mut self.background, PICKER_WIDTH)
                    .on_hover_text("Background color");
            });

        egui::Area::new("bottom_left".into())
            .fixed_pos(content_rect.left_bottom() + egui::vec2(MARGIN, -MARGIN * 3.))
            .show(ui.ctx(), |ui| {
                color_edit_button_oklcha(ui, &mut self.bottom_left, PICKER_WIDTH);
            });

        egui::Area::new("bottom_right".into())
            .fixed_pos(content_rect.right_bottom() + egui::vec2(-MARGIN, -MARGIN * 3.))
            .show(ui.ctx(), |ui| {
                color_edit_button_oklcha(ui, &mut self.bottom_right, PICKER_WIDTH);
            });
    }
}
