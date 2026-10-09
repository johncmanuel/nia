use super::{App, Tab};
use iced::widget::{pane_grid, text_editor};

impl App {
    pub(super) fn primary_view(&self, index: usize) -> usize {
        let Some(path) = self.tabs[index].document.path.as_ref() else {
            return index;
        };
        self.tabs
            .iter()
            .position(|tab| tab.document.path.as_ref() == Some(path))
            .unwrap_or(index)
    }

    pub(super) fn view_survives(&self, index: usize, closing: &[u64]) -> bool {
        let Some(path) = self.tabs[index].document.path.as_ref() else {
            return false;
        };
        self.tabs
            .iter()
            .any(|tab| tab.document.path.as_ref() == Some(path) && !closing.contains(&tab.id))
    }

    pub(super) fn open_another_view(&mut self, source: usize) {
        self.tabs[source].sync_document();
        let document = self.tabs[source].document.clone();
        let current = self.active;
        if self.tabs[current].document.path.is_none()
            && !self.tabs[current].document.dirty
            && self.tabs[current].editor.is_empty()
        {
            let id = self.tabs[current].id;
            self.tabs[current] = Tab::new(id, document);
            return;
        }

        let id = self.next_id();
        if let Some((pane, _)) = self
            .panes
            .split(pane_grid::Axis::Vertical, self.active_pane, id)
        {
            self.tabs.push(Tab::new(id, document));
            self.groups[self.active_group].tabs.push(id);
            self.active = self.tabs.len() - 1;
            self.active_pane = pane;
        }
    }

    pub(super) fn sync_shared_views(&mut self, source: usize, text_changed: bool) {
        let Some(path) = self.tabs[source].document.path.clone() else {
            return;
        };
        if !self
            .tabs
            .iter()
            .enumerate()
            .any(|(index, tab)| index != source && tab.document.path.as_ref() == Some(&path))
        {
            return;
        }
        self.tabs[source].sync_document();
        let document = self.tabs[source].document.clone();
        let last_edit = self.tabs[source].last_edit;
        let conflict = self.tabs[source].conflict;
        for (index, tab) in self.tabs.iter_mut().enumerate() {
            if index == source || tab.document.path.as_ref() != Some(&path) {
                continue;
            }
            if text_changed {
                let cursor = tab.editor.cursor();
                tab.editor = text_editor::Content::with_text(&document.contents);
                tab.editor.move_to(cursor);
                tab.refresh_reading();
                tab.history.clear();
            }
            tab.document = document.clone();
            tab.last_edit = last_edit;
            tab.conflict = conflict;
        }
    }

    pub(super) fn mirror_edit(
        &mut self,
        source: usize,
        before: text_editor::Cursor,
        actions: &[text_editor::Action],
    ) {
        let Some(path) = self.tabs[source].document.path.clone() else {
            return;
        };
        if !self
            .tabs
            .iter()
            .enumerate()
            .any(|(index, tab)| index != source && tab.document.path.as_ref() == Some(&path))
        {
            return;
        }
        let revision = self.tabs[source].document.revision;
        let last_edit = self.tabs[source].last_edit;
        for (index, tab) in self.tabs.iter_mut().enumerate() {
            if index == source || tab.document.path.as_ref() != Some(&path) {
                continue;
            }
            let cursor = tab.editor.cursor();
            tab.editor.move_to(before);
            for action in actions {
                tab.editor.perform(action.clone());
            }
            tab.editor.move_to(cursor);
            tab.document.revision = revision;
            tab.document.dirty = true;
            tab.last_edit = last_edit;
            tab.history.clear();
            tab.refresh_reading();
        }
    }
}

#[cfg(test)]
mod tests;
