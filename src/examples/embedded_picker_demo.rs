use eframe::egui;
use egui_oklab::{color_picker_oklch_2d, oklch::Oklch};

pub struct EmbeddedDemo {
    left: Oklch,
    right: Oklch,
}

impl Default for EmbeddedDemo {
    fn default() -> Self {
        Self {
            left: egui::Color32::from_rgb(70, 150, 255).into(),
            right: egui::Color32::from_rgba_unmultiplied(255, 80, 80, 160).into(),
        }
    }
}

impl EmbeddedDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let picker_width = (ui.available_width() * 0.28).clamp(180.0, 380.0);
        let panel_width = picker_width + 80.0;
        const HEADER_HEIGHT: f32 = 48.0;

        let rect = ui.available_rect_before_wrap();

        //
        // Header
        //
        let header_rect = egui::Rect::from_min_max(
            rect.left_top(),
            egui::pos2(rect.right(), rect.top() + HEADER_HEIGHT),
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(header_rect), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new(
                        "Permanent panels on the sides control the colors in the center.",
                    )
                    .size(20.0)
                    .italics(),
                );
            });
        });

        //
        // Remaining content
        //
        let content = egui::Rect::from_min_max(
            egui::pos2(rect.left(), header_rect.bottom()),
            rect.right_bottom(),
        );

        let left_rect = egui::Rect::from_min_max(
            content.left_top(),
            egui::pos2(content.left() + panel_width, content.bottom()),
        );

        let right_rect = egui::Rect::from_min_max(
            egui::pos2(content.right() - panel_width, content.top()),
            content.right_bottom(),
        );

        let center_rect = egui::Rect::from_min_max(
            egui::pos2(left_rect.right(), content.top()),
            egui::pos2(right_rect.left(), content.bottom()),
        );

        //
        // Left picker
        //
        ui.scope_builder(egui::UiBuilder::new().max_rect(left_rect), |ui| {
            ui.heading("Opaque OKLCH");

            color_picker_oklch_2d(ui, &mut self.left, picker_width, false, false);
        });

        //
        // Right picker
        //
        ui.scope_builder(egui::UiBuilder::new().max_rect(right_rect), |ui| {
            ui.heading("OKLCH with alpha");

            color_picker_oklch_2d(ui, &mut self.right, picker_width, true, false);
        });

        //
        // Center preview
        //
        ui.scope_builder(egui::UiBuilder::new().max_rect(center_rect), |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            painter.rect_filled(rect, 0.0, ui.visuals().panel_fill);

            let center = rect.center();

            let size = rect.size().min_elem();

            let radius = (size * 0.28).clamp(40.0, 120.0);
            let offset = radius * 0.46;

            painter.circle_filled(center + egui::vec2(-offset, 0.0), radius, self.left);

            painter.circle_filled(center + egui::vec2(offset, 0.0), radius, self.right);
        });
    }
}
