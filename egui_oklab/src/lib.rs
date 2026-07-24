pub mod oklab;
pub mod oklch;

mod color_projection;
mod conversions;
mod coordinate_transforms;
mod oklch_ui_state;
mod picker_2d;
mod supporting_widgets;
mod text_fields;

use crate::color_projection::*;
use crate::conversions::*;
use crate::coordinate_transforms::*;
use crate::oklab::*;
use crate::oklch::*;
use crate::oklch_ui_state::*;
use crate::picker_2d::*;
use crate::supporting_widgets::color_pane_2d::*;
use crate::supporting_widgets::oklch_sliders::*;
use crate::supporting_widgets::show_color::*;
use crate::text_fields::color_text_fields::*;

use std::f32::consts::TAU;

use egui::*;
use epaint::{
    StrokeKind,
    ecolor::{Color32, Rgba},
    vec2,
};

fn color_button(ui: &mut Ui, srgba: [u8; 4], open: bool) -> Response {
    let size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::new(WidgetType::ColorButton));

    if ui.is_rect_visible(rect) {
        let visuals = if open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&response)
        };
        let rect = rect.expand(visuals.expansion);

        let stroke_width = 1.0;
        show_srgba_unmultiplied_at(ui.painter(), srgba, rect.shrink(stroke_width), 3);

        let corner_radius = visuals.corner_radius.at_most(2); // Can't do more rounding because the background grid doesn't do any rounding
        ui.painter().rect_stroke(
            rect,
            corner_radius,
            (stroke_width, visuals.bg_fill), // Using fill for stroke is intentional, because default style has no border
            StrokeKind::Inside,
        );
    }

    response
}

#[derive(Debug, Clone)]
enum OKLabPlotType {
    StableLightness,
    StableChroma,
    StableHue,
}

pub fn color_edit_button_oklch(ui: &mut Ui, color: &mut Oklch, widget_width: f32) -> Response {
    let mut response = color_button(
        ui,
        color.to_srgba_unmultiplied(),
        Popup::is_id_open(ui.ctx(), ui.auto_id_with("popup")),
    );

    if ui.style().explanation_tooltips {
        response = response.on_hover_text("Click to edit color");
    }

    if color_popup(&response, color, widget_width, false) {
        response.mark_changed();
    }

    response
}

fn color_popup(
    response: &Response,
    color: &mut Oklch,
    widget_width: f32,
    with_alpha: bool,
) -> bool {
    let popup_id = response.id.with("popup");

    let mut changed = false;
    let just_opened = response.clicked();

    Popup::menu(response)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = widget_width;

            if color_picker_oklch_2d(ui, color, widget_width, with_alpha, just_opened) {
                changed = true;
            }
        });

    changed
}

pub fn color_edit_button_oklcha(ui: &mut Ui, color: &mut Oklch, widget_width: f32) -> Response {
    let mut response = color_button(
        ui,
        color.to_srgba_unmultiplied(),
        Popup::is_id_open(ui.ctx(), ui.auto_id_with("popup")),
    );

    if ui.style().explanation_tooltips {
        response = response.on_hover_text("Click to edit color");
    }

    if color_popup(&response, color, widget_width, true) {
        response.mark_changed();
    }

    response
}

pub fn color_popup_oklcha_simple(
    response: &Response,
    color: &mut Oklch,
    widget_width: f32,
) -> bool {
    color_popup_oklcha_advanced(
        response,
        response.clicked(),
        response.clicked(),
        color,
        widget_width,
    )
}

pub fn color_popup_oklch_simple(response: &Response, color: &mut Oklch, widget_width: f32) -> bool {
    color_popup_oklch_advanced(
        response,
        response.clicked(),
        response.clicked(),
        color,
        widget_width,
    )
}

pub fn color_popup_oklcha_advanced(
    response: &Response,
    open: bool,
    reset_state: bool,
    color: &mut Oklch,
    widget_width: f32,
) -> bool {
    let mut changed = false;

    let popup_id = response.id.with("popup");
    if open {
        Popup::open_id(&response.ctx, popup_id);
    }
    Popup::from_response(response)
        .id(popup_id)
        .open_memory(None)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = widget_width;

            if color_picker_oklch_2d(ui, color, widget_width, true, reset_state) {
                changed = true;
            }
        });

    changed
}

