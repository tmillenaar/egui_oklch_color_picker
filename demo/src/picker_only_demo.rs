//! # Embedded color picker example
//!
//! This example demonstrates the simplest way to embed the
//! [`egui_oklch_color_picker::color_picker_oklch_2d`] widget directly into an `egui`
//! interface.
//!
//! The application stores a single [`egui_oklch_color_picker::oklch::Oklch`] color in its
//! state and passes it to the picker each frame. As the user interacts with
//! the widget, the selected color is updated in place.
//!
//! The important parts of this example are:
//!
//! - Storing the selected color as an [`Oklch`] value.
//! - Calling [`color_picker_oklch_2d`] from your UI code.
//! - Choosing a suitable width for the picker based on the available space.
//! - Providing a stable identifier so the picker can preserve its interaction
//!   state between frames.
//!
//! This approach is ideal for settings panels, inspectors, asset editors, or
//! any application where the color picker is intended to remain visible rather
//! than appearing in a popup.
//!
//! See [`PickerDemo::ui`] for the complete implementation.

use eframe::egui;
use serde::{Deserialize, Serialize};

use egui_oklch_color_picker::{color_picker_oklch_2d, oklch::Oklch};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
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
            color_picker_oklch_2d(
                ui,
                &mut self.color,
                picker_width,
                false,
                false,
                "the_lonely_picker",
            );
        });
    }
}
