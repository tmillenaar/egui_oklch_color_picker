use egui::*;

pub enum NumericEdit {
    None,
    Commit(String),
    Cancel,
}

#[derive(Clone, Default)]
struct NumericFieldState {
    text: String,
    select_all_next_frame: bool,
}

pub fn numeric_field(
    ui: &mut Ui,
    id: Id,
    prefix: &str,
    value: String,
) -> (Response, NumericEdit) {
    let mut state = ui
        .data(|d| d.get_temp::<NumericFieldState>(id))
        .unwrap_or_else(|| NumericFieldState {
            text: value.clone(),
            select_all_next_frame: false,
        });

    // If we're not editing, keep the displayed text in sync.
    if !ui.memory(|m| m.has_focus(id)) {
        state.text = value.clone();
    }

    // 1. Was the flag already set from the previous frame?
    let select_all = state.select_all_next_frame;
    state.select_all_next_frame = false;

    // 2. Build the widget
    let mut output = ui
        .horizontal(|ui| {
            ui.label(prefix);

            egui::TextEdit::singleline(&mut state.text)
                .desired_width(25.0)
                .id(id)
                .show(ui)
        })
        .inner;

    let response = output.response;

    // 3. If requested, perform the selection now.
    if select_all {
        use egui::text::{CCursor, CCursorRange};

        output.state.cursor.set_char_range(Some(
            CCursorRange::two(
                CCursor::new(0),
                CCursor::new(state.text.chars().count()),
            ),
        ));

        output.state.store(ui.ctx(), response.id);
    }

    // 4. If we *just* gained focus, arm it for next frame.
    if response.gained_focus() {
        state.select_all_next_frame = true;
    }

    let pasted = response.has_focus()
        && ui.input(|i| {
            i.events.iter().any(|e| matches!(e, egui::Event::Paste(_)))
        });
    let enter = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    let escape = ui.input(|i| i.key_pressed(Key::Escape));

    let commit = response.lost_focus() || enter || pasted;

    let result = if escape {
        state.text = value;
        NumericEdit::Cancel
    } else if commit {
        NumericEdit::Commit(state.text.clone())
    } else {
        NumericEdit::None
    };

    // Drop focus on element after pasting
    if pasted {
        ui.memory_mut(|mem| mem.surrender_focus(id));
        return (response, NumericEdit::Commit(state.text.clone()));
    }

    ui.data_mut(|d| d.insert_temp(id, state));

    (response, result)
}
