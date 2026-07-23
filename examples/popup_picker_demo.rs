use eframe::egui;
use egui_oklab::{
    color_edit_button_oklch, color_edit_button_oklcha, color_popup_oklch_advanced,
    color_popup_oklcha_advanced, oklch::Oklch,
};

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

fn editable_rect(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    color: &mut Oklch,
    picker_width: f32,
    id_source: impl std::hash::Hash,
    with_alpha: bool,
) {
    let corner_rounding = 24;

    let response = ui.interact(rect, ui.make_persistent_id(id_source), egui::Sense::click());
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);

        ui.painter().rect_stroke(
            rect.expand(3.0),
            corner_rounding,
            egui::Stroke::new(3.0, ui.visuals().selection.stroke.color),
            egui::StrokeKind::Outside,
        );
    }

    ui.painter().rect_filled(rect, corner_rounding, *color);

    if with_alpha {
        color_popup_oklcha_advanced(
            &response,
            response.clicked(),
            response.clicked(),
            color,
            picker_width,
        );
    } else {
        color_popup_oklch_advanced(
            &response,
            response.clicked(),
            response.clicked(),
            color,
            picker_width,
        );
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        const PICKER_WIDTH: f32 = 300.0;
        const MARGIN: f32 = 10.0;

        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let center = rect.center();
            let bg_size = egui::vec2(420.0, 420.0);
            let fg_size = egui::vec2(180.0, 180.0);
            let offset = 70.0;

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

                    let response = ui.add_sized([100.0, 20.0], egui::Button::new(label));

                    if response.clicked() {
                        self.dark_mode = !self.dark_mode;
                    }

                    response.on_hover_text("Switch between light and dark mode");
                });

            // Instruction description
            egui::Area::new("instructions".into())
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Click any colored shape to open its color picker.")
                            .size(24.0)
                            .italics(),
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
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "OKLCHA Color Picker Demo",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
