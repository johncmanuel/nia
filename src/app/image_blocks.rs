use super::Tab;
use crate::markdown;
use iced::widget::text_editor::{self, Action, Cursor, Edit, Motion, Position};

// blocks have separate selections; a custom editor is needed for cross-block selection.
pub(crate) struct ImageBlock {
    pub(crate) content: text_editor::Content,
    pub(crate) url: Option<String>,
}

impl Tab {
    pub(crate) fn rebuild_image_blocks(&mut self) {
        let source = self.editor.text();
        let images = markdown::standalone_images(&source);
        if images.is_empty() {
            self.image_blocks.clear();
            self.active_image_block = 0;
            return;
        }

        let lines: Vec<_> = source.split_inclusive('\n').collect();
        let mut blocks = Vec::with_capacity(images.len() * 2 + 1);
        let mut next = 0;
        for (line, url) in images {
            blocks.push(ImageBlock {
                content: text_editor::Content::with_text(&lines[next..line].concat()),
                url: None,
            });
            blocks.push(ImageBlock {
                content: text_editor::Content::with_text(lines[line]),
                url: Some(url),
            });
            next = line + 1;
        }
        blocks.push(ImageBlock {
            content: text_editor::Content::with_text(&lines[next..].concat()),
            url: None,
        });
        self.image_blocks = blocks;
        self.move_image_cursor(self.editor.cursor());
    }

    pub(crate) fn move_image_cursor(&mut self, cursor: Cursor) {
        self.editor.move_to(cursor);
        if self.image_blocks.is_empty() {
            return;
        }
        let mut start = 0;
        for (index, block) in self.image_blocks.iter().enumerate() {
            let lines = source_lines(block);
            let last = index + 1 == self.image_blocks.len();
            if cursor.position.line < start + lines || last {
                self.active_image_block = index;
                self.image_blocks[index].content.move_to(Cursor {
                    position: Position {
                        line: cursor.position.line.saturating_sub(start),
                        column: cursor.position.column,
                    },
                    selection: cursor.selection.map(|selection| Position {
                        line: selection.line.saturating_sub(start),
                        column: selection.column,
                    }),
                });
                return;
            }
            start += lines;
        }
    }

    pub(crate) fn activate_image_block(&mut self, index: usize) {
        if index >= self.image_blocks.len() {
            return;
        }
        self.active_image_block = index;
        let start: usize = self.image_blocks[..index].iter().map(source_lines).sum();
        let cursor = offset_cursor(self.image_blocks[index].content.cursor(), start);
        self.editor.move_to(cursor);
    }

    pub(crate) fn perform_image_action(&mut self, index: usize, action: Action) -> bool {
        if index >= self.image_blocks.len() {
            return false;
        }
        let start: usize = self.image_blocks[..index].iter().map(source_lines).sum();
        let before = self.image_blocks[index].content.cursor();
        let prior = matches!(
            action,
            Action::Edit(Edit::Backspace | Edit::Delete | Edit::Unindent)
        )
        .then(|| self.image_blocks[index].content.text());
        self.image_blocks[index].content.perform(action.clone());
        let after = self.image_blocks[index].content.cursor();
        let changed = if let Some(text) = prior {
            text != self.image_blocks[index].content.text()
        } else {
            action.is_edit()
        };
        self.active_image_block = index;
        if changed {
            self.editor.move_to(offset_cursor(before, start));
            self.editor.perform(action.clone());
        }
        self.editor.move_to(offset_cursor(after, start));
        if !changed && before.position == after.position && before.selection == after.selection {
            let next = match action {
                Action::Move(Motion::Down | Motion::Right) => index.checked_add(1),
                Action::Move(Motion::Up | Motion::Left) => index.checked_sub(1),
                _ => None,
            };
            if let Some(next) = next.filter(|next| *next < self.image_blocks.len()) {
                let target = &mut self.image_blocks[next].content;
                if next < index {
                    target.perform(Action::Move(Motion::DocumentEnd));
                } else {
                    target.perform(Action::Move(Motion::DocumentStart));
                }
                self.activate_image_block(next);
            }
        }
        if changed && self.image_blocks[index].url.is_some() {
            let source = self.image_blocks[index].content.text();
            let mut images = markdown::standalone_images(&source);
            if source.lines().count() == 1 && images.len() == 1 {
                self.image_blocks[index].url = images.pop().map(|(_, url)| url);
            } else {
                self.rebuild_image_blocks();
            }
        } else if changed && changes_image_structure(&action) {
            let line = after.position.line;
            let pasted_image =
                matches!(&action, Action::Edit(Edit::Paste(text)) if text.contains("!["));
            if pasted_image
                || [line, line.saturating_sub(1)].into_iter().any(|line| {
                    self.image_blocks[index]
                        .content
                        .line(line)
                        .is_some_and(|line| line.text.contains("!["))
                })
            {
                self.rebuild_image_blocks();
            }
        }
        changed
    }
}

fn source_lines(block: &ImageBlock) -> usize {
    let count = block.content.line_count();
    count.saturating_sub(usize::from(
        block
            .content
            .line(count.saturating_sub(1))
            .is_some_and(|line| line.text.is_empty()),
    ))
}

fn offset_cursor(cursor: Cursor, lines: usize) -> Cursor {
    Cursor {
        position: Position {
            line: cursor.position.line + lines,
            column: cursor.position.column,
        },
        selection: cursor.selection.map(|position| Position {
            line: position.line + lines,
            column: position.column,
        }),
    }
}

pub(super) fn changes_image_structure(action: &Action) -> bool {
    matches!(
        action,
        Action::Edit(
            Edit::Insert(')') | Edit::Paste(_) | Edit::Enter | Edit::Backspace | Edit::Delete
        )
    )
}
