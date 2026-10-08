use super::{App, GroupLayout, Tab, TabGroup};
use crate::document::Document;
use crate::recovery;
use iced::widget::pane_grid;

impl App {
    pub(super) fn neighbor_pane(&self, forward: bool) -> Option<pane_grid::Pane> {
        fn collect(node: &pane_grid::Node, panes: &mut Vec<pane_grid::Pane>) {
            match node {
                pane_grid::Node::Split { a, b, .. } => {
                    collect(a, panes);
                    collect(b, panes);
                }
                pane_grid::Node::Pane(pane) => panes.push(*pane),
            }
        }

        let mut panes = Vec::with_capacity(self.panes.len());
        collect(self.panes.layout(), &mut panes);
        let current = panes.iter().position(|pane| *pane == self.active_pane)?;
        let next = if forward {
            (current + 1) % panes.len()
        } else {
            (current + panes.len() - 1) % panes.len()
        };
        panes.get(next).copied()
    }

    pub(super) fn remove_tab(&mut self, index: usize) {
        let id = self.tabs[index].id;
        let previous_group = self.active_group;
        let Some(owner) = self
            .groups
            .iter()
            .position(|group| group.tabs.contains(&id))
        else {
            return;
        };
        self.select_group(owner);
        let closed = self.tabs.remove(index);
        if let Err(error) = recovery::remove(&closed.recovery_key) {
            log::warn!("recovery cleanup failed: {:?}", error.kind());
        }
        self.groups[self.active_group]
            .tabs
            .retain(|tab| *tab != closed.id);
        if self.groups[self.active_group].tabs.is_empty() && self.groups.len() > 1 {
            let removed = self.active_group;
            let next = if previous_group != removed {
                previous_group
            } else if removed == 0 {
                1
            } else {
                removed - 1
            };
            self.select_group(next);
            self.groups.remove(removed);
            if removed < self.active_group {
                self.active_group -= 1;
            }
        } else if self.tabs.is_empty() {
            let id = self.next_id();
            self.tabs.push(Tab::new(id, Document::new()));
            self.groups[self.active_group].tabs.push(id);
            if let Some(tab_id) = self.panes.get_mut(self.active_pane) {
                *tab_id = id;
            }
        } else {
            self.remove_pane_tab(closed.id);
            if previous_group != owner {
                self.select_group(previous_group);
            }
        }
        self.activate_visible_tab();
        self.clear_search();
        log::info!("document tab closed");
    }

    pub(super) fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub(super) fn select_tab(&mut self, index: usize) {
        if index != self.active {
            self.current_mut().history.break_group();
            self.tabs[index].history.break_group();
        }
        let id = self.tabs[index].id;
        if let Some(group) = self
            .groups
            .iter()
            .position(|group| group.tabs.contains(&id))
        {
            self.select_group(group);
        }
        let other_pane = self
            .panes
            .iter()
            .find_map(|(&pane, &tab_id)| (tab_id == id).then_some(pane));
        if let Some(pane) = other_pane {
            self.active_pane = pane;
        } else if let Some(active) = self.panes.get_mut(self.active_pane) {
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
            let replacement = self.groups[self.active_group]
                .tabs
                .iter()
                .find(|&&candidate| {
                    !self
                        .panes
                        .iter()
                        .any(|(_, visible_id)| *visible_id == candidate)
                })
                .copied();
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
                *tab_id = self.groups[self.active_group]
                    .tabs
                    .first()
                    .copied()
                    .unwrap_or(id);
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

    pub(super) fn push_group(&mut self, document: Document) {
        let id = self.next_id();
        self.push_group_tab(Tab::new(id, document));
    }

    pub(super) fn push_recovered_group(&mut self, draft: recovery::Draft) {
        let id = self.next_id();
        self.push_group_tab(Tab::recovered(id, draft));
    }

    fn push_group_tab(&mut self, tab: Tab) {
        self.current_mut().history.break_group();
        let id = tab.id;
        self.tabs.push(tab);
        let (panes, active_pane) = pane_grid::State::new(id);
        let previous = GroupLayout {
            panes: std::mem::replace(&mut self.panes, panes),
            active_pane: std::mem::replace(&mut self.active_pane, active_pane),
        };
        self.groups[self.active_group].inactive = Some(previous);
        self.groups.push(TabGroup {
            tabs: vec![id],
            inactive: None,
        });
        self.active_group = self.groups.len() - 1;
        self.active = self.tabs.len() - 1;
        self.show_settings = false;
        self.clear_search();
        self.notice = None;
    }

    pub(super) fn select_group(&mut self, index: usize) {
        if index == self.active_group || index >= self.groups.len() {
            return;
        }
        let Some(target) = self.groups[index].inactive.take() else {
            return;
        };
        if let Some(active) = self.tabs.get_mut(self.active) {
            active.history.break_group();
        }
        let previous = GroupLayout {
            panes: std::mem::replace(&mut self.panes, target.panes),
            active_pane: std::mem::replace(&mut self.active_pane, target.active_pane),
        };
        self.groups[self.active_group].inactive = Some(previous);
        self.active_group = index;
        self.activate_visible_tab();
        if let Some(active) = self.tabs.get_mut(self.active) {
            active.history.break_group();
        }
    }

    pub(crate) fn group_label(&self, index: usize) -> String {
        self.group_active_id(index)
            .and_then(|id| self.tabs.iter().find(|tab| tab.id == id))
            .map_or_else(|| "Untitled".into(), |tab| tab.document.file_name())
    }

    pub(crate) fn group_active_id(&self, index: usize) -> Option<u64> {
        let group = self.groups.get(index)?;
        if index == self.active_group {
            self.panes.get(self.active_pane).copied()
        } else {
            group
                .inactive
                .as_ref()
                .and_then(|layout| layout.panes.get(layout.active_pane).copied())
        }
    }
}