pub fn color_popup_oklch_advanced(
    response: &Response,
    open: bool,
    reset_state: bool,
    color: &mut Oklch,
    widget_width: f32,
) -> bool {
    let popup_id = response.id.with("popup");

    if open {
        Popup::open_id(&response.ctx, popup_id);
    }

    let mut changed = false;

    Popup::from_response(response)
        .id(popup_id)
        .open_memory(None)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = widget_width;

            if color_picker_oklch_2d(ui, color, widget_width, false, reset_state) {
                changed = true;
            }
        });

    changed
}

fn plot_type_selector(
    ui: &mut Ui,
    ui_identifier: *const Oklch, // use pointer to color in RAM as identifier to allow for multiple color panels. They may share state if they refer to the same color
) -> (bool, OKLabPlotType) {
    let mut tab_changed = false;

    let tab_id = ui.id().with(("plot_type", ui_identifier));
    let mut tab = ui.data(|d| {
        d.get_temp::<OKLabPlotType>(tab_id)
            .unwrap_or(OKLabPlotType::StableHue)
    });

    ui.horizontal(|ui| {
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableHue), "Stable Hue")
            .clicked()
        {
            tab_changed = true;
            tab = OKLabPlotType::StableHue;
        }
        if ui
            .selectable_label(matches!(tab, OKLabPlotType::StableChroma), "Stable Chroma")
            .clicked()
        {
            tab_changed = true;
            tab = OKLabPlotType::StableChroma;
        }
        if ui
            .selectable_label(
                matches!(tab, OKLabPlotType::StableLightness),
                "Stable Lightness",
            )
            .clicked()
        {
            tab_changed = true;
            tab = OKLabPlotType::StableLightness;
        }
    });

    ui.data_mut(|d| {
        d.insert_temp(tab_id, tab.clone());
    });

    (tab_changed, tab)
}

pub fn color_picker_panel_oklch_2d(
    ui: &mut Ui,
    plot_type: OKLabPlotType,
    ui_identifier: *const Oklch, // use pointer to color in RAM as identifier to allow for multiple color panels. They may share state if they refer to the same color
) -> (bool, Oklch) {
    // Recover state from cache
    let state_id = ui.id().with(("oklch_ui_state", ui_identifier.clone()));
    let mut state: OklchUiState =
        ui.data(|d| d.get_temp::<OklchUiState>(state_id))
            .unwrap_or(OklchUiState {
                l: 0.5,
                c: 0.3,
                h: 0.0,
                a: 1.0,
            });

    let state_original = state.clone();

    let (mut new_color, color_changed) = match plot_type {
        OKLabPlotType::StableHue => {
            let projection = StableHueProjection {
                hue: state.h * std::f32::consts::TAU,
            };

            let mut coord = CartesianCoord {
                x: state.c,
                y: state.l,
            };

            let (_response, update_color) =
                color_slider_2d::<CartesianTransform, StableHueProjection>(
                    ui,
                    &mut coord,
                    projection,
                    ("StableHueColorPicker", ui_identifier.clone()),
                );

            state.c = coord.x.min(1.0);
            state.l = coord.y.min(1.0);

            let picker_id = ui
                .id()
                .with(("StableHueColorPicker", ui_identifier.clone()));

            let picker = ui
                .data(|d| d.get_temp::<StableHuePicker>(picker_id))
                .unwrap();

            (picker.nearest_valid_color(coord), update_color)
        }

        OKLabPlotType::StableChroma => {
            let projection = StableChromaProjection { chroma: state.c };

            let mut coord = PolarCoord {
                angle: state.h * std::f32::consts::TAU,
                radius: 1.0 - state.l,
            };

            let (_response, update_color) = color_slider_2d::<PolarTransform, StableChromaProjection>(
                ui,
                &mut coord,
                projection,
                ("StableChromaColorPicker", ui_identifier.clone()),
            );

            state.h = coord.angle.rem_euclid(360.0) / std::f32::consts::TAU;
            state.l = 1.0 - coord.radius.min(1.0);

            let picker_id = ui
                .id()
                .with(("StableChromaColorPicker", ui_identifier.clone()));

            let picker = ui
                .data(|d| d.get_temp::<StableChromaPicker>(picker_id))
                .unwrap();

            (picker.nearest_valid_color(coord), update_color)
        }

        OKLabPlotType::StableLightness => {
            let projection = StableLightnessProjection { lightness: state.l };

            let mut coord = PolarCoord {
                angle: state.h * std::f32::consts::TAU,
                radius: state.c,
            };

            let (_response, update_color) =
                color_slider_2d::<PolarTransform, StableLightnessProjection>(
                    ui,
                    &mut coord,
                    projection,
                    ("StableLightnessColorPicker", ui_identifier.clone()),
                );

            state.h = (coord.angle / std::f32::consts::TAU).rem_euclid(360.0);
            state.c = coord.radius.min(1.0);

            let picker_id = ui
                .id()
                .with(("StableLightnessColorPicker", ui_identifier.clone()));

            let picker = ui
                .data(|d| d.get_temp::<StableLightnessPicker>(picker_id))
                .unwrap();

            (picker.nearest_valid_color(coord), update_color)
        }
    };

    if color_changed {
        ui.data_mut(|d| {
            d.insert_temp(state_id, state);
        });
    } else {
        // Note: We put back the original state in case the user did not click the
        //       viewport to prevent rounding of state to the nearest pixel in case
        //       state was set before by a different widget.
        ui.data_mut(|d| {
            d.insert_temp(state_id, state_original);
        });
    }

    (color_changed, new_color)
}

