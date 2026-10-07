use crate::{document::Document, focus, recovery};
use iced::widget::{markdown, text_editor};
use std::time::Instant;

pub(crate) struct Tab {
    pub(crate) id: u64,
    pub(crate) document: Document,
    pub(crate) editor: text_editor::Content,
    pub(super) typewriter_aligned: bool,
    pub(crate) reading: Option<markdown::Content>,
    pub(crate) saving: bool,
    pub(crate) checking: bool,
    pub(crate) conflict: bool,
    pub(crate) recovering: bool,
    pub(crate) recovered_revision: u64,
    pub(crate) recovery_key: String,
    pub(super) last_edit: Option<Instant>,
    pub(super) last_checked: Option<Instant>,
}

impl Tab {
    pub(super) fn new(id: u64, document: Document) -> Self {
        let editor = text_editor::Content::with_text(&document.contents);
        Self {
            id,
            document,
            editor,
            typewriter_aligned: false,
            reading: None,
            saving: false,
            checking: false,
            conflict: false,
            recovering: false,
            recovered_revision: 0,
            recovery_key: recovery::new_key(),
            last_edit: None,
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

    pub(super) fn align_typewriter(&mut self, lines_above: u8) {
        if self.typewriter_aligned {
            return;
        }
        let line = self.editor.cursor().position.line;
        let scroll = line.saturating_sub(usize::from(lines_above));
        if scroll > 0 {
            self.editor.perform(text_editor::Action::Scroll {
                lines: scroll.min(i32::MAX as usize) as i32,
            });
        }
        self.typewriter_aligned = true;
    }

    fn follow_typewriter(&mut self, old_line: usize, focus: focus::Settings) {
        if !focus.typewriter {
            return;
        }
        if !self.typewriter_aligned {
            self.align_typewriter(focus.lines_above);
            return;
        }
        let new_line = self.editor.cursor().position.line;
        let distance = new_line.abs_diff(old_line).min(i32::MAX as usize) as i32;
        let lines = if new_line >= old_line {
            distance
        } else {
            -distance
        };
        if lines != 0 {
            self.editor.perform(text_editor::Action::Scroll { lines });
        }
    }

    pub(super) fn perform_editor_action(
        &mut self,
        action: text_editor::Action,
        focus: focus::Settings,
    ) {
        if focus.typewriter && matches!(action, text_editor::Action::Scroll { .. }) {
            return;
        }
        let old_line = self.editor.cursor().position.line;
        self.editor.perform(action);
        self.follow_typewriter(old_line, focus);
    }

    pub(super) fn move_editor_to(&mut self, cursor: text_editor::Cursor, focus: focus::Settings) {
        let old_line = self.editor.cursor().position.line;
        self.editor.move_to(cursor);
        self.follow_typewriter(old_line, focus);
    }
}
