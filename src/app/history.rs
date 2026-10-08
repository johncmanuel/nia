use iced::widget::text_editor::{Action, Content, Cursor, Edit, Position};

const HISTORY_LIMIT: usize = 1_000;

#[derive(Default)]
pub(super) struct History {
    undo: Vec<Change>,
    redo: Vec<Change>,
}

struct Change {
    before: Fragment,
    after: Fragment,
    before_cursor: Cursor,
    after_cursor: Cursor,
}

struct Fragment {
    start_line: usize,
    end_line: usize,
    text: String,
}

pub(super) struct Pending {
    before: Fragment,
    before_count: usize,
    before_cursor: Cursor,
}

impl History {
    pub(super) fn begin(content: &Content, action: &Action) -> Option<Pending> {
        let Action::Edit(edit) = action else {
            return None;
        };
        let cursor = content.cursor();
        let (mut start, mut end) = match cursor.selection {
            Some(selection) => (
                cursor.position.line.min(selection.line),
                cursor.position.line.max(selection.line),
            ),
            None => (cursor.position.line, cursor.position.line),
        };
        if cursor.selection.is_none() {
            if matches!(edit, Edit::Backspace) && cursor.position.column == 0 {
                start = start.saturating_sub(1);
            }
            if matches!(edit, Edit::Delete)
                && content
                    .line(end)
                    .is_some_and(|line| cursor.position.column == line.text.len())
            {
                end = (end + 1).min(content.line_count() - 1);
            }
        }
        Some(Pending {
            before: Fragment::capture(content, start, end),
            before_count: content.line_count(),
            before_cursor: cursor,
        })
    }

    pub(super) fn finish(&mut self, content: &Content, pending: Option<Pending>) {
        let Some(pending) = pending else {
            return;
        };
        let end = pending.before.end_line as isize + content.line_count() as isize
            - pending.before_count as isize;
        let end = end.max(pending.before.start_line as isize) as usize;
        let after = Fragment::capture(content, pending.before.start_line, end);
        if pending.before.text == after.text {
            return;
        }
        self.undo.push(Change {
            before: pending.before,
            after,
            before_cursor: pending.before_cursor,
            after_cursor: content.cursor(),
        });
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub(super) fn undo(&mut self, content: &mut Content) -> bool {
        let Some(change) = self.undo.pop() else {
            return false;
        };
        change.before.replace(content, &change.after);
        content.move_to(change.before_cursor);
        self.redo.push(change);
        true
    }

    pub(super) fn redo(&mut self, content: &mut Content) -> bool {
        let Some(change) = self.redo.pop() else {
            return false;
        };
        change.after.replace(content, &change.before);
        content.move_to(change.after_cursor);
        self.undo.push(change);
        true
    }

    pub(super) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

impl Fragment {
    fn capture(content: &Content, start_line: usize, end_line: usize) -> Self {
        let mut text = String::new();
        for line in start_line..=end_line {
            if let Some(value) = content.line(line) {
                text.push_str(&value.text);
                if line + 1 < content.line_count() {
                    text.push_str(value.ending.as_str());
                }
            }
        }
        Self {
            start_line,
            end_line,
            text,
        }
    }

    fn replace(&self, content: &mut Content, current: &Self) {
        let end = if current.end_line + 1 < content.line_count() {
            Position {
                line: current.end_line + 1,
                column: 0,
            }
        } else {
            Position {
                line: current.end_line,
                column: content
                    .line(current.end_line)
                    .map_or(0, |line| line.text.len()),
            }
        };
        content.move_to(Cursor {
            position: end,
            selection: Some(Position {
                line: self.start_line,
                column: 0,
            }),
        });
        content.perform(Action::Edit(Edit::Paste(std::sync::Arc::new(
            self.text.clone(),
        ))));
    }
}

#[cfg(test)]
mod tests;
