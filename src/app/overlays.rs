use super::{App, EDITOR, Message, OUTLINE, RECENT, SEARCH, open_document, update};
use iced::Task;
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
        Message::ShowSearch | Message::ShowTabSearch => {
            state.zen_mode = false;
            state.current_mut().reading = None;
            state.show_layout = false;
            state.show_settings = false;
            state.show_recent = false;
            state.show_search = true;
            state.search_all_tabs = matches!(message, Message::ShowTabSearch);
            state.search_index = None;
            if state.search_all_tabs {
                state.refresh_tab_search();
            } else {
                state.refresh_search();
            }
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
        Message::FindNext | Message::FindPrevious => {
            let count = if state.search_all_tabs {
                state.tab_search_hits.len()
            } else {
                state.search_matches().len()
            };
            if count > 0 {
                let index = if matches!(message, Message::FindNext) {
                    state.search_index.map_or(0, |index| (index + 1) % count)
                } else {
                    state
                        .search_index
                        .map_or(count - 1, |index| (index + count - 1) % count)
                };
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
                state.current_mut().reading = None;
                state.current_mut().move_editor_to(
                    text_editor::Cursor {
                        position: text_editor::Position { line, column: 0 },
                        selection: None,
                    },
                    focus,
                );
                return operation::focus(EDITOR);
            }
        }
        Message::ShowRecent => {
            state.zen_mode = false;
            state.show_settings = false;
            state.show_search = false;
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
        _ => unreachable!("non-overlay message routed to overlays"),
    }
    Task::none()
}
