use eframe::egui;
use egui_oklab::{color_picker_oklch_2d, oklch::Oklch};

struct MyApp {
    left: Oklch,
    right: Oklch,
    dark_mode: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            left: egui::Color32::from_rgb(70, 150, 255).into(),
            right: egui::Color32::from_rgba_unmultiplied(255, 80, 80, 160).into(),
            dark_mode: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        const PICKER_WIDTH: f32 = 370.0;
        const MARGIN: f32 = 10.0;

        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
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
                    egui::RichText::new(
                        "Permanent panels on the sides control the colors in the center.",
                    )
                    .size(20.0)
                    .italics(),
                );
            });

        // Center visualization
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            painter.rect_filled(rect, 0.0, ui.visuals().panel_fill);

            let center = rect.center();
            let radius = 120.0;

            painter.circle_filled(center + egui::vec2(-55.0, 0.0), radius, self.left);

            painter.circle_filled(center + egui::vec2(55.0, 0.0), radius, self.right);
        });

        // Left picker (OKLCH)
        egui::SidePanel::left("left_picker")
            .default_width(PICKER_WIDTH + 20.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Opaque OKLCH");

                color_picker_oklch_2d(
                    ui,
                    &mut self.left,
                    PICKER_WIDTH,
                    false, // no alpha slider
                    false, // don't reset state every frame
                );
            });

        // Right picker (OKLCHA)
        egui::SidePanel::right("right_picker")
            .default_width(PICKER_WIDTH + 20.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("OKLCH with alpha");

                color_picker_oklch_2d(
                    ui,
                    &mut self.right,
                    PICKER_WIDTH,
                    true, // alpha slider enabled
                    false,
                );
            });
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "OKLCH 2D Picker Demo",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(MyApp::default()))),
    )
}
