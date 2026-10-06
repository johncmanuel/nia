use super::{App, CloseTarget, EDITOR, Message};
use crate::document::save_file;
use crate::recovery;
use iced::widget::operation;
use iced::{Task, window};
use std::path::PathBuf;

impl App {
    pub(super) fn save(&mut self) -> Task<Message> {
        if self.current().saving || self.loading || self.dialog_open {
            return Task::none();
        }
        if self.current().conflict {
            self.confirmed_close = None;
            self.notice = Some("File changed on disk. Use Save As to keep your edits.".into());
            return Task::none();
        }
        if let Some(path) = self.current().document.path.clone() {
            self.start_save_at(self.active, path, false)
        } else {
            self.dialog_open = true;
            Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Markdown", &["md", "markdown"])
                        .set_file_name("Untitled.md")
                        .save_file()
                        .await
                        .map(|file| file.path().to_path_buf())
                },
                Message::SavePicked,
            )
        }
    }

    pub(super) fn start_save_at(
        &mut self,
        index: usize,
        path: PathBuf,
        as_new: bool,
    ) -> Task<Message> {
        let tab = &mut self.tabs[index];
        tab.sync_document();
        tab.saving = true;
        tab.last_edit = None;
        let id = tab.id;
        let revision = tab.document.revision;
        let contents = tab.document.contents.clone();
        let expected = if as_new {
            None
        } else {
            tab.document.saved_contents.clone()
        };
        let save_path = path.clone();
        Task::perform(
            async move {
                save_file(&save_path, &contents, expected.as_deref())
                    .map_err(|error| error.to_string())
                    .map(|()| contents)
            },
            move |result| match result {
                Ok(contents) => Message::Saved(id, path, revision, contents, Ok(())),
                Err(error) => Message::Saved(id, path, revision, String::new(), Err(error)),
            },
        )
    }

    pub(super) fn continue_close(&mut self) -> Task<Message> {
        let Some(target) = self.confirmed_close else {
            return Task::none();
        };
        if self.loading
            || self.dialog_open
            || self.tabs.iter().any(|tab| tab.saving || tab.recovering)
        {
            return Task::none();
        }
        for tab in &mut self.tabs {
            if tab.document.dirty {
                tab.sync_document();
            }
        }
        let dirty = match target {
            CloseTarget::Tab(id) => self
                .tabs
                .iter()
                .position(|tab| tab.id == id && tab.document.dirty),
            CloseTarget::Window(_) => self.tabs.iter().position(|tab| tab.document.dirty),
        };
        if let Some(index) = dirty {
            self.active = index;
            return self.save();
        }
        self.confirmed_close = None;
        match target {
            CloseTarget::Tab(id) => {
                if let Some(index) = self.tabs.iter().position(|tab| tab.id == id) {
                    self.remove_tab(index);
                    return operation::focus(EDITOR);
                }
                Task::none()
            }
            CloseTarget::Window(id) => window::close(id),
        }
    }

    pub(super) fn discard_close(&mut self) -> Task<Message> {
        let Some(target) = self.pending_close else {
            return Task::none();
        };
        if self.loading || self.dialog_open {
            self.notice =
                Some("Wait for the open dialog to finish, then choose Discard again.".into());
            return Task::none();
        }
        let indexes: Vec<_> = match target {
            CloseTarget::Tab(id) => self
                .tabs
                .iter()
                .position(|tab| tab.id == id)
                .into_iter()
                .collect(),
            CloseTarget::Window(_) => (0..self.tabs.len()).collect(),
        };
        if indexes
            .iter()
            .any(|&index| self.tabs[index].saving || self.tabs[index].recovering)
        {
            self.notice = Some("Wait for saving to finish, then choose Discard again.".into());
            return Task::none();
        }
        for &index in &indexes {
            self.tabs[index].sync_document();
            if self.tabs[index].document.dirty
                && let Err(error) = recovery::remove(&self.tabs[index].recovery_key)
            {
                self.notice = Some(format!("Could not remove recovery draft: {error}"));
                log::warn!("recovery cleanup failed");
                return Task::none();
            }
        }
        self.pending_close = None;
        self.notice = None;
        match target {
            CloseTarget::Tab(id) => {
                if let Some(index) = self.tabs.iter().position(|tab| tab.id == id) {
                    self.remove_tab(index);
                    return operation::focus(EDITOR);
                }
                Task::none()
            }
            CloseTarget::Window(id) => window::close(id),
        }
    }
}
