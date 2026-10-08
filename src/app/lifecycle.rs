use super::{App, CloseTarget, EDITOR, Message, Tab, open_document};
use crate::{recent, recovery};
use iced::Task;
use iced::widget::{operation, text_editor};
use std::io::ErrorKind;
use std::time::{Duration, Instant};

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
                        if let Err(error) = recent::save(&state.recent) {
                            log::warn!("recent documents save failed: {:?}", error.kind());
                        }
                    }
                    log::info!("document opened");
                }
                Err(error) => {
                    state.notice = Some(error.open_notice());
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
                    let mut cleanup_notice = None;
                    state.tabs[index].sync_document();
                    state.tabs[index]
                        .document
                        .saved(path.clone(), revision, contents);
                    state.tabs[index].conflict = false;
                    state.tabs[index].last_checked = None;
                    state.tabs[index].last_save_failure = None;
                    state.tabs[index].recovered_revision = revision;
                    state.tabs[index].last_recovery_failure = None;
                    if !state.tabs[index].document.dirty
                        && let Err(error) = recovery::remove(&state.tabs[index].recovery_key)
                    {
                        log::warn!("recovery cleanup failed: {:?}", error.kind());
                        cleanup_notice = Some(
                            "Saved the document, but could not remove its recovery draft. An old draft may reopen at the next start."
                                .into(),
                        );
                    }
                    recent::remember(&mut state.recent, &path);
                    if let Err(error) = recent::save(&state.recent) {
                        log::warn!("recent documents save failed: {:?}", error.kind());
                    }
                    state.notice = cleanup_notice;
                    log::info!("document saved");
                    if state.confirmed_close.is_some() {
                        return state.continue_close();
                    }
                }
                Err(error) => {
                    let now = Instant::now();
                    state.tabs[index].last_edit = Some(now);
                    state.tabs[index].last_save_failure = Some(now);
                    state.confirmed_close = None;
                    if state.tabs[index].document.path.as_ref() == Some(&path)
                        && matches!(error.kind, ErrorKind::AlreadyExists | ErrorKind::NotFound)
                    {
                        state.tabs[index].conflict = true;
                    }
                    state.notice = Some(error.save_notice());
                }
            }
        }
        Message::RecoveryWritten(id, revision, result) => {
            if let Some(tab) = state.tabs.iter_mut().find(|tab| tab.id == id) {
                tab.recovering = false;
                match result {
                    Ok(()) => {
                        tab.recovered_revision = revision;
                        tab.last_recovery_failure = None;
                        if !tab.document.dirty
                            && let Err(error) = recovery::remove(&tab.recovery_key)
                        {
                            log::warn!("recovery cleanup failed: {:?}", error.kind());
                            state.notice = Some(
                                "Could not remove an old recovery draft. It may reopen at the next start."
                                    .into(),
                            );
                        }
                    }
                    Err(error) => {
                        tab.last_recovery_failure = Some(Instant::now());
                        state.notice = Some(error.recovery_notice().into());
                    }
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
                        tab.history.clear();
                        tab.rebuild_image_blocks();
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
                Err(error) => {
                    if !tab.conflict {
                        log::warn!("external document check failed");
                    }
                    tab.conflict = true;
                    state.notice = Some(error.external_notice().into());
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
                    && tab.last_recovery_failure.is_none_or(|failure| {
                        now.saturating_duration_since(failure) >= Duration::from_secs(5)
                    })
                    && tab
                        .last_edit
                        .is_some_and(|last| now.duration_since(last) >= Duration::from_millis(500))
            }) {
                tab.sync_document();
                if tab.document.dirty {
                    tab.recovering = true;
                    log::debug!("recovery snapshot queued");
                    let id = tab.id;
                    let revision = tab.document.revision;
                    let key = tab.recovery_key.clone();
                    let contents = tab.document.contents.clone();
                    return Task::perform(
                        async move {
                            recovery::write(&key, &contents).map_err(|error| {
                                log::warn!("recovery snapshot failed: {:?}", error.kind());
                                error.into()
                            })
                        },
                        move |result| Message::RecoveryWritten(id, revision, result),
                    );
                }
                tab.recovered_revision = tab.document.revision;
                if let Err(error) = recovery::remove(&tab.recovery_key) {
                    log::warn!("recovery cleanup failed: {:?}", error.kind());
                }
            }
            if let Some(index) = state.tabs.iter().position(|tab| {
                tab.document.dirty
                    && !tab.saving
                    && !tab.conflict
                    && tab.last_save_failure.is_none_or(|failure| {
                        now.saturating_duration_since(failure) >= Duration::from_secs(5)
                    })
                    && tab.document.path.is_some()
                    && tab
                        .last_edit
                        .is_some_and(|last| now.duration_since(last) >= Duration::from_millis(800))
            }) && let Some(path) = state.tabs[index].document.path.clone()
            {
                log::debug!("autosave queued");
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
                        async move {
                            std::fs::read_to_string(path).map_err(|error| {
                                log::debug!("external document check failed: {:?}", error.kind());
                                error.into()
                            })
                        },
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
