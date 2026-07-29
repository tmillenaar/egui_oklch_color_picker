//! # Embedded picker example
//!
//! This example demonstrates the simplest way to embed an OKLCH picker
//! directly into an `egui` interface.
//!
//! The important pieces are:
//!
//! - storing an [`Oklch`] in your application state,
//! - calling [`color_picker_oklch_2d`],
//! - providing a stable `ui_identifier`,
//! - choosing an appropriate widget width.
//!
//! ```text
//! App State
//!     │
//!     ▼
//! Oklch
//!     │
//!     ▼
//! color_picker_oklch_2d(...)
//! ```
//!
//! See `PickerDemo::ui()` below for the complete implementation.

use eframe::egui;
use serde::{Deserialize, Serialize};

use egui_oklch_color_picker::{color_picker_oklch_2d, oklch::Oklch};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
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
    pub fn ui(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {

        let picker_width = ((content_rect.width() - 60.0) / 2.0).clamp(180.0, 320.0);

        const HEADER_HEIGHT: f32 = 48.0;
        const PREVIEW_HEIGHT: f32 = 240.0;

        let header_rect = egui::Rect::from_min_max(
            content_rect.left_top(),
            egui::pos2(content_rect.right(), content_rect.top() + HEADER_HEIGHT),
        );

        let preview_rect = egui::Rect::from_min_max(
            egui::pos2(content_rect.left(), header_rect.bottom()),
            egui::pos2(content_rect.right(), header_rect.bottom() + PREVIEW_HEIGHT),
        );

        let pickers_rect = egui::Rect::from_min_max(
            egui::pos2(content_rect.left(), preview_rect.bottom()),
            content_rect.right_bottom(),
        );

        let left_rect = egui::Rect::from_min_max(
            pickers_rect.left_top(),
            egui::pos2(
                pickers_rect.center().x - 10.0,
                pickers_rect.bottom(),
            ),
        );

        let right_rect = egui::Rect::from_min_max(
            egui::pos2(
                pickers_rect.center().x + 10.0,
                pickers_rect.top(),
            ),
            pickers_rect.right_bottom(),
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(header_rect), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new(
                        "Permanent panels on the sides control the colors in the center",
                    )
                    .size(20.0)
                    .italics(),
                );
            });
        });

        ui.scope_builder(egui::UiBuilder::new().max_rect(preview_rect), |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            painter.rect_filled(rect, 0.0, ui.visuals().panel_fill);

            let center = rect.center();

            let radius = (rect.height() * 0.30).clamp(40.0, 90.0);
            let offset = radius * 0.46;

            painter.circle_filled(
                center + egui::vec2(-offset, 0.0),
                radius,
                self.left,
            );

            painter.circle_filled(
                center + egui::vec2(offset, 0.0),
                radius,
                self.right,
            );
        });

        ui.scope_builder(egui::UiBuilder::new().max_rect(preview_rect), |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            painter.rect_filled(rect, 0.0, ui.visuals().panel_fill);

            let center = rect.center();

            let radius = (rect.height() * 0.30).clamp(40.0, 90.0);
            let offset = radius * 0.46;

            painter.circle_filled(
                center + egui::vec2(-offset, 0.0),
                radius,
                self.left,
            );

            painter.circle_filled(
                center + egui::vec2(offset, 0.0),
                radius,
                self.right,
            );
        });

        // Left picker
        ui.scope_builder(egui::UiBuilder::new().max_rect(left_rect), |ui| {
            ui.heading("Opaque OKLCH");

            color_picker_oklch_2d(
                ui,
                &mut self.left,
                picker_width,
                false,
                false,
                "left_picker",
            );
        });

        // Right picker
        ui.scope_builder(egui::UiBuilder::new().max_rect(right_rect), |ui| {
            ui.heading("OKLCH with alpha");

            color_picker_oklch_2d(
                ui,
                &mut self.right,
                picker_width,
                true,
                false,
                "right_picker",
            );
        });

        // // Center preview
        // ui.scope_builder(egui::UiBuilder::new().max_rect(center_rect), |ui| {
        //     let rect = ui.max_rect();
        //     let painter = ui.painter();

        //     painter.rect_filled(rect, 0.0, ui.visuals().panel_fill);

        //     let center = rect.center();

        //     let size = rect.size().min_elem();

        //     let radius = (size * 0.28).clamp(40.0, 120.0);
        //     let offset = radius * 0.46;

        //     painter.circle_filled(center + egui::vec2(-offset, 0.0), radius, self.left);

        //     painter.circle_filled(center + egui::vec2(offset, 0.0), radius, self.right);
        // });
    }
}
