use eframe::egui;
use egui_oklab::{Alpha, color_edit_button_oklcha};

struct MyApp {
    color: egui::Color32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            color: egui::Color32::from_rgb(128, 128, 255),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui Color Picker");

            ui.label("Pick a color:");

            color_edit_button_oklcha(ui, &mut self.color);

            ui.separator();

            ui.label("Preview:");

            let (rect, _) = ui.allocate_exact_size(egui::vec2(200.0, 100.0), egui::Sense::hover());

            ui.painter().rect_filled(rect, 5.0, self.color);
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Color Picker Example",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
