//! # Popup picker example
//!
//! This example demonstrates how to attach an OKLCH color picker popup to
//! arbitrary widgets using
//! [`egui_oklch_color_picker::color_popup_oklch_advanced`] and
//! [`egui_oklch_color_picker::color_popup_oklcha_advanced`].
//!
//! Unlike the convenience button widgets, the popup APIs can be attached to
//! any interactive element. In this example, clicking one of several colored
//! rectangles opens a color picker anchored to that rectangle.
//!
//! The important parts of this example are:
//!
//! - Creating an interactive region with [`egui::Ui::interact`].
//! - Passing the resulting [`egui::Response`] to one of the popup functions.
//! - Providing a stable, unique identifier for each popup instance.
//! - Using the alpha-enabled picker (`color_popup_oklcha_advanced`) for
//!   translucent colors while using the simpler OKLCH picker for opaque ones.
//!
//! This approach is useful when your application already has custom controls,
//! drawings, or other interactive elements and you want them to act as color
//! pickers without replacing them with standard buttons.
//!
//! See [`PopupDemo::ui`] for the complete implementation.

use eframe::egui;
use serde::{Deserialize, Serialize};

use egui_oklch_color_picker::{
    color_popup_oklch_advanced, color_popup_oklcha_advanced, oklch::Oklch,
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
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
    id_source_name: &str,
    with_alpha: bool,
) {
    const CORNER_ROUNDING: u8 = 24;

    let response = ui.interact(
        rect,
        ui.make_persistent_id(id_source_name.clone()),
        egui::Sense::click(),
    );

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
            id_source_name,
        );
    } else {
        color_popup_oklch_advanced(
            &response,
            response.clicked(),
            response.clicked(),
            color,
            picker_width,
            id_source_name,
        );
    }
}

impl PopupDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        const PICKER_WIDTH: f32 = 380.0;

        let center = content_rect.center();

        let bg_size = egui::vec2(420.0, 420.0);
        let fg_size = egui::vec2(180.0, 180.0);
        let offset = 70.0;

        ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
            ui.put(
                egui::Rect::from_center_size(
                    content_rect.center_top() + egui::vec2(0.0, 30.0),
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
