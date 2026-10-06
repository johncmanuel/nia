use super::{App, EDITOR, Message, OUTLINE, RECENT, SEARCH, open_document, update};
use crate::keybindings::{Keybindings, Shortcut};
use iced::Task;
use iced::keyboard::{Key, key::Named};
use iced::widget::{operation, text_editor};

pub(super) fn handle(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::ShowKeybindings => {
            state.show_layout = false;
            state.show_font_picker = false;
            state.show_keybindings = true;
            state.recording = None;
            state.clear_search();
            state.notice = None;
        }
        Message::ShowEditor => {
            state.show_keybindings = false;
            state.recording = None;
            state.notice = None;
            return operation::focus(EDITOR);
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
            state.theme = theme;
            state.notice = theme
                .save()
                .err()
                .map(|error| format!("Could not save theme: {error}"));
            if state.notice.is_some() {
                log::warn!("theme save failed");
            } else {
                log::info!("theme changed to {}", theme.label());
            }
        }
        Message::ToggleFontPicker => {
            state.show_font_picker = !state.show_font_picker;
            if state.show_font_picker {
                state.show_layout = false;
                state.show_search = false;
                state.show_outline = false;
                state.show_recent = false;
                state.show_keybindings = false;
            }
        }
        Message::SetFont(family) => {
            if state.font_families.contains(&family) {
                state.editor_font = crate::font::font_for(&family);
                state.font_family = Some(family);
                state.show_font_picker = false;
                state.notice = crate::font::save(state.font_family.as_deref())
                    .err()
                    .map(|error| format!("Could not save font setting: {error}"));
            }
        }
        Message::ResetFont => {
            state.editor_font = iced::Font::MONOSPACE;
            state.font_family = None;
            state.show_font_picker = false;
            state.notice = crate::font::save(None)
                .err()
                .map(|error| format!("Could not save font setting: {error}"));
        }
        Message::ShowSearch => {
            state.show_layout = false;
            state.show_font_picker = false;
            state.show_keybindings = false;
            state.show_outline = false;
            state.show_recent = false;
            state.show_search = true;
            state.search_all_tabs = false;
            state.search_index = None;
            state.refresh_search();
            return operation::focus(SEARCH);
        }
        Message::ShowTabSearch => {
            state.show_layout = false;
            state.show_font_picker = false;
            state.show_keybindings = false;
            state.show_outline = false;
            state.show_recent = false;
            state.show_search = true;
            state.search_all_tabs = true;
            state.search_index = None;
            state.refresh_tab_search();
            return operation::focus(SEARCH);
        }
        Message::HideSearch => {
            state.show_search = false;
            return operation::focus(EDITOR);
        }
        Message::SearchAllTabs(all_tabs) => {
            state.search_all_tabs = all_tabs;
            state.search_index = None;
            if all_tabs {
                state.refresh_tab_search();
            } else {
                state.refresh_search();
                if !state.search_query.is_empty() {
                    state.jump_match(0);
                }
            }
        }
        Message::SearchChanged(query) => {
            state.search_query = query;
            state.search_index = None;
            if state.search_all_tabs {
                state.refresh_tab_search();
            } else {
                state.refresh_search();
                if !state.search_query.is_empty() {
                    state.jump_match(0);
                }
            }
        }
        Message::SearchCaseChanged(match_case) => {
            state.search_case_sensitive = match_case;
            state.search_index = None;
            if state.search_all_tabs {
                state.refresh_tab_search();
            } else {
                state.refresh_search();
                state.jump_match(0);
            }
        }
        Message::FindNext => {
            let count = if state.search_all_tabs {
                state.tab_search_hits.len()
            } else {
                state.search_matches().len()
            };
            if count > 0 {
                let index = state.search_index.map_or(0, |index| (index + 1) % count);
                if state.search_all_tabs {
                    state.jump_tab_search(index);
                } else {
                    state.jump_match(index);
                }
            }
        }
        Message::FindPrevious => {
            let count = if state.search_all_tabs {
                state.tab_search_hits.len()
            } else {
                state.search_matches().len()
            };
            if count > 0 {
                let index = state
                    .search_index
                    .map_or(count - 1, |index| (index + count - 1) % count);
                if state.search_all_tabs {
                    state.jump_tab_search(index);
                } else {
                    state.jump_match(index);
                }
            }
        }
        Message::JumpTabSearch(index) => {
            if state.jump_tab_search(index) {
                return operation::focus(EDITOR);
            }
        }
        Message::ShowOutline => {
            state.show_font_picker = false;
            state.show_keybindings = false;
            state.show_search = false;
            state.show_recent = false;
            state.show_outline = true;
            state.outline_query.clear();
            state.outline_index = 0;
            return operation::focus(OUTLINE);
        }
        Message::HideOutline => {
            state.show_outline = false;
            return operation::focus(EDITOR);
        }
        Message::OutlineChanged(query) => {
            state.outline_query = query;
            state.outline_index = 0;
        }
        Message::OutlineNext => {
            let count = state.outline_items().len();
            if count > 0 {
                state.outline_index = (state.outline_index + 1) % count;
            }
        }
        Message::OutlinePrevious => {
            let count = state.outline_items().len();
            if count > 0 {
                state.outline_index = (state.outline_index + count - 1) % count;
            }
        }
        Message::JumpOutlineSelected => {
            return update(state, Message::JumpOutline(state.outline_index));
        }
        Message::JumpOutline(index) => {
            if let Some(heading) = state.outline_items().get(index) {
                let line = heading.line;
                state.current_mut().editor.move_to(text_editor::Cursor {
                    position: text_editor::Position { line, column: 0 },
                    selection: None,
                });
                state.show_outline = false;
                return operation::focus(EDITOR);
            }
        }
        Message::ShowRecent => {
            state.show_font_picker = false;
            state.show_keybindings = false;
            state.show_search = false;
            state.show_outline = false;
            state.show_recent = true;
            state.recent_query.clear();
            state.recent_index = 0;
            return operation::focus(RECENT);
        }
        Message::HideRecent => {
            state.show_recent = false;
            return operation::focus(EDITOR);
        }
        Message::RecentChanged(query) => {
            state.recent_query = query;
            state.recent_index = 0;
        }
        Message::RecentNext => {
            let count = state.recent_items().len();
            if count > 0 {
                state.recent_index = (state.recent_index + 1) % count;
            }
        }
        Message::RecentPrevious => {
            let count = state.recent_items().len();
            if count > 0 {
                state.recent_index = (state.recent_index + count - 1) % count;
            }
        }
        Message::OpenRecentSelected => {
            if let Some(path) = state
                .recent_items()
                .get(state.recent_index)
                .map(|path| (*path).clone())
            {
                return update(state, Message::OpenRecent(path));
            }
        }
        Message::OpenRecent(path) => {
            if state.loading || state.dialog_open {
                return Task::none();
            }
            state.show_recent = false;
            state.loading = true;
            return open_document(path);
        }
        Message::KeyPressed(key, modifiers) => {
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
                            state.notice = state
                                .keybindings
                                .save()
                                .err()
                                .map(|error| format!("Could not save keybindings: {error}"));
                            if state.notice.is_some() {
                                log::warn!("keybindings save failed");
                            }
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
                if state.show_search {
                    return update(state, Message::HideSearch);
                }
                if state.show_outline {
                    return update(state, Message::HideOutline);
                }
                if state.show_recent {
                    return update(state, Message::HideRecent);
                }
                if state.show_keybindings {
                    return update(state, Message::ShowEditor);
                }
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
        }
        _ => unreachable!("non-overlay message routed to overlays"),
    }
    Task::none()
}
