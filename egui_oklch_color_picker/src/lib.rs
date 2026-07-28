//! # egui OKLCH Color Picker
//!
//! This is an interactive color picker widget for `egui` in the OKLCH color space featuring perceptually
//! uniform color editing, multiple 2D projection modes, sliders, text input,
//! and popup color editors.
//!
//! This crate is intended for applications that want perceptually uniform
//! color selection, unlike traditional RGB or HSV pickers.
//!
//! ## Features
//!
//! - Interactive 2D OKLCH color picker
//! - Color editor widgets either embedded or as popup window
//! - Optional alpha channel editing
//! - Multiple visualization modes:
//!   - Stable Hue
//!   - Stable Chroma
//!   - Stable Lightness
//! - Perceptually uniform editing using OKLCH
//! - Hex/color text input
//! - Native `egui` integration, including interoperability with wgui::Color32
//!
//! ## Main Types
//!
//! - [`Oklch`] — OKLCH color representation.
//! - [`Oklab`] — OKLab color representation.
//!
//! ## Main Widgets
//!
//! - [`color_picker_oklch_2d`] — Embedded 2D color picker widget.
//! - [`color_edit_button_oklch` / [`color_edit_button_oklcha`] — Color button that opens an OKLCH popup editor.
//! - [`color_popup_oklch_simple`] / [`color_popup_oklcha_simple`] — Popup OKLCH editor for arbitrary egui elements.
//! - [`color_popup_oklch_advanced`] / [`color_popup_oklcha_advanced`] — Same as above with explicit open/reset control.
//!
//! # Examples
//!
//! This crate ships with an interactive demonstration application showing the
//! primary integration styles. The live demo is hosted at <https://www.timomillenaar.nl/oklch_color_picker>.
//! The code for the examples is documented in the [`egui_oklch_color_picker_demo`](../egui_oklch_color_picker_demo/index.html) create.
//!
//! ## State
//!
//! Widget state is stored using `egui`'s temporary memory. Each picker should
//! be given a unique `ui_identifier` when multiple pickers exist in the same UI
//! so that their interaction state remains independent.
//!
//! ## Color Gamut
//!
//! The OKLCH color space can represent colors outside the sRGB gamut. When the
//! selected color is not representable in sRGB, the picker computes and displays
//! the nearest valid fallback color while preserving as much of the user's
//! intent as possible.
//!
//! ## Modules
//!
//! - [`oklab`] — OKLab color types and conversions.
//! - [`oklch`] — OKLCH color types and conversions.

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
    ecolor::{Color32, Rgba},
    vec2, StrokeKind,
};

fn color_button(ui: &mut Ui, oklch: Oklch, open: bool) -> Response {
    let size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::new(WidgetType::ColorButton));

    let srgba = oklch.to_srgba_unmultiplied();

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

/// Displays a color button that opens an interactive OKLCH color picker.
///
/// The button shows the current color and, when clicked, opens a popup
/// containing the full 2D OKLCH color picker with sliders and text input.
///
/// This variant edits only the color components (`L`, `C`, and `H`). The
/// alpha channel is preserved but cannot be modified by the picker.
///
/// # Arguments
///
/// * `ui` - The `egui` UI to add the widget to.
/// * `color` - The color to display and edit.
/// * `widget_width` - Preferred width of the controls inside the popup.
/// * `ui_identifier` - A unique identifier used to store persistent picker
///   state in `egui`. Use a different identifier for each picker instance.
///
/// # Returns
///
/// Returns the widget's [`Response`]. The response is marked as changed when
/// the user modifies the selected color.
pub fn color_edit_button_oklch(
    ui: &mut Ui,
    color: &mut Oklch,
    widget_width: f32,
    ui_identifier: impl std::hash::Hash,
) -> Response {
    let mut response = color_button(
        ui,
        *color,
        Popup::is_id_open(ui.ctx(), ui.auto_id_with("popup")),
    );

    if ui.style().explanation_tooltips {
        response = response.on_hover_text("Click to edit color");
    }

    if color_popup(&response, color, widget_width, false, ui_identifier) {
        response.mark_changed();
    }

    response
}

fn color_popup(
    response: &Response,
    color: &mut Oklch,
    widget_width: f32,
    with_alpha: bool,
    ui_identifier: impl std::hash::Hash,
) -> bool {
    let popup_id = response.id.with("popup");

    let mut changed = false;
    let just_opened = response.clicked();

    Popup::menu(response)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = widget_width;

            if color_picker_oklch_2d(
                ui,
                color,
                widget_width,
                with_alpha,
                just_opened,
                ui_identifier,
            ) {
                changed = true;
            }
        });

    changed
}

