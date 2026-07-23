use eframe::egui;

mod button_picker_demo;
mod embedded_picker_demo;
mod popup_picker_demo;

enum Demo {
    Embedded(embedded_picker_demo::EmbeddedDemo),
    Popup(popup_picker_demo::PopupDemo),
    Button(button_picker_demo::ButtonDemo),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DemoKind {
    Embedded,
    Popup,
    Button,
}

pub struct GalleryApp {
    current: DemoKind,
    demo: Demo,
    dark_mode: bool,
}

impl Default for GalleryApp {
    fn default() -> Self {
        Self {
            current: DemoKind::Embedded,
            demo: Demo::Embedded(embedded_picker_demo::EmbeddedDemo::default()),
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

        self.demo = match demo {
            DemoKind::Embedded => Demo::Embedded(embedded_picker_demo::EmbeddedDemo::default()),
            DemoKind::Popup => Demo::Popup(popup_picker_demo::PopupDemo::default()),
            DemoKind::Button => Demo::Button(button_picker_demo::ButtonDemo::default()),
        };
    }
}

impl eframe::App for GalleryApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("egui-oklab");

                ui.separator();

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

                ui.separator();

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
                    ui.separator();

                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match &mut self.demo {
            Demo::Embedded(app) => app.ui(ui),
            Demo::Popup(app) => app.ui(ui),
            Demo::Button(app) => app.ui(ui),
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "egui-oklab Demo Gallery",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(GalleryApp::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

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
