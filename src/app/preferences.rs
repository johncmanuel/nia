use super::{App, EDITOR, Message};
use crate::keybindings::Keybindings;
use iced::Task;
use iced::theme::Base;
use iced::widget::operation;
use std::io;

pub(super) fn handle(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::ToggleZen => {
            state.zen_mode = !state.zen_mode;
            state.show_format_menu = false;
            if state.show_settings {
                return Task::none();
            }
            return operation::focus(EDITOR);
        }
        Message::SetHighlightLine(enabled) => {
            state.focus.highlight_line = enabled;
            save_focus(state);
        }
        Message::SetInactivePaneDim(dim) => {
            state.focus.inactive_pane_dim = dim.min(100);
            save_focus(state);
        }
        Message::RecordShortcut(slot) => {
            state.recording = Some(slot);
            state.notice = None;
        }
        Message::ResetKeybindings => {
            state.keybindings = Keybindings::default();
            state.recording = None;
            state.notice = setting_error(state.keybindings.save(), "keybindings");
        }
        Message::SetTheme(theme) => {
            state.show_theme_picker = false;
            state.notice = setting_error(crate::theme::save(&theme), "theme");
            if state.notice.is_none() {
                log::info!("theme changed to {}", theme.name());
            }
            state.theme = theme;
        }
        Message::SetTransparentBackground(enabled) => {
            state.transparent_background = enabled;
            save_transparency(state);
        }
        Message::SetBackgroundOpacity(opacity) => {
            state.background_opacity = opacity;
            save_transparency(state);
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
                state.notice = setting_error(
                    crate::font::save(state.font_family.as_deref()),
                    "font setting",
                );
            } else {
                log::warn!("font selection ignored: unavailable family");
            }
        }
        Message::ResetFont => {
            state.editor_font = iced::Font::MONOSPACE;
            state.font_family = None;
            state.notice = setting_error(crate::font::save(None), "font setting");
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
            state.notice =
                setting_error(crate::font::save_size(state.editor_font_size), "font size");
        }
        _ => unreachable!("non-preference message routed to preferences"),
    }
    Task::none()
}

fn save_focus(state: &mut App) {
    state.notice = setting_error(state.focus.save(), "focus settings");
}

fn save_transparency(state: &mut App) {
    state.notice = setting_error(
        crate::theme::save_transparency(state.transparent_background, state.background_opacity),
        "transparency setting",
    );
}

fn setting_error(result: io::Result<()>, name: &str) -> Option<String> {
    result.err().map(|error| {
        log::warn!("{name} save failed: {:?}", error.kind());
        format!("Could not save {name}: {error}")
    })
}
