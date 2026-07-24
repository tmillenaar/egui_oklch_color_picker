use crate::egui::*;

pub fn credit_footer(ui: &mut Ui) {
    ui.with_layout(
        crate::egui::Layout::bottom_up(crate::egui::Align::Center),
        |ui| {
            ui.horizontal(|ui| {
                // Egui + eframe credits
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("UI powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
            // Krita color picker credits
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Picker design inspired by ");
                ui.hyperlink_to(
                    "A Better Way To Color Pick (video)",
                    "https://www.youtube.com/watch?v=fSnltHozDcU",
                );
                ui.label(" and ");
                ui.hyperlink_to(
                    "oklab_colour_picker_krita (github)",
                    "https://github.com/gibapiu/oklab_colour_picker_krita",
                );
                ui.label(".");
            });
            // OKLAB & OKLCH credits
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Learn more about the OKLCH color space at ");
                ui.hyperlink_to("oklch.fyi", "https://oklch.fyi/");
                ui.label(".");
            });
            crate::egui::warn_if_debug_build(ui);
        },
    );
}
