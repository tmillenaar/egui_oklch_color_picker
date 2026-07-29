//! # egui_oklch_color_picker Demo Gallery
//!
//! This crate contains complete example applications demonstrating the
//! widgets provided by `egui_oklch_color_picker`.
//!
//! ## Examples
//!
//! - [`picker_only_demo`] — Embed a standalone color picker into an existing UI.
//! - [`button_picker_demo`] — Use compact color buttons that open popup pickers.
//! - [`popup_picker_demo`] — Attach color pickers to arbitrary widgets.
//! - [`embedded_picker_demo`] — A larger application embedding the picker in a layout.
//!
//! Each module contains a complete, self-contained example that can be copied
//! into your own application.

use eframe::egui;
use serde::{Deserialize, Serialize};

mod credits;
use crate::credits::credit_footer;
use egui_oklch_color_picker_demo::{
    button_picker_demo, embedded_picker_demo, picker_only_demo, popup_picker_demo,
};

#[derive(serde::Serialize, serde::Deserialize)]
enum Demo {
    Embedded(embedded_picker_demo::EmbeddedDemo),
    Popup(popup_picker_demo::PopupDemo),
    Button(button_picker_demo::ButtonDemo),
    Picker(picker_only_demo::PickerDemo),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq)]
enum DemoKind {
    Embedded,
    Popup,
    Button,
    Picker,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct GalleryApp {
    current: DemoKind,

    embedded: embedded_picker_demo::EmbeddedDemo,
    popup: popup_picker_demo::PopupDemo,
    button: button_picker_demo::ButtonDemo,
    picker: picker_only_demo::PickerDemo,

    dark_mode: bool,
}

impl Default for GalleryApp {
    fn default() -> Self {
        Self {
            current: DemoKind::Picker,
            embedded: embedded_picker_demo::EmbeddedDemo::default(),
            popup: popup_picker_demo::PopupDemo::default(),
            button: button_picker_demo::ButtonDemo::default(),
            picker: picker_only_demo::PickerDemo::default(),
            dark_mode: true,
        }
    }
}

impl GalleryApp {
    fn switch_demo(&mut self, demo: DemoKind) {
        if demo == self.current {
            return;
        }
        self.current = demo;
    }
}

impl GalleryApp {
    fn demo_selector(&mut self, ui: &mut egui::Ui) {
        if ui
            .selectable_label(self.current == DemoKind::Picker, "Picker only")
            .clicked()
        {
            self.switch_demo(DemoKind::Picker);
        }

        if ui
            .selectable_label(self.current == DemoKind::Embedded, "Embedded Picker")
            .clicked()
        {
            self.switch_demo(DemoKind::Embedded);
        }

        if ui
            .selectable_label(self.current == DemoKind::Popup, "Popup Picker")
            .clicked()
        {
            self.switch_demo(DemoKind::Popup);
        }

        if ui
            .selectable_label(self.current == DemoKind::Button, "Button Picker")
            .clicked()
        {
            self.switch_demo(DemoKind::Button);
        }
    }
    fn bottom_buttons(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let theme = if self.dark_mode {
            "☀ Flash me baby!"
        } else {
            "🌙 Dark Theme"
        };

        if ui.button(theme).clicked() {
            self.dark_mode = !self.dark_mode;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if ui.button("Quit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

impl eframe::App for GalleryApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            let narrow = ui.available_width() < 750.0;

               if narrow {
                   ui.vertical(|ui| {
                       ui.heading("Oklch color picker");

                       ui.separator();

                       self.demo_selector(ui);

                       ui.separator();

                       self.bottom_buttons(ui, ctx);
                   });
               } else {
                   ui.horizontal(|ui| {
                       ui.heading("Oklch color picker");

                       ui.separator();

                       self.demo_selector(ui);

                       ui.separator();

                       self.bottom_buttons(ui, ctx);
                   });
               }
        });

        // Note: show bottom panel before central panel, otherwise the central panel does not know it's size.
        //       This is because we render the bottom panel from bottom to top.
        // Note: only show the credits panel on local build since the web version has the credits in the html
        #[cfg(not(target_arch = "wasm32"))]
        egui::TopBottomPanel::bottom("credits")
            .resizable(false)
            .show(ctx, |ui| {
                credit_footer(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let content_rect = ui.max_rect();

            match self.current {
                DemoKind::Embedded => self.embedded.ui(ui, content_rect),
                DemoKind::Popup => self.popup.ui(ui, content_rect),
                DemoKind::Button => self.button.ui(ui, content_rect),
                DemoKind::Picker => self.picker.ui(ui, content_rect),
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Oklch color picker Demo Gallery",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            Ok(Box::new(if let Some(storage) = cc.storage {
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
            } else {
                GalleryApp::default()
            }))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();

        let canvas = document
            .get_element_by_id("the_canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_| Ok(Box::new(GalleryApp::default()))),
            )
            .await
            .unwrap();
    });
}
