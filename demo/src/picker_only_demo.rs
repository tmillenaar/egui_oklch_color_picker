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
        let picker_width = (ui.available_width() * 0.6).clamp(180.0, 540.0);
        let panel_width = picker_width + 80.0;
        let picker_height = picker_width*2.5;

        // Center rect if there is a lot of space on the screen, place at top if not.
        // We do this fallback because centering when there is little space causes the top of
        // the panel to be outside the top of the page.
        let top = if content_rect.height() > picker_height {
            content_rect.center().y - picker_height / 2.0
        } else {
            content_rect.top()
        };

        let rect = egui::Rect::from_min_size(
            egui::pos2(content_rect.center().x - panel_width / 2.0, content_rect.top()),
            egui::vec2(panel_width, picker_height),
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
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
