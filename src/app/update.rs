use super::{App, CloseTarget, EDITOR, Message, Tab, lifecycle, overlays};
use crate::document::Document;
use iced::Task;
use iced::widget::{operation, pane_grid};
use std::time::Instant;

pub(crate) fn update(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Edit(id, action) => {
            if state.loading || state.dialog_open {
                return Task::none();
            }
            let Some(index) = state.tabs.iter().position(|tab| tab.id == id) else {
                return Task::none();
            };
            state.active = index;
            if let Some((&pane, _)) = state.panes.iter().find(|(_, tab_id)| **tab_id == id) {
                state.active_pane = pane;
            }
            let focus = state.focus;
            let tab = state.current_mut();
            let changed = action.is_edit();
            tab.perform_editor_action(action, focus);
            if changed {
                tab.refresh_reading();
                tab.document.mark_edited();
                tab.last_edit = Some(Instant::now());
                if state.show_search && state.search_all_tabs {
                    state.refresh_tab_search();
                } else {
                    state.refresh_search();
                }
                state.search_index = None;
                state.notice = None;
            }
        }
        Message::New => {
            if state.dialog_open {
                return Task::none();
            }
            state.show_layout = false;
            state.push_tab(Document::new());
            log::info!("new document tab");
            return operation::focus(EDITOR).chain(Task::done(Message::AlignTypewriter));
        }
        message @ (Message::Open
        | Message::OpenPicked(_)
        | Message::Opened(_)
        | Message::Save
        | Message::SavePicked(_)
        | Message::SaveAs
        | Message::SaveAsPicked(_)
        | Message::Saved(..)
        | Message::RecoveryWritten(..)
        | Message::ExternalChecked(..)
        | Message::Tick(_)
        | Message::CloseRequested(_)
        | Message::SaveAndClose
        | Message::DiscardAndClose
        | Message::CancelClose
        | Message::FileDropped(_)) => return lifecycle::handle(state, message),
        Message::SelectTab(index) => {
            if state.dialog_open || index >= state.tabs.len() {
                return Task::none();
            }
            state.select_tab(index);
            state.show_layout = false;
            state.show_settings = false;
            state.clear_search();
            state.notice = None;
            return operation::focus(EDITOR).chain(Task::done(Message::AlignTypewriter));
        }
        Message::CloseCurrentTab => {
            let active = state.active;
            return update(state, Message::CloseTab(active));
        }
        Message::NextTab => {
            let next = (state.active + 1) % state.tabs.len();
            return update(state, Message::SelectTab(next));
        }
        Message::PreviousTab => {
            let previous = (state.active + state.tabs.len() - 1) % state.tabs.len();
            return update(state, Message::SelectTab(previous));
        }
        Message::ActivatePane(pane) => {
            if let Some(&id) = state.panes.get(pane)
                && let Some(index) = state.tabs.iter().position(|tab| tab.id == id)
            {
                if pane != state.active_pane {
                    state.clear_search();
                }
                state.active_pane = pane;
                state.active = index;
            }
        }
        Message::SinglePane => {
            state.show_layout = false;
            let (panes, pane) = pane_grid::State::new(state.current().id);
            state.panes = panes;
            state.active_pane = pane;
        }
        Message::SplitPane(axis) => {
            state.show_layout = false;
            let existing = state
                .tabs
                .iter()
                .find(|tab| {
                    !state
                        .panes
                        .iter()
                        .any(|(_, visible_id)| *visible_id == tab.id)
                })
                .map(|tab| tab.id);
            let id = existing.unwrap_or_else(|| state.next_id());
            if let Some((pane, _)) = state.panes.split(axis, state.active_pane, id) {
                if existing.is_none() {
                    state.tabs.push(Tab::new(id, Document::new()));
                }
                if let Some(index) = state.tabs.iter().position(|tab| tab.id == id) {
                    state.active = index;
                }
                state.active_pane = pane;
                state.clear_search();
                log::info!("editor pane split");
                return operation::focus(EDITOR).chain(Task::done(Message::AlignTypewriter));
            }
        }
        Message::ClosePane(pane) => {
            if state.panes.len() > 1
                && let Some((_, sibling)) = state.panes.close(pane)
                && state.active_pane == pane
            {
                state.active_pane = sibling;
                state.activate_visible_tab();
                state.clear_search();
            }
        }
        Message::ToggleLayout => {
            state.show_layout = !state.show_layout;
            if state.show_layout {
                state.show_search = false;
            }
        }
        Message::PaneDragged(pane_grid::DragEvent::Dropped { pane, target }) => {
            state.panes.drop(pane, target);
        }
        Message::PaneDragged(_) => {}
        Message::PaneResized(event) => {
            state.panes.resize(event.split, event.ratio);
        }
        Message::CloseTab(index) => {
            if state.dialog_open || index >= state.tabs.len() {
                return Task::none();
            }
            state.tabs[index].sync_document();
            if state.tabs[index].document.dirty {
                state.select_tab(index);
                state.pending_close = Some(CloseTarget::Tab(state.tabs[index].id));
                return Task::none();
            }
            if state.tabs[index].saving || state.tabs[index].recovering {
                state.notice =
                    Some("Wait for the document to finish saving before closing.".into());
                return Task::none();
            }
            state.remove_tab(index);
            return operation::focus(EDITOR);
        }
        message => return overlays::handle(state, message),
    }
    Task::none()
}
