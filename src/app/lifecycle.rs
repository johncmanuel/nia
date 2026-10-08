use super::{App, CloseTarget, EDITOR, Message, Tab, open_document};
use crate::{recent, recovery};
use iced::Task;
use iced::widget::{operation, text_editor};
use std::time::Duration;

pub(super) fn handle(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Open => {
            if state.dialog_open || state.loading {
                return Task::none();
            }
            state.dialog_open = true;
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Markdown", &["md", "markdown"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf())
                },
                Message::OpenPicked,
            );
        }
        Message::OpenPicked(path) => {
            state.dialog_open = false;
            if let Some(path) = path {
                state.loading = true;
                return open_document(path);
            }
            if state.confirmed_close.is_some() {
                return state.continue_close();
            }
            return operation::focus(EDITOR);
        }
        Message::Opened(result) => {
            state.loading = false;
            match result {
                Ok(document) => {
                    if let Some(index) = state
                        .tabs
                        .iter()
                        .position(|tab| tab.document.path == document.path)
                    {
                        state.select_tab(index);
                    } else if state.tabs.len() == 1
                        && state.current().document.path.is_none()
                        && !state.current().document.dirty
                        && state.current().document.contents.is_empty()
                    {
                        state.tabs[0] = Tab::new(state.tabs[0].id, document);
                    } else {
                        state.push_group(document);
                    }
                    state.show_settings = false;
                    state.clear_search();
                    state.notice = None;
                    if let Some(path) = state.current().document.path.clone() {
                        recent::remember(&mut state.recent, &path);
                        if recent::save(&state.recent).is_err() {
                            log::warn!("recent documents save failed");
                        }
                    }
                    log::info!("document opened");
                }
                Err(error) => {
                    state.notice = Some(format!("Could not open file: {error}"));
                    log::warn!("document open failed");
                }
            }
            if state.confirmed_close.is_some() {
                return state.continue_close();
            }
            return operation::focus(EDITOR);
        }
        Message::Save => return state.save(),
        Message::SavePicked(path) => {
            state.dialog_open = false;
            if let Some(path) = path {
                return state.start_save_at(state.active, path, false);
            }
            state.confirmed_close = None;
            return operation::focus(EDITOR);
        }
        Message::SaveAs => {
            if state.dialog_open || state.loading || state.current().saving {
                return Task::none();
            }
            state.dialog_open = true;
            let name = state.current().document.file_name();
            return Task::perform(
                async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Markdown", &["md", "markdown"])
                        .set_file_name(&name)
                        .save_file()
                        .await
                        .map(|file| file.path().to_path_buf())
                },
                Message::SaveAsPicked,
            );
        }
        Message::SaveAsPicked(path) => {
            state.dialog_open = false;
            if let Some(path) = path {
                return state.start_save_at(state.active, path, true);
            }
            state.confirmed_close = None;
            return operation::focus(EDITOR);
        }
        Message::Saved(id, path, revision, contents, result) => {
            let Some(index) = state.tabs.iter().position(|tab| tab.id == id) else {
                return Task::none();
            };
            state.tabs[index].saving = false;
            match result {
                Ok(()) => {
                    state.tabs[index].sync_document();
                    state.tabs[index]
                        .document
                        .saved(path.clone(), revision, contents);
                    state.tabs[index].conflict = false;
                    state.tabs[index].last_checked = None;
                    state.tabs[index].recovered_revision = revision;
                    if recovery::remove(&state.tabs[index].recovery_key).is_err() {
                        log::warn!("recovery cleanup failed");
                    }
                    recent::remember(&mut state.recent, &path);
                    if recent::save(&state.recent).is_err() {
                        log::warn!("recent documents save failed");
                    }
                    state.notice = None;
                    log::info!("document saved");
                    if state.confirmed_close.is_some() {
                        return state.continue_close();
                    }
                }
                Err(error) => {
                    state.tabs[index].last_edit = None;
                    state.confirmed_close = None;
                    state.notice = Some(format!("Could not save file: {error}"));
                    log::warn!("document save failed");
                }
            }
        }
        Message::RecoveryWritten(id, revision, result) => {
            if let Some(tab) = state.tabs.iter_mut().find(|tab| tab.id == id) {
                tab.recovering = false;
                tab.recovered_revision = revision;
                if result.is_err() {
                    log::warn!("recovery snapshot failed");
                    state.notice = Some("Could not write a local recovery draft.".into());
                } else if !tab.document.dirty && recovery::remove(&tab.recovery_key).is_err() {
                    log::warn!("recovery cleanup failed");
                }
            }
            if state.confirmed_close.is_some() {
                return state.continue_close();
            }
        }
        Message::ExternalChecked(id, path, checked_revision, result) => {
            let Some(tab) = state.tabs.iter_mut().find(|tab| tab.id == id) else {
                return Task::none();
            };
            tab.checking = false;
            if tab.document.path.as_ref() != Some(&path)
                || tab.document.revision != checked_revision
                || tab.saving
            {
                return Task::none();
            }
            match result {
                Ok(contents) if tab.document.saved_contents.as_deref() != Some(&contents) => {
                    if tab.document.dirty {
                        tab.conflict = true;
                        state.notice =
                            Some("File changed on disk. Use Save As to keep your edits.".into());
                    } else {
                        tab.document.reload(contents);
                        tab.editor = text_editor::Content::with_text(&tab.document.contents);
                        tab.refresh_reading();
                        tab.conflict = false;
                        if state.show_search && state.search_all_tabs {
                            state.refresh_tab_search();
                        } else {
                            state.refresh_search();
                        }
                        state.search_index = None;
                        state.notice = Some("Reloaded changes from disk.".into());
                    }
                    log::info!("external document change detected");
                }
                Ok(_) => {
                    if tab.conflict {
                        state.notice = Some("File is available again.".into());
                    }
                    tab.conflict = false;
                }
                Err(_) => {
                    tab.conflict = true;
                    state.notice =
                        Some("File is unavailable. Use Save As to keep your edits.".into());
                    log::warn!("external document check failed");
                }
            }
        }
        Message::Tick(now) => {
            if state.pending_close.is_some() {
                return Task::none();
            }
            if let Some(tab) = state.tabs.iter_mut().find(|tab| {
                tab.document.dirty
                    && !tab.recovering
                    && tab.recovered_revision != tab.document.revision
                    && tab
                        .last_edit
                        .is_some_and(|last| now.duration_since(last) >= Duration::from_millis(500))
            }) {
                tab.sync_document();
                if tab.document.dirty {
                    tab.recovering = true;
                    let id = tab.id;
                    let revision = tab.document.revision;
                    let key = tab.recovery_key.clone();
                    let contents = tab.document.contents.clone();
                    return Task::perform(
                        async move {
                            recovery::write(&key, &contents).map_err(|error| error.to_string())
                        },
                        move |result| Message::RecoveryWritten(id, revision, result),
                    );
                }
                tab.recovered_revision = tab.document.revision;
                if recovery::remove(&tab.recovery_key).is_err() {
                    log::warn!("recovery cleanup failed");
                }
            }
            if let Some(index) = state.tabs.iter().position(|tab| {
                tab.document.dirty
                    && !tab.saving
                    && !tab.conflict
                    && tab.document.path.is_some()
                    && tab
                        .last_edit
                        .is_some_and(|last| now.duration_since(last) >= Duration::from_millis(800))
            }) && let Some(path) = state.tabs[index].document.path.clone()
            {
                return state.start_save_at(index, path, false);
            }
            if let Some(tab) = state.tabs.iter_mut().find(|tab| {
                tab.document.path.is_some()
                    && !tab.saving
                    && !tab.checking
                    && tab
                        .last_checked
                        .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(3))
            }) {
                tab.checking = true;
                tab.last_checked = Some(now);
                let id = tab.id;
                let revision = tab.document.revision;
                if let Some(path) = tab.document.path.clone() {
                    let checked_path = path.clone();
                    return Task::perform(
                        async move { std::fs::read_to_string(path).map_err(|error| error.to_string()) },
                        move |result| Message::ExternalChecked(id, checked_path, revision, result),
                    );
                }
            }
        }
        Message::CloseRequested(id) => {
            if state.pending_close.is_some() || state.confirmed_close.is_some() {
                return Task::none();
            }
            for tab in &mut state.tabs {
                tab.sync_document();
            }
            if state.tabs.iter().any(|tab| tab.document.dirty) {
                state.pending_close = Some(CloseTarget::Window(id));
                return Task::none();
            }
            state.confirmed_close = Some(CloseTarget::Window(id));
            return state.continue_close();
        }
        Message::SaveAndClose => {
            if let Some(target) = state.pending_close.take() {
                state.notice = None;
                state.confirmed_close = Some(target);
                return state.continue_close();
            }
        }
        Message::DiscardAndClose => return state.discard_close(),
        Message::CancelClose => {
            state.pending_close = None;
            state.notice = None;
            return operation::focus(EDITOR);
        }
        Message::FileDropped(path) => {
            if state.dialog_open || state.loading {
                return Task::none();
            }
            state.loading = true;
            return open_document(path);
        }
        _ => unreachable!("non-file message routed to lifecycle"),
    }
    Task::none()
}
