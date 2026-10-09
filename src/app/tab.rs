use super::history::History;
use crate::{document::Document, recovery};
use iced::widget::{markdown, text_editor};
use std::time::Instant;

pub(crate) struct Tab {
    pub(crate) id: u64,
    pub(crate) document: Document,
    pub(crate) editor: text_editor::Content,
    pub(super) history: History,
    pub(crate) reading: Option<markdown::Content>,
    pub(crate) saving: bool,
    pub(crate) checking: bool,
    pub(crate) conflict: bool,
    pub(crate) recovering: bool,
    pub(crate) recovered_revision: u64,
    pub(super) last_recovery_failure: Option<Instant>,
    pub(crate) recovery_key: String,
    pub(super) last_edit: Option<Instant>,
    pub(super) last_save_failure: Option<Instant>,
    pub(super) last_checked: Option<Instant>,
}

impl Tab {
    pub(super) fn new(id: u64, document: Document) -> Self {
        let editor = text_editor::Content::with_text(&document.contents);
        Self {
            id,
            document,
            editor,
            history: History::default(),
            reading: None,
            saving: false,
            checking: false,
            conflict: false,
            recovering: false,
            recovered_revision: 0,
            last_recovery_failure: None,
            recovery_key: recovery::new_key(),
            last_edit: None,
            last_save_failure: None,
            last_checked: None,
        }
    }

    pub(super) fn recovered(id: u64, draft: recovery::Draft) -> Self {
        let mut document = Document::new();
        document.edit(draft.contents);
        document.dirty = true;
        let mut tab = Self::new(id, document);
        tab.recovery_key = draft.key;
        tab.recovered_revision = tab.document.revision;
        tab
    }

    pub(super) fn sync_document(&mut self) {
        self.document.sync_contents(self.editor.text());
    }

    pub(super) fn refresh_reading(&mut self) {
        if self.reading.is_some() {
            self.reading = Some(markdown::Content::parse(&self.editor.text()));
        }
    }
}