/// Displays a color button that opens an interactive OKLCHA color picker.
///
/// This behaves identically to [`color_edit_button_oklch`], but also exposes
/// an alpha slider, allowing the opacity of the selected color to be edited.
///
/// # Arguments
///
/// * `ui` - The `egui` UI to add the widget to.
/// * `color` - The color to display and edit.
/// * `widget_width` - Preferred width of the controls inside the popup.
/// * `ui_identifier` - A unique identifier used to store persistent picker
///   state in `egui`.
///
/// # Returns
///
/// Returns the widget's [`Response`]. The response is marked as changed when
/// the user modifies the selected color or its alpha channel.
pub fn color_edit_button_oklcha(
    ui: &mut Ui,
    color: &mut Oklch,
    widget_width: f32,
    ui_identifier: impl std::hash::Hash,
) -> Response {
    let mut response = color_button(
        ui,
        *color,
        Popup::is_id_open(ui.ctx(), ui.auto_id_with("popup")),
    );

    if ui.style().explanation_tooltips {
        response = response.on_hover_text("Click to edit color");
    }

    if color_popup(&response, color, widget_width, true, ui_identifier) {
        response.mark_changed();
    }

    response
}

/// Opens an OKLCH popup editor anchored to an existing widget.
///
/// The popup automatically opens when `response.clicked()` is true and resets
/// its internal state each time it is opened.
///
/// This helper is suitable for the common case where the popup should simply
/// appear when the associated widget is clicked.
///
/// # Arguments
///
/// * `response` - The response of the widget that owns the popup.
/// * `color` - The color to edit.
/// * `widget_width` - Preferred width of the picker controls.
/// * `ui_identifier` - Unique identifier for the picker state.
///
/// # Returns
///
/// Returns `true` if the color was modified.
pub fn color_popup_oklcha_simple(
    response: &Response,
    color: &mut Oklch,
    widget_width: f32,
    ui_identifier: impl std::hash::Hash,
) -> bool {
    color_popup_oklcha_advanced(
        response,
        response.clicked(),
        response.clicked(),
        color,
        widget_width,
        ui_identifier,
    )
}

/// Opens an OKLCHA popup editor anchored to an existing widget.
///
/// This is the alpha-enabled version of [`color_popup_oklch_simple`]. The
/// popup automatically opens when the associated widget is clicked and resets
/// its internal state whenever it is reopened.
///
/// # Arguments
///
/// * `response` - The response of the widget that owns the popup.
/// * `color` - The color to edit.
/// * `widget_width` - Preferred width of the picker controls.
/// * `ui_identifier` - Unique identifier for the picker state.
///
/// # Returns
///
/// Returns `true` if the color or alpha value was modified.
pub fn color_popup_oklch_simple(
    response: &Response,
    color: &mut Oklch,
    widget_width: f32,
    ui_identifier: impl std::hash::Hash,
) -> bool {
    color_popup_oklch_advanced(
        response,
        response.clicked(),
        response.clicked(),
        color,
        widget_width,
        ui_identifier,
    )
}

/// Displays an OKLCH popup editor with explicit control over its lifetime.
///
/// Unlike [`color_popup_oklch_simple`], this function allows callers to
/// independently control whether the popup should be opened and whether its
/// internal interaction state should be reset.
///
/// This is useful when the popup is driven by application state instead of a
/// simple button click.
///
/// # Arguments
///
/// * `response` - The response of the widget that owns the popup.
/// * `open` - Opens the popup when `true`.
/// * `reset_state` - Resets the picker's cached interaction state when `true`.
/// * `color` - The color to edit.
/// * `widget_width` - Preferred width of the picker controls.
/// * `ui_identifier` - Unique identifier for the picker state.
///
/// # Returns
///
/// Returns `true` if the selected color changed.
pub fn color_popup_oklcha_advanced(
    response: &Response,
    open: bool,
    reset_state: bool,
    color: &mut Oklch,
    widget_width: f32,
    ui_identifier: impl std::hash::Hash,
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

            if color_picker_oklch_2d(ui, color, widget_width, true, reset_state, ui_identifier) {
                changed = true;
            }
        });

    changed
}

/// Displays an OKLCHA popup editor with explicit control over its lifetime.
///
/// This is the alpha-enabled version of
/// [`color_popup_oklch_advanced`]. In addition to editing the OKLCH color
/// components, it also allows the opacity to be modified.
///
/// # Arguments
///
/// * `response` - The response of the widget that owns the popup.
/// * `open` - Opens the popup when `true`.
/// * `reset_state` - Resets the picker's cached interaction state when `true`.
/// * `color` - The color to edit.
/// * `widget_width` - Preferred width of the picker controls.
/// * `ui_identifier` - Unique identifier for the picker state.
///
/// # Returns
///
/// Returns `true` if the selected color or alpha value changed.
pub fn color_popup_oklch_advanced(
    response: &Response,
    open: bool,
    reset_state: bool,
    color: &mut Oklch,
    widget_width: f32,
    ui_identifier: impl std::hash::Hash,
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

            if color_picker_oklch_2d(ui, color, widget_width, false, reset_state, ui_identifier) {
                changed = true;
            }
        });

    changed
}

