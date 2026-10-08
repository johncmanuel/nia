use super::App;
use crate::search;
use iced::widget::text_editor;

#[derive(Clone, Debug)]
pub(crate) struct TabMatch {
    pub(crate) tab_id: u64,
    pub(crate) start: text_editor::Position,
    pub(crate) end: text_editor::Position,
    pub(crate) preview: String,
}

impl App {
    pub(super) fn refresh_tab_search(&mut self) {
        self.tab_search_hits.clear();
        if self.search_query.is_empty() {
            return;
        }
        for tab in &self.tabs {
            for (line, content) in tab.editor.lines().enumerate() {
                let matches = search::matches(
                    &content.text,
                    &self.search_query,
                    self.search_case_sensitive,
                );
                let preview: String = content.text.trim().chars().take(100).collect();
                self.tab_search_hits
                    .extend(matches.into_iter().map(|range| TabMatch {
                        tab_id: tab.id,
                        start: text_editor::Position {
                            line,
                            column: range.start,
                        },
                        end: text_editor::Position {
                            line,
                            column: range.end,
                        },
                        preview: preview.clone(),
                    }));
            }
        }
    }

    pub(super) fn jump_tab_search(&mut self, index: usize) -> bool {
        let Some(hit) = self.tab_search_hits.get(index).cloned() else {
            return false;
        };
        let Some(tab_index) = self.tabs.iter().position(|tab| tab.id == hit.tab_id) else {
            return false;
        };
        self.select_tab(tab_index);
        self.current_mut().reading = None;
        self.current_mut().move_image_cursor(text_editor::Cursor {
            position: hit.end,
            selection: Some(hit.start),
        });
        self.search_index = Some(index);
        true
    }
}