pub fn color_picker_oklch_2d(
    ui: &mut Ui,
    color: &mut Oklch,
    widget_width: f32,
    with_alpha: bool,
    reset_state: bool,
) -> bool {
    ui.spacing_mut().slider_width = widget_width;

    let ui_identifier = std::ptr::from_ref(color).clone();

    let state_id = ui.id().with(("oklch_ui_state", ui_identifier));

    let alpha_state = if with_alpha { color.alpha } else { 1.0 };
    let state_from_color = OklchUiState {
        l: color.l,
        c: color.c / 0.325,
        h: color.h / TAU,
        a: alpha_state,
    };
    let mut state: OklchUiState = ui
        .data(|d| d.get_temp::<OklchUiState>(state_id))
        .unwrap_or(state_from_color.clone());

    if reset_state {
        state = state_from_color;
    }

    ui.data_mut(|d| {
        d.insert_temp(state_id, state);
    });

    let (tab_changed, plot_type) = plot_type_selector(ui, ui_identifier);

    let current_color_size = vec2(
        ui.spacing().slider_width + 64.,
        ui.spacing().interact_size.y,
    );
    show_srgba_unmultiplied(ui, color.to_srgba_unmultiplied(), current_color_size, 3)
        .on_hover_text("Selected color");

    let (color_changed_in_2d_panel, new_color_opague) =
        color_picker_panel_oklch_2d(ui, plot_type, ui_identifier);

    let (color_changed_by_sliders, new_state) =
        oklch_sliders(ui, new_color_opague, with_alpha, ui_identifier);

    if color_changed_in_2d_panel | tab_changed {
        let mut new_color = new_color_opague.clone();
        new_color.alpha = new_state.a;
        *color = new_color;
    }
    if color_changed_by_sliders {
        let oklch = Oklch {
            l: new_state.l,
            c: new_state.c * 0.325,
            h: new_state.h * std::f32::consts::TAU,
            alpha: new_state.a,
        };
        if oklch.is_valid_srgb() {
            *color = oklch;
        } else {
            // TODO: Fallback color using 2d panel
            let mut new_color = new_color_opague.clone();
            new_color.alpha = new_state.a;
            *color = new_color;
        }
    }

    let (color_changed_by_text_field, new_color_from_text) = color_text_fields(
        ui,
        Color32::from(new_color_opague),
        widget_width,
        ui_identifier,
    );
    if color_changed_by_text_field {
        // Also update state indicators when color changed from direct text inserts
        let state_id = ui.id().with(("oklch_ui_state", ui_identifier));
        let lch = color32_to_oklch(new_color_from_text);
        let state = OklchUiState {
            l: lch.l,
            c: lch.c / 0.325,
            h: lch.h / std::f32::consts::TAU,
            a: new_state.a,
        };
        let rgba = new_color_from_text.to_array();
        *color =
            Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], (new_state.a * 255.) as u8)
                .into();
        ui.data_mut(|d| {
            d.insert_temp(state_id, state);
        });
    };

    let color_changed = color_changed_in_2d_panel
        | color_changed_by_sliders
        | color_changed_by_text_field
        | tab_changed;

    color_changed
}
