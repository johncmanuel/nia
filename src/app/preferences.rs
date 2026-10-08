use super::{App, EDITOR, Message};
use crate::keybindings::Keybindings;
use iced::Task;
use iced::theme::Base;
use iced::widget::operation;

pub(super) fn handle(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::ToggleZen => {
            state.zen_mode = !state.zen_mode;
            if state.show_settings {
                return Task::none();
            }
            return operation::focus(EDITOR);
        }
        Message::SetHighlightLine(enabled) => {
            state.focus.highlight_line = enabled;
            save_focus(state);
        }
        Message::SetDimming(dimming) => {
            state.focus.dimming = dimming;
            save_focus(state);
        }
        Message::RecordShortcut(slot) => {
            state.recording = Some(slot);
            state.notice = None;
        }
        Message::ResetKeybindings => {
            state.keybindings = Keybindings::default();
            state.recording = None;
            state.notice = state
                .keybindings
                .save()
                .err()
                .map(|error| format!("Could not save keybindings: {error}"));
            if state.notice.is_some() {
                log::warn!("keybindings save failed");
            }
        }
        Message::SetTheme(theme) => {
            state.show_theme_picker = false;
            state.notice = crate::theme::save(&theme)
                .err()
                .map(|error| format!("Could not save theme: {error}"));
            if state.notice.is_some() {
                log::warn!("theme save failed");
            } else {
                log::info!("theme changed to {}", theme.name());
            }
            state.theme = theme;
        }
        Message::SetTransparentBackground(enabled) => {
            state.notice = crate::theme::save_transparency(enabled)
                .err()
                .map(|error| format!("Could not save transparency setting: {error}"));
            if state.notice.is_some() {
                log::warn!("transparency setting save failed");
            }
            state.transparent_background = enabled;
        }
        Message::ToggleThemePicker => {
            state.show_theme_picker = !state.show_theme_picker;
        }
        Message::HideThemePicker => {
            state.show_theme_picker = false;
        }
        Message::SetFont(family) => {
            if state.font_families.contains(&family) {
                state.editor_font = crate::font::font_for(&family);
                state.font_family = Some(family);
                state.notice = crate::font::save(state.font_family.as_deref())
                    .err()
                    .map(|error| format!("Could not save font setting: {error}"));
            }
        }
        Message::ResetFont => {
            state.editor_font = iced::Font::MONOSPACE;
            state.font_family = None;
            state.notice = crate::font::save(None)
                .err()
                .map(|error| format!("Could not save font setting: {error}"));
        }
        Message::IncreaseFontSize | Message::DecreaseFontSize => {
            state.editor_font_size = if matches!(message, Message::IncreaseFontSize) {
                state
                    .editor_font_size
                    .saturating_add(1)
                    .min(crate::font::MAX_SIZE)
            } else {
                state
                    .editor_font_size
                    .saturating_sub(1)
                    .max(crate::font::MIN_SIZE)
            };
            state.notice = crate::font::save_size(state.editor_font_size)
                .err()
                .map(|error| format!("Could not save font size: {error}"));
        }
        _ => unreachable!("non-preference message routed to preferences"),
    }
    Task::none()
}

fn save_focus(state: &mut App) {
    state.notice = state
        .focus
        .save()
        .err()
        .map(|error| format!("Could not save focus settings: {error}"));
    if state.notice.is_some() {
        log::warn!("focus settings save failed");
    }
}
