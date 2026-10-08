use iced::widget::text_editor::{Action, Content, Cursor, Edit, Position};
use std::time::{Duration, Instant};

const HISTORY_LIMIT: usize = 1_000;
const GROUP_PAUSE: Duration = Duration::from_millis(750);

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
    kind: Option<Kind>,
    closed: bool,
    last_edit: Instant,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Typing,
    Backspace,
    Delete,
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
    kind: Option<Kind>,
    boundary: bool,
    bytes: usize,
}

impl History {
    pub(super) fn begin(&mut self, content: &Content, action: &Action) -> Option<Pending> {
        let Action::Edit(edit) = action else {
            self.break_group();
            return None;
        };
        let cursor = content.cursor();
        let character = content
            .line(cursor.position.line)
            .and_then(|line| match edit {
                Edit::Insert(character) => Some(*character),
                Edit::Backspace => line.text.get(..cursor.position.column)?.chars().next_back(),
                Edit::Delete => line.text.get(cursor.position.column..)?.chars().next(),
                _ => None,
            });
        let kind = if cursor.selection.is_some() {
            None
        } else {
            match edit {
                Edit::Insert(_) => Some(Kind::Typing),
                Edit::Backspace if cursor.position.column > 0 => Some(Kind::Backspace),
                Edit::Delete => Some(Kind::Delete),
                _ => None,
            }
        };
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
            kind,
            boundary: character.is_some_and(|character| !character.is_alphanumeric()),
            bytes: character.map_or(0, char::len_utf8),
        })
    }

    pub(super) fn finish(&mut self, content: &Content, pending: Option<Pending>) {
        self.finish_at(content, pending, Instant::now());
    }

    fn finish_at(&mut self, content: &Content, pending: Option<Pending>, now: Instant) {
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
        let after_cursor = content.cursor();
        let delta = pending.before.text.len().abs_diff(after.text.len());
        let kind = pending.kind.filter(|kind| {
            pending.before_count == content.line_count()
                && pending.bytes == delta
                && after_cursor.selection.is_none()
                && after_cursor.position.line == pending.before_cursor.position.line
                && match kind {
                    Kind::Typing => {
                        after_cursor.position.column
                            == pending.before_cursor.position.column + pending.bytes
                    }
                    Kind::Backspace => {
                        after_cursor.position.column + pending.bytes
                            == pending.before_cursor.position.column
                    }
                    Kind::Delete => after_cursor.position == pending.before_cursor.position,
                }
        });
        let change = Change {
            before: pending.before,
            after,
            before_cursor: pending.before_cursor,
            after_cursor,
            kind,
            closed: pending.boundary || kind.is_none(),
            last_edit: now,
        };
        if let Some(previous) = self.undo.last_mut()
            && !previous.closed
            && kind.is_some()
            && previous.kind == kind
            && now.saturating_duration_since(previous.last_edit) <= GROUP_PAUSE
            && previous.after_cursor.position == change.before_cursor.position
            && previous.after_cursor.selection.is_none()
            && change.before_cursor.selection.is_none()
            && previous.after.start_line == change.before.start_line
            && previous.after.end_line == change.before.end_line
            && previous.after.text == change.before.text
        {
            previous.after = change.after;
            previous.after_cursor = change.after_cursor;
            previous.closed = change.closed;
            previous.last_edit = now;
            self.redo.clear();
            return;
        }
        self.undo.push(change);
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub(super) fn break_group(&mut self) {
        if let Some(change) = self.undo.last_mut() {
            change.closed = true;
        }
    }

    pub(super) fn undo(&mut self, content: &mut Content) -> bool {
        let Some(change) = self.undo.pop() else {
            return false;
        };
        self.break_group();
        change.before.replace(content, &change.after);
        content.move_to(change.before_cursor);
        self.redo.push(Change {
            closed: true,
            ..change
        });
        true
    }

    pub(super) fn redo(&mut self, content: &mut Content) -> bool {
        let Some(change) = self.redo.pop() else {
            return false;
        };
        change.after.replace(content, &change.before);
        content.move_to(change.after_cursor);
        self.undo.push(Change {
            closed: true,
            ..change
        });
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
