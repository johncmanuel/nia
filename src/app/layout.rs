use super::{App, Tab};
use crate::document::Document;
use crate::recovery;

impl App {
    pub(super) fn remove_tab(&mut self, index: usize) {
        let closed = self.tabs.remove(index);
        if recovery::remove(&closed.recovery_key).is_err() {
            log::warn!("recovery cleanup failed");
        }
        if self.tabs.is_empty() {
            let id = self.next_id();
            self.tabs.push(Tab::new(id, Document::new()));
        }
        self.remove_pane_tab(closed.id);
        if index < self.active {
            self.active -= 1;
        }
        self.active = self.active.min(self.tabs.len() - 1);
        self.activate_visible_tab();
        self.clear_search();
        log::info!("document tab closed");
    }

    pub(super) fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub(super) fn push_tab(&mut self, document: Document) {
        let id = self.next_id();
        self.tabs.push(Tab::new(id, document));
        self.active = self.tabs.len() - 1;
        if let Some(tab_id) = self.panes.get_mut(self.active_pane) {
            *tab_id = id;
        }
        self.show_keybindings = false;
        self.clear_search();
        self.notice = None;
    }

    pub(super) fn select_tab(&mut self, index: usize) {
        let id = self.tabs[index].id;
        let other_pane = self
            .panes
            .iter()
            .find_map(|(&pane, &tab_id)| (tab_id == id).then_some(pane));
        if let Some(pane) = other_pane
            && pane != self.active_pane
        {
            let previous = self.panes.get(self.active_pane).copied();
            if let (Some(previous), Some(other)) = (previous, self.panes.get_mut(pane)) {
                *other = previous;
            }
        }
        if let Some(active) = self.panes.get_mut(self.active_pane) {
            *active = id;
        }
        self.active = index;
    }

    pub(super) fn remove_pane_tab(&mut self, id: u64) {
        let panes: Vec<_> = self
            .panes
            .iter()
            .filter_map(|(&pane, &tab_id)| (tab_id == id).then_some(pane))
            .collect();
        for pane in panes {
            let replacement = self
                .tabs
                .iter()
                .find(|tab| {
                    !self
                        .panes
                        .iter()
                        .any(|(_, visible_id)| *visible_id == tab.id)
                })
                .map(|tab| tab.id);
            if let Some(replacement) = replacement
                && let Some(tab_id) = self.panes.get_mut(pane)
            {
                *tab_id = replacement;
                continue;
            }
            if self.panes.len() > 1 {
                if let Some((_, sibling)) = self.panes.close(pane)
                    && self.active_pane == pane
                {
                    self.active_pane = sibling;
                }
            } else if let Some(tab_id) = self.panes.get_mut(pane) {
                *tab_id = self.tabs.first().map_or(id, |tab| tab.id);
            }
        }
    }

    pub(super) fn activate_visible_tab(&mut self) {
        if let Some(&id) = self.panes.get(self.active_pane)
            && let Some(index) = self.tabs.iter().position(|tab| tab.id == id)
        {
            self.active = index;
        }
    }
}
