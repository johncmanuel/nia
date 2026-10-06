use super::{App, Tab};
use crate::markdown::{self, Heading};
use crate::search;
use iced::widget::text_editor;
use std::path::PathBuf;

impl App {
    pub(crate) fn current(&self) -> &Tab {
        &self.tabs[self.active]
    }

    pub(crate) fn needs_tick(&self) -> bool {
        self.tabs.iter().any(|tab| {
            tab.document.path.is_some() || (tab.document.dirty && tab.last_edit.is_some())
        })
    }

    pub(super) fn current_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active]
    }

    pub(super) fn clear_search(&mut self) {
        self.show_search = false;
        self.search_query.clear();
        self.search_index = None;
        self.search_hits.clear();
        self.show_outline = false;
        self.outline_query.clear();
        self.outline_index = 0;
        self.show_recent = false;
        self.recent_query.clear();
        self.recent_index = 0;
    }

    pub(crate) fn recent_items(&self) -> Vec<&PathBuf> {
        let query = self.recent_query.to_lowercase();
        self.recent
            .iter()
            .filter(|path| path.to_string_lossy().to_lowercase().contains(&query))
            .collect()
    }

    pub(crate) fn outline_items(&self) -> Vec<Heading> {
        let query = self.outline_query.to_lowercase();
        markdown::headings(&self.current().editor.text())
            .into_iter()
            .filter(|heading| heading.title.to_lowercase().contains(&query))
            .collect()
    }

    pub(crate) fn title(&self) -> String {
        let document = &self.current().document;
        format!(
            "{}{} — nia",
            document.file_name(),
            if document.dirty { " •" } else { "" }
        )
    }

    pub(crate) fn search_matches(&self) -> &[(text_editor::Position, text_editor::Position)] {
        &self.search_hits
    }

    pub(super) fn refresh_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_hits.clear();
            return;
        }
        self.search_hits = self
            .current()
            .editor
            .lines()
            .enumerate()
            .flat_map(|(line, content)| {
                search::matches(
                    &content.text,
                    &self.search_query,
                    self.search_case_sensitive,
                )
                .into_iter()
                .map(move |range| {
                    (
                        text_editor::Position {
                            line,
                            column: range.start,
                        },
                        text_editor::Position {
                            line,
                            column: range.end,
                        },
                    )
                })
            })
            .collect();
    }

    pub(super) fn jump_match(&mut self, index: usize) {
        if let Some((start, end)) = self.search_matches().get(index).copied() {
            self.current_mut().editor.move_to(text_editor::Cursor {
                position: end,
                selection: Some(start),
            });
            self.search_index = Some(index);
        }
    }
}