fn plot_type_selector(ui: &mut Ui, ui_identifier: impl std::hash::Hash) -> (bool, OKLabPlotType) {
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

/// Displays the embedded OKLCH color picker panel.
///
/// This is the core implementation used by [`color_picker_oklch_2d`] and its
/// popup variants. It renders the picker directly into the provided `Ui`
/// without managing popup behavior or surrounding layout.
///
/// The selected color is edited in place through the provided [`Oklch`] value.
/// A stable `ui_identifier` should be supplied for each independent picker
/// instance to preserve its interaction state between frames.
///
/// Returns `true` if the selected color was modified during this frame.
pub fn color_picker_panel_oklch_2d(
    ui: &mut Ui,
    plot_type: OKLabPlotType,
    ui_identifier: egui::Id,
) -> (bool, Oklch) {
    // Recover state from cache
    let state_id = ui_identifier.with("oklch_ui_state");
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
                    ("StableHueColorPicker", ui_identifier),
                );

            state.c = coord.x.min(1.0);
            state.l = coord.y.min(1.0);

            let picker_id = ui.id().with(("StableHueColorPicker", ui_identifier));

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
                ("StableChromaColorPicker", ui_identifier),
            );

            state.h = coord.angle.rem_euclid(360.0) / std::f32::consts::TAU;
            state.l = 1.0 - coord.radius.min(1.0);

            let picker_id = ui.id().with(("StableChromaColorPicker", ui_identifier));

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
                    ("StableLightnessColorPicker", ui_identifier),
                );

            state.h = (coord.angle / std::f32::consts::TAU).rem_euclid(360.0);
            state.c = coord.radius.min(1.0);

            let picker_id = ui.id().with(("StableLightnessColorPicker", ui_identifier));

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

/// Displays the full embedded OKLCH color picker.
///
/// The picker combines:
///
/// - a 2D color selection viewport,
/// - multiple projection modes,
/// - OKLCH sliders,
/// - optional alpha editing,
/// - hexadecimal color input,
/// - and a preview of the selected color.
///
/// Unlike the popup helpers, this function renders the picker directly into
/// the current UI, making it suitable for settings panels, inspectors, and
/// dedicated color editing windows.
///
/// Internal interaction state is cached using `egui`'s temporary memory.
/// Supplying a stable and unique `ui_identifier` ensures multiple picker
/// instances maintain independent state.
///
/// # Arguments
///
/// * `ui` - The `egui` UI to add the picker to.
/// * `color` - The color being edited.
/// * `widget_width` - Preferred width of the slider controls.
/// * `with_alpha` - Whether to display an alpha slider.
/// * `reset_state` - Whether to synchronize the cached picker state with
///   `color` before rendering.
/// * `ui_identifier` - Unique identifier used for caching picker state.
///
/// # Returns
///
/// Returns `true` if the user modified the selected color.
pub fn color_picker_oklch_2d(
    ui: &mut Ui,
    color: &mut Oklch,
    widget_width: f32,
    with_alpha: bool,
    reset_state: bool,
    ui_identifier: impl std::hash::Hash,
) -> bool {
    let ui_identifier = ui.id().with(ui_identifier);

    ui.spacing_mut().slider_width = widget_width;

    let state_id = ui_identifier.with("oklch_ui_state");

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

    let color_from_state = state.to_oklch();

    ui.data_mut(|d| {
        d.insert_temp(state_id, state);
    });

    let (tab_changed, plot_type) = plot_type_selector(ui, ui_identifier);

    let current_color_size = vec2(
        ui.spacing().slider_width + 64.,
        ui.spacing().interact_size.y,
    );
    let label = if color_from_state.is_valid_srgb() {
        None
    } else {
        Some("Fallback color!")
    };
    let color_display_response = show_srgba_unmultiplied(
        ui,
        color.to_srgba_unmultiplied(),
        current_color_size,
        3,
        label,
    )
    .on_hover_text("Selected color -- click to snap to this color (snap to fallback)");

    if color_display_response.clicked() {
        state = OklchUiState {
            l: color.l,
            c: color.c / 0.325,
            h: color.h / TAU,
            a: if with_alpha { color.alpha } else { 1.0 },
        };
        ui.data_mut(|d| {
            d.insert_temp(state_id, state);
        });
    }

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
        let state_id = ui_identifier.with("oklch_ui_state");
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
