use eframe::egui;
use egui_oklab::{Alpha, color_edit_button_oklcha};

use std::time::Instant;

struct MyApp {
    color: egui::Color32,
    color_reference: egui::Color32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            color: egui::Color32::from_rgb(128, 128, 255),
            color_reference: egui::Color32::from_rgb(128, 128, 255),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame_start = Instant::now();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui Color Picker");

            ui.label("Pick a color:");

            let widget_width = 350.;
            color_edit_button_oklcha(ui, &mut self.color, widget_width);

            let mut hsva = self.color_reference.into();
            ui.color_edit_button_hsva(&mut hsva);
            self.color_reference = hsva.into();

            ui.separator();

            ui.label("Preview:");

            let (rect, _) = ui.allocate_exact_size(egui::vec2(200.0, 100.0), egui::Sense::hover());

            ui.painter().rect_filled(rect, 5.0, self.color);
        });
        let frame_time = frame_start.elapsed().as_secs_f64();
        let fps = 1.0 / frame_time;
        println!("FPS: {:.1}", fps);
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
