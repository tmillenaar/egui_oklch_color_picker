use std::f32::consts::TAU;

use egui::*;

use crate::oklch::*;
use crate::oklch_to_color32;
use crate::oklch_ui_state::*;
use crate::supporting_widgets::color_slider_1d_grid::color_slider_1d_grid;

pub fn oklch_sliders(
    ui: &mut Ui,
    selected_color: Oklch,
    with_alpha: bool,
    ui_identifier: *const Oklch, // use pointer to color in RAM as identifier to allow for multiple color panels. They may share state if they refer to the same color
) -> (bool, OklchUiState) {
    // Recover state from cache
    let state_id = ui.id().with(("oklch_ui_state", ui_identifier));
    let mut state: OklchUiState = ui.data(|d| d.get_temp::<OklchUiState>(state_id)).unwrap();
    let original_state = state.clone();
    let l = state.l;
    let c = state.c * 0.325;
    let h = state.h * TAU;

    // Lightness slider
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.l,
            selected_color.l,
            selected_color.into(),
            |l_mut| Oklch {
                l: l_mut,
                c,
                h,
                alpha: 1.0,
            },
            None::<fn(f32) -> Color32>,
            &"lightness",
            ui_identifier,
        )
        .on_hover_text("Lightness");
        ui.add_sized(
            [56.0, ui.spacing().interact_size.y],
            DragValue::new(&mut state.l)
                .prefix("L: ")
                .speed(0.003)
                .range(0.0..=1.0)
                .custom_formatter(|n, _| format!("{n:.03}")),
        )
        .on_hover_text("Lightness");
    });

    // Chroma slider
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.c,
            selected_color.c / 0.325,
            selected_color.into(),
            |c_mut| Oklch {
                l,
                c: c_mut * 0.325,
                h,
                alpha: 1.0,
            },
            None::<fn(f32) -> Color32>,
            &"chroma",
            ui_identifier,
        )
        .on_hover_text("Chroma");
        let mut chroma_val = state.c * 0.325;
        ui.add_sized(
            [56.0, ui.spacing().interact_size.y],
            DragValue::new(&mut chroma_val)
                .prefix("C: ")
                .speed(0.003)
                .range(0.0..=0.325)
                .custom_formatter(|n, _| format!("{n:.03}")),
        )
        .on_hover_text("Chroma");
        state.c = chroma_val / 0.325;
    });

    // Hue slider
    let determine_hue_slider_colors = |h_mut| {
        // Define lightness and chroma that exist for all hues
        let bg_color = oklch_to_color32(Oklch {
            l: 0.750,
            c: 0.127,
            h: h_mut * std::f32::consts::TAU,
            alpha: 1.0,
        });
        let bg_color = bg_color.blend(Color32::from_black_alpha(40));
        let color = Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 210);
        color
    };
    ui.horizontal(|ui| {
        color_slider_1d_grid(
            ui,
            &mut state.h,
            selected_color.h / std::f32::consts::TAU,
            selected_color.into(),
            |h_mut| Oklch {
                l: l,
                c: c,
                h: h_mut * std::f32::consts::TAU,
                alpha: 1.0,
            },
            Some(determine_hue_slider_colors),
            &"hue",
            ui_identifier,
        )
        .on_hover_text("Hue");
        let mut hue_val = state.h * 360.;
        ui.add_sized(
            [56.0, ui.spacing().interact_size.y],
            DragValue::new(&mut hue_val)
                .prefix("H: ")
                .speed(1.0)
                .range(0.0..=360.)
                .custom_formatter(|n, _| format!("{n:.1}")),
        )
        .on_hover_text("Hue");
        state.h = hue_val / 360.;
    });

    // Alpha slider
    if with_alpha {
        ui.horizontal(|ui| {
            color_slider_1d_grid(
                ui,
                &mut state.a,
                selected_color.alpha,
                selected_color.into(),
                |alpha| {
                    let mut color = selected_color;
                    color.alpha = alpha;
                    color
                },
                None::<fn(f32) -> Color32>,
                &"alpha",
                ui_identifier,
            )
            .on_hover_text("Transparency (Alpha)");
            ui.add_sized(
                [56.0, ui.spacing().interact_size.y],
                DragValue::new(&mut state.a)
                    .prefix("A: ")
                    .speed(0.003)
                    .range(0.0..=1.0)
                    .custom_formatter(|n, _| format!("{n:.03}")),
            )
            .on_hover_text("Transparency (Alpha)");
        });
    }

    let state_changed = state != original_state;

    ui.data_mut(|d| {
        d.insert_temp(state_id, state.clone());
    });

    (state_changed, state)
}
