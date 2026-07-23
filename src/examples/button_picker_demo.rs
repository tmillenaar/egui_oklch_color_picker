use eframe::egui;
use egui_oklab::{color_edit_button_oklch, color_edit_button_oklcha, oklch::Oklch};

struct MyApp {
    top_left: Oklch,
    top_right: Oklch,
    bottom_left: Oklch,
    bottom_right: Oklch,
    background: Oklch,
    dark_mode: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            top_left: egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128).into(),
            top_right: egui::Color32::from_rgba_unmultiplied(0, 255, 0, 128).into(),
            bottom_left: egui::Color32::from_rgba_unmultiplied(0, 128, 255, 128).into(),
            bottom_right: egui::Color32::from_rgba_unmultiplied(255, 255, 0, 128).into(),
            background: egui::Color32::from_rgb(128, 128, 128).into(),
            dark_mode: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        const PICKER_WIDTH: f32 = 300.0;
        const MARGIN: f32 = 10.0;

        // Set theme
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        // Draw the center visualization first.
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            let center = rect.center();

            let radius = 110.0;
            let offset = 70.0;

            painter.rect_filled(rect, 0.0, ui.visuals().panel_fill);

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
        });

        // Quit button
        egui::Area::new("quit_button".into())
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 20.0))
            .show(ctx, |ui| {
                if ui
                    .add_sized([34.0, 20.0], egui::Button::new("Quit"))
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

        // Theme toggle
        egui::Area::new("theme_toggle".into())
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 50.0))
            .show(ctx, |ui| {
                let label = if self.dark_mode {
                    "🌙 Dark Theme"
                } else {
                    "☀ Light Theme"
                };

                let button_response = ui.add_sized([100.0, 20.0], egui::Button::new(label));
                if button_response.clicked() {
                    self.dark_mode = !self.dark_mode;
                }
                button_response.on_hover_text("Switch between light and dark mode");
            });

        // Instruction description
        egui::Area::new("instructions".into())
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Buttons around the edge to open the color pickers")
                        .size(24.0)
                        .italics(),
                );
            });

        // Top-left picker
        egui::Area::new("top_left".into())
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(MARGIN, MARGIN))
            .show(ctx, |ui| {
                color_edit_button_oklcha(ui, &mut self.top_left, PICKER_WIDTH);
            });

        // Top-right picker
        egui::Area::new("top_right".into())
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-MARGIN, MARGIN))
            .show(ctx, |ui| {
                color_edit_button_oklcha(ui, &mut self.top_right, PICKER_WIDTH);
            });

        // Background picker
        egui::Area::new("background_picker".into())
            .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-MARGIN, MARGIN))
            .show(ctx, |ui| {
                color_edit_button_oklch(ui, &mut self.background, PICKER_WIDTH)
                    .on_hover_text("Background color");
            });

        // Bottom-left picker
        egui::Area::new("bottom_left".into())
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(MARGIN, -MARGIN))
            .show(ctx, |ui| {
                color_edit_button_oklcha(ui, &mut self.bottom_left, PICKER_WIDTH);
            });

        // Bottom-right picker
        egui::Area::new("bottom_right".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-MARGIN, -MARGIN))
            .show(ctx, |ui| {
                color_edit_button_oklcha(ui, &mut self.bottom_right, PICKER_WIDTH);
            });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "OKLCHA Color Picker Demo",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas",
                eframe::WebOptions::default(),
                Box::new(|_| Ok(Box::new(MyApp::default()))),
            )
            .await
            .unwrap();
    });
}
