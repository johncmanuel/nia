use super::{
    App, CloseTarget, EDITOR, Message, Tab, formatting, image_blocks, keyboard, lifecycle,
    overlays, preferences,
};
use crate::document::Document;
use iced::Task;
use iced::widget::{operation, pane_grid, text_editor};
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
            if index != state.active {
                state.current_mut().history.break_group();
                state.tabs[index].history.break_group();
            }
            state.active = index;
            if let Some((&pane, _)) = state.panes.iter().find(|(_, tab_id)| **tab_id == id) {
                state.active_pane = pane;
            }
            let tab = state.current_mut();
            let before = tab.editor.cursor();
            let pending = tab.history.begin(&tab.editor, &action);
            let had_image_blocks = !tab.image_blocks.is_empty();
            let auto = formatting::auto_edit(&tab.editor, &action);
            let skipped_image_close = matches!(auto, Some(formatting::AutoEdit::SkipCloser))
                && matches!(
                    action,
                    text_editor::Action::Edit(text_editor::Edit::Insert(')'))
                );
            let mut actions = Vec::new();
            let changed = if let Some(auto) = auto {
                auto.perform(|action| {
                    tab.editor.perform(action.clone());
                    actions.push(action);
                })
            } else {
                tab.editor.perform(action.clone());
                actions.push(action.clone());
                action.is_edit()
            };
            let image_structure = image_blocks::changes_image_structure(&action);
            let pasted_image = matches!(&action, text_editor::Action::Edit(text_editor::Edit::Paste(text)) if text.contains("!["));
            if skipped_image_close {
                tab.rebuild_image_blocks();
            }
            if changed {
                tab.history.finish(&tab.editor, pending);
                let line = tab.editor.cursor().position.line;
                if image_structure
                    && (pasted_image
                        || [line, line.saturating_sub(1)].into_iter().any(|line| {
                            tab.editor
                                .line(line)
                                .is_some_and(|line| line.text.contains("!["))
                        }))
                {
                    tab.rebuild_image_blocks();
                }
                tab.refresh_reading();
                tab.document.mark_edited();
                tab.last_edit = Some(Instant::now());
                state.mirror_edit(index, before, &actions);
                if state.show_search && state.search_all_tabs {
                    state.refresh_tab_search();
                } else {
                    state.refresh_search();
                }
                state.search_index = None;
                state.notice = None;
                state.show_format_menu = false;
            }
            if had_image_blocks != !state.current().image_blocks.is_empty() {
                return operation::focus(EDITOR);
            }
        }
        Message::EditImageBlock(id, block, action) => {
            if state.loading || state.dialog_open {
                return Task::none();
            }
            let Some(index) = state.tabs.iter().position(|tab| tab.id == id) else {
                return Task::none();
            };
            if index != state.active {
                state.current_mut().history.break_group();
                state.tabs[index].history.break_group();
            }
            state.active = index;
            if let Some((&pane, _)) = state.panes.iter().find(|(_, tab_id)| **tab_id == id) {
                state.active_pane = pane;
            }
            state.current_mut().activate_image_block(block);
            let tab = state.current_mut();
            let before = tab.editor.cursor();
            let pending = tab.history.begin(&tab.editor, &action);
            let auto = state
                .current()
                .image_blocks
                .get(block)
                .and_then(|image| formatting::auto_edit(&image.content, &action));
            let mut actions = Vec::new();
            let changed = if let Some(auto) = auto {
                auto.perform(|action| {
                    state
                        .current_mut()
                        .perform_image_action(block, action.clone());
                    actions.push(action);
                })
            } else {
                let changed = state
                    .current_mut()
                    .perform_image_action(block, action.clone());
                actions.push(action.clone());
                changed
            };
            let moved_block = state.current().active_image_block != block;
            let removed_image_blocks = state.current().image_blocks.is_empty();
            if changed {
                let tab = state.current_mut();
                tab.history.finish(&tab.editor, pending);
                tab.refresh_reading();
                tab.document.mark_edited();
                tab.last_edit = Some(Instant::now());
                state.mirror_edit(index, before, &actions);
                if state.show_search && state.search_all_tabs {
                    state.refresh_tab_search();
                } else {
                    state.refresh_search();
                }
                state.search_index = None;
                state.notice = None;
                state.show_format_menu = false;
            }
            if moved_block
                || removed_image_blocks
                || matches!(action, text_editor::Action::Click(_))
            {
                return operation::focus(EDITOR);
            }
        }
        Message::ActivateImageBlock(id, block) => {
            let Some(index) = state.tabs.iter().position(|tab| tab.id == id) else {
                return Task::none();
            };
            state.active = index;
            if let Some((&pane, _)) = state.panes.iter().find(|(_, tab_id)| **tab_id == id) {
                state.active_pane = pane;
            }
            state.current_mut().history.break_group();
            state.current_mut().activate_image_block(block);
            return operation::focus(EDITOR);
        }
        Message::ToggleFormatMenu => {
            state.show_format_menu = !state.show_format_menu;
        }
        Message::Format(format) => return formatting::insert(state, format),
        Message::Undo | Message::Redo => return restore_edit(state, message),
        Message::New => {
            if state.dialog_open {
                return Task::none();
            }
            state.push_group(Document::new());
            log::info!("new document tab group");
            return operation::focus(EDITOR);
        }
        Message::NewGroup => {
            if state.dialog_open {
                return Task::none();
            }
            state.push_group(Document::new());
            log::info!("new tab group");
            return operation::focus(EDITOR);
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
            state.show_settings = false;
            state.clear_search();
            state.notice = None;
            return operation::focus(EDITOR);
        }
        Message::SelectGroup(index) => {
            if state.dialog_open || index >= state.groups.len() {
                return Task::none();
            }
            state.select_group(index);
            state.show_settings = false;
            state.clear_search();
            return operation::focus(EDITOR);
        }
        Message::CloseCurrentTab => {
            let active = state.active;
            return update(state, Message::CloseTab(active));
        }
        Message::CloseCurrentGroup => {
            if state.loading
                || state.dialog_open
                || state.pending_close.is_some()
                || state.confirmed_close.is_some()
            {
                return Task::none();
            }
            let ids = state.groups[state.active_group].tabs.clone();
            for tab in &mut state.tabs {
                if ids.contains(&tab.id) {
                    tab.sync_document();
                }
            }
            let Some(&anchor) = ids.first() else {
                return Task::none();
            };
            let target = CloseTarget::Group(anchor);
            if state.tabs.iter().enumerate().any(|(index, tab)| {
                ids.contains(&tab.id) && tab.document.dirty && !state.view_survives(index, &ids)
            }) {
                log::debug!("group close requires unsaved-change confirmation");
                state.pending_close = Some(target);
                return Task::none();
            }
            state.confirmed_close = Some(target);
            return state.continue_close();
        }
        message @ (Message::NextPane | Message::PreviousPane) => {
            if let Some(pane) = state.neighbor_pane(matches!(message, Message::NextPane)) {
                return update(state, Message::ActivatePane(pane));
            }
        }
        Message::ActivatePane(pane) => {
            if let Some(&id) = state.panes.get(pane)
                && let Some(index) = state.tabs.iter().position(|tab| tab.id == id)
            {
                if pane != state.active_pane {
                    state.current_mut().history.break_group();
                    state.tabs[index].history.break_group();
                    state.clear_search();
                }
                state.active_pane = pane;
                state.active = index;
                return operation::focus(EDITOR);
            }
        }
        Message::SinglePane => {
            let (panes, pane) = pane_grid::State::new(state.current().id);
            state.panes = panes;
            state.active_pane = pane;
        }
        Message::SplitPane(axis) => {
            let id = state.next_id();
            if let Some((pane, _)) = state.panes.split(axis, state.active_pane, id) {
                state.tabs.push(Tab::new(id, Document::new()));
                state.groups[state.active_group].tabs.push(id);
                state.active = state.tabs.len() - 1;
                state.active_pane = pane;
                state.clear_search();
                log::info!("editor pane split");
                return operation::focus(EDITOR);
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
            if state.tabs[index].document.dirty
                && !state.view_survives(index, &[state.tabs[index].id])
            {
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
        Message::KeyPressed(key, modifiers) => return keyboard::handle(state, key, modifiers),
        message @ (Message::ToggleZen
        | Message::SetHighlightLine(_)
        | Message::SetDimming(_)
        | Message::RecordShortcut(_)
        | Message::ResetKeybindings
        | Message::SetTheme(_)
        | Message::SetTransparentBackground(_)
        | Message::SetBackgroundOpacity(_)
        | Message::ToggleThemePicker
        | Message::HideThemePicker
        | Message::SetFont(_)
        | Message::ResetFont
        | Message::IncreaseFontSize
        | Message::DecreaseFontSize) => return preferences::handle(state, message),
        message @ (Message::ShowSettings
        | Message::ShowHandbook
        | Message::HideHandbook
        | Message::ShowEditor
        | Message::ToggleReading
        | Message::CopyLink(_)
        | Message::ShowSearch
        | Message::ShowTabSearch
        | Message::HideSearch
        | Message::SearchAllTabs(_)
        | Message::SearchChanged(_)
        | Message::SearchCaseChanged(_)
        | Message::FindNext
        | Message::FindPrevious
        | Message::JumpTabSearch(_)
        | Message::ShowOutline
        | Message::HideOutline
        | Message::OutlineChanged(_)
        | Message::OutlineNext
        | Message::OutlinePrevious
        | Message::JumpOutline(_)
        | Message::JumpOutlineSelected
        | Message::ShowRecent
        | Message::HideRecent
        | Message::RecentChanged(_)
        | Message::RecentNext
        | Message::RecentPrevious
        | Message::OpenRecentSelected
        | Message::OpenRecent(_)) => return overlays::handle(state, message),
    }
    Task::none()
}

fn restore_edit(state: &mut App, message: Message) -> Task<Message> {
    if state.loading
        || state.dialog_open
        || state.pending_close.is_some()
        || state.show_settings
        || state.show_search
        || state.show_recent
        || state.handbook.is_some()
    {
        return Task::none();
    }
    let tab = state.current_mut();
    let restored = match message {
        Message::Undo => tab.history.undo(&mut tab.editor),
        Message::Redo => tab.history.redo(&mut tab.editor),
        _ => return Task::none(),
    };
    if !restored {
        return Task::none();
    }
    tab.rebuild_image_blocks();
    tab.refresh_reading();
    tab.document.mark_edited();
    tab.sync_document();
    tab.last_edit = tab.document.dirty.then(Instant::now);
    let index = state.active;
    state.sync_shared_views(index, true);
    if !state.tabs[index].document.dirty {
        let primary = state.primary_view(index);
        state.tabs[primary].recovered_revision = state.tabs[primary].document.revision;
        if let Err(error) = crate::recovery::remove(&state.tabs[primary].recovery_key) {
            log::warn!("recovery cleanup after undo failed: {:?}", error.kind());
        }
    }
    if state.show_search && state.search_all_tabs {
        state.refresh_tab_search();
    } else {
        state.refresh_search();
    }
    state.search_index = None;
    state.notice = None;
    state.show_format_menu = false;
    operation::focus(EDITOR)
}
