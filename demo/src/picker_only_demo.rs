use eframe::egui;
use egui_oklab::{color_picker_oklch_2d, oklch::Oklch};

pub struct PickerDemo {
    color: Oklch,
}

impl Default for PickerDemo {
    fn default() -> Self {
        Self {
            color: egui::Color32::from_rgb(46, 220, 234).into(),
        }
    }
}

impl PickerDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let picker_width = (ui.available_width() * 0.28).clamp(180.0, 380.0);
        let panel_width = picker_width + 80.0;
        let centered_rect =
            egui::Rect::from_center_size(content_rect.center(), egui::vec2(panel_width, 800.));

        ui.scope_builder(egui::UiBuilder::new().max_rect(centered_rect), |ui| {
            color_picker_oklch_2d(ui, &mut self.color, picker_width, false, false);
        });
    }
}
