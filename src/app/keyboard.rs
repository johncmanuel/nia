use super::{App, Format, Message, update};
use crate::keybindings::{Shortcut, Slot};
use iced::Task;
use iced::keyboard::{Key, Modifiers, key::Named};
use iced::widget::{operation, pane_grid};

pub(super) fn handle(state: &mut App, key: Key, modifiers: Modifiers) -> Task<Message> {
    if state.pending_close.is_some() {
        if key == Key::Named(Named::Escape) {
            return update(state, Message::CancelClose);
        }
        return Task::none();
    }
    if let Some(slot) = state.recording {
        if key == Key::Named(Named::Escape) {
            state.recording = None;
            state.notice = None;
        } else if let Some(shortcut) = Shortcut::from_event(&key, modifiers) {
            match state.keybindings.assign(slot, shortcut) {
                Ok(()) => {
                    state.recording = None;
                    state.notice = state.keybindings.save().err().map(|error| {
                        log::warn!("keybindings save failed: {:?}", error.kind());
                        format!("Could not save keybindings: {error}")
                    });
                }
                Err(conflict) => {
                    state.notice = Some(format!(
                        "{} is already assigned to {}.",
                        shortcut.label(),
                        conflict.label()
                    ))
                }
            }
        }
    } else if key == Key::Named(Named::Escape) {
        if state.show_theme_picker {
            return update(state, Message::HideThemePicker);
        }
        if state.show_search {
            return update(state, Message::HideSearch);
        }
        if state.show_format_menu {
            state.show_format_menu = false;
            return operation::focus(super::EDITOR);
        }
        if state.show_settings {
            return update(state, Message::ShowEditor);
        }
        if state.handbook.is_some() {
            return update(state, Message::HideHandbook);
        }
        if state.zen_mode {
            return update(state, Message::ToggleZen);
        }
        if state.show_recent {
            return update(state, Message::HideRecent);
        }
    } else if state.handbook.is_some() {
        return Task::none();
    } else if state.show_outline && key == Key::Named(Named::ArrowDown) {
        return update(state, Message::OutlineNext);
    } else if state.show_outline && key == Key::Named(Named::ArrowUp) {
        return update(state, Message::OutlinePrevious);
    } else if state.show_recent && key == Key::Named(Named::ArrowDown) {
        return update(state, Message::RecentNext);
    } else if state.show_recent && key == Key::Named(Named::ArrowUp) {
        return update(state, Message::RecentPrevious);
    } else if let Some(message) = state.key_message(&key, modifiers) {
        return update(state, message);
    }
    Task::none()
}

fn shortcut_message(slot: Slot) -> Message {
    match slot {
        Slot::New => Message::New,
        Slot::NewTab => Message::NewGroup,
        Slot::Open => Message::Open,
        Slot::Save | Slot::AlternateSave => Message::Save,
        Slot::Find | Slot::AlternateFind => Message::ShowSearch,
        Slot::FindTabs => Message::ShowTabSearch,
        Slot::Outline => Message::ShowOutline,
        Slot::Recent => Message::ShowRecent,
        Slot::CloseTab => Message::CloseCurrentTab,
        Slot::CloseGroup => Message::CloseCurrentGroup,
        Slot::NextTab => Message::NextPane,
        Slot::PreviousTab => Message::PreviousPane,
        Slot::SinglePane => Message::SinglePane,
        Slot::SplitRight => Message::SplitPane(pane_grid::Axis::Vertical),
        Slot::SplitDown => Message::SplitPane(pane_grid::Axis::Horizontal),
        Slot::IncreaseFontSize => Message::IncreaseFontSize,
        Slot::DecreaseFontSize => Message::DecreaseFontSize,
        Slot::Settings => Message::ShowSettings,
        Slot::Zen => Message::ToggleZen,
        Slot::Bold => Message::Format(Format::Bold),
        Slot::Italic => Message::Format(Format::Italic),
        Slot::Undo => Message::Undo,
        Slot::Redo => Message::Redo,
    }
}

impl App {
    pub(crate) fn key_message(&self, key: &Key, modifiers: Modifiers) -> Option<Message> {
        if let Some(slot) = self.keybindings.matching(key, modifiers) {
            if slot == Slot::Outline && self.show_outline {
                return Some(Message::HideOutline);
            }
            return Some(shortcut_message(slot));
        }
        if modifiers != Modifiers::COMMAND {
            return None;
        }
        let Key::Character(value) = key else {
            return None;
        };
        let number = value.parse::<usize>().ok()?;
        let index = match number {
            1..=8 => number - 1,
            9 => self.groups.len().checked_sub(1)?,
            _ => return None,
        };
        (index < self.groups.len()).then_some(Message::SelectGroup(index))
    }
}
