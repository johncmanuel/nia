use super::{App, EDITOR, Message, OUTLINE, RECENT, SEARCH, open_document, update};
use crate::keybindings::{Keybindings, Shortcut};
use iced::Task;
use iced::keyboard::{Key, key::Named};
use iced::theme::Base;
use iced::widget::{operation, text_editor};

pub(super) fn handle(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::ShowSettings => {
            state.show_layout = false;
            state.show_settings = true;
            state.show_theme_picker = false;
            state.recording = None;
            state.clear_search();
            state.notice = None;
        }
        Message::ShowEditor => {
            state.show_settings = false;
            state.show_theme_picker = false;
            state.current_mut().reading = None;
            state.recording = None;
            state.notice = None;
            return operation::focus(EDITOR).chain(Task::done(Message::AlignTypewriter));
        }
        Message::ToggleZen => {
            state.zen_mode = !state.zen_mode;
            if state.zen_mode && !state.show_settings {
                state.show_layout = false;
                state.clear_search();
                state.current_mut().reading = None;
            }
            if state.show_settings {
                return Task::none();
            }
            return operation::focus(EDITOR).chain(Task::done(Message::AlignTypewriter));
        }
        Message::ToggleTypewriter => {
            state.focus.typewriter = !state.focus.typewriter;
            for tab in &mut state.tabs {
                tab.typewriter_aligned = false;
            }
            save_focus(state);
            if state.focus.typewriter && !state.show_settings {
                return operation::focus(EDITOR).chain(Task::done(Message::AlignTypewriter));
            }
        }
        Message::AlignTypewriter => {
            if state.focus.typewriter && !state.show_settings {
                let above = state.focus.lines_above;
                state.current_mut().align_typewriter(above);
            }
        }
        Message::SetHighlightLine(enabled) => {
            state.focus.highlight_line = enabled;
            save_focus(state);
        }
        Message::SetLinesAbove(count) => {
            let count = count.min(crate::focus::MAX_CONTEXT);
            let difference = i32::from(state.focus.lines_above) - i32::from(count);
            state.focus.lines_above = count;
            if state.focus.typewriter && difference != 0 {
                for tab in &mut state.tabs {
                    if tab.typewriter_aligned {
                        tab.editor
                            .perform(text_editor::Action::Scroll { lines: difference });
                    }
                }
            }
            save_focus(state);
        }
        Message::SetLinesBelow(count) => {
            state.focus.lines_below = count.min(crate::focus::MAX_CONTEXT);
            save_focus(state);
        }
        Message::SetDimming(dimming) => {
            state.focus.dimming = dimming;
            save_focus(state);
        }
        Message::ToggleReading => {
            state.zen_mode = false;
            state.show_settings = false;
            if state.current().reading.is_some() {
                state.current_mut().reading = None;
                return operation::focus(EDITOR);
            }
            state.show_layout = false;
            state.clear_search();
            state.current_mut().sync_document();
            let content =
                iced::widget::markdown::Content::parse(&state.current().document.contents);
            state.current_mut().reading = Some(content);
        }
        Message::CopyLink(url) => {
            state.notice = Some("Link copied".into());
            return iced::clipboard::write(url);
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
        Message::ShowSearch => {
            state.zen_mode = false;
            state.current_mut().reading = None;
            state.show_layout = false;
            state.show_settings = false;
            state.show_outline = false;
            state.show_recent = false;
            state.show_search = true;
            state.search_all_tabs = false;
            state.search_index = None;
            state.refresh_search();
            return operation::focus(SEARCH);
        }
        Message::ShowTabSearch => {
            state.zen_mode = false;
            state.current_mut().reading = None;
            state.show_layout = false;
            state.show_settings = false;
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
            state.zen_mode = false;
            state.current_mut().reading = None;
            state.show_settings = false;
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
                let focus = state.focus;
                state.current_mut().move_editor_to(
                    text_editor::Cursor {
                        position: text_editor::Position { line, column: 0 },
                        selection: None,
                    },
                    focus,
                );
                state.show_outline = false;
                return operation::focus(EDITOR);
            }
        }
        Message::ShowRecent => {
            state.zen_mode = false;
            state.show_settings = false;
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
                if state.show_theme_picker {
                    return update(state, Message::HideThemePicker);
                }
                if state.show_search {
                    return update(state, Message::HideSearch);
                }
                if state.show_outline {
                    return update(state, Message::HideOutline);
                }
                if state.show_recent {
                    return update(state, Message::HideRecent);
                }
                if state.show_settings {
                    return update(state, Message::ShowEditor);
                }
                if state.zen_mode {
                    return update(state, Message::ToggleZen);
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
