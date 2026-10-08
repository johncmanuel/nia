use super::{App, EDITOR, Message, update};
use iced::Task;
use iced::widget::{
    operation,
    text_editor::{Action, Content, Edit, Motion},
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Bold,
    Italic,
    Table,
    Blockquote,
    OrderedList,
    Code,
    FencedCode,
    Link,
    HorizontalRule,
    Footnote,
    DefinitionList,
    Strikethrough,
    TaskList,
    Emoji,
    Highlight,
    Subscript,
    Superscript,
}

impl Format {
    pub(crate) const ALL: [Self; 17] = [
        Self::Bold,
        Self::Italic,
        Self::Table,
        Self::Blockquote,
        Self::OrderedList,
        Self::Code,
        Self::FencedCode,
        Self::Link,
        Self::HorizontalRule,
        Self::Footnote,
        Self::DefinitionList,
        Self::Strikethrough,
        Self::TaskList,
        Self::Emoji,
        Self::Highlight,
        Self::Subscript,
        Self::Superscript,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Bold => "Bold",
            Self::Italic => "Italic",
            Self::Table => "Table",
            Self::Blockquote => "Blockquote",
            Self::OrderedList => "Ordered list",
            Self::Code => "Code",
            Self::FencedCode => "Fenced code block",
            Self::Link => "Link",
            Self::HorizontalRule => "Horizontal rule",
            Self::Footnote => "Footnote",
            Self::DefinitionList => "Definition list",
            Self::Strikethrough => "Strikethrough",
            Self::TaskList => "Task list",
            Self::Emoji => "Emoji",
            Self::Highlight => "Highlight",
            Self::Subscript => "Subscript",
            Self::Superscript => "Superscript",
        }
    }

    pub(crate) fn replacement(self, selection: Option<&str>) -> (String, usize) {
        let selected = selection.unwrap_or_default();
        let wrap = |open: &str, close: &str| (format!("{open}{selected}{close}"), close.len());
        match self {
            Self::Bold => wrap("**", "**"),
            Self::Italic => wrap("*", "*"),
            Self::Code => wrap("`", "`"),
            Self::Strikethrough => wrap("~~", "~~"),
            Self::Highlight => wrap("==", "=="),
            Self::Subscript => wrap("~", "~"),
            Self::Superscript => wrap("^", "^"),
            Self::Emoji => {
                let name = if selected.is_empty() {
                    "emoji"
                } else {
                    selected
                };
                (format!(":{name}:"), 1)
            }
            Self::Link => {
                let title = if selected.is_empty() {
                    "text"
                } else {
                    selected
                };
                (
                    format!("[{title}](url)"),
                    if selected.is_empty() { 6 } else { 4 },
                )
            }
            Self::Blockquote => (
                prefix_lines(selected, |_, line| format!("> {line}"), "> "),
                0,
            ),
            Self::OrderedList => (
                prefix_lines(
                    selected,
                    |index, line| format!("{}. {line}", index + 1),
                    "1. ",
                ),
                0,
            ),
            Self::TaskList => (
                prefix_lines(selected, |_, line| format!("- [ ] {line}"), "- [ ] "),
                0,
            ),
            Self::FencedCode => (format!("```\n{selected}\n```"), 4),
            Self::Table => {
                let rows = if selected.is_empty() {
                    "| Cell 1 | Cell 2 |".into()
                } else {
                    selected
                        .lines()
                        .map(|line| format!("| {line} |  |"))
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                (format!("| Column 1 | Column 2 |\n| --- | --- |\n{rows}"), 0)
            }
            Self::HorizontalRule => (format!("{selected}\n---\n"), 0),
            Self::Footnote => (format!("{selected}[^1]\n\n[^1]: "), 0),
            Self::DefinitionList => {
                let term = if selected.is_empty() {
                    "term"
                } else {
                    selected
                };
                (format!("{term}\n: definition"), 0)
            }
        }
    }

    fn needs_own_line(self) -> bool {
        matches!(
            self,
            Self::Table
                | Self::Blockquote
                | Self::OrderedList
                | Self::FencedCode
                | Self::DefinitionList
                | Self::TaskList
        )
    }
}

fn prefix_lines(text: &str, prefix: impl Fn(usize, &str) -> String, empty: &str) -> String {
    if text.is_empty() {
        return empty.into();
    }
    text.split('\n')
        .enumerate()
        .map(|(index, line)| prefix(index, line))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) enum AutoEdit {
    Replace(String, usize),
    SkipCloser,
    DeletePair,
}

impl AutoEdit {
    pub(crate) fn perform(self, mut perform: impl FnMut(Action)) -> bool {
        match self {
            Self::Replace(text, move_left) => {
                perform(Action::Edit(Edit::Paste(Arc::new(text))));
                for _ in 0..move_left {
                    perform(Action::Move(Motion::Left));
                }
                true
            }
            Self::SkipCloser => {
                perform(Action::Move(Motion::Right));
                false
            }
            Self::DeletePair => {
                perform(Action::Edit(Edit::Backspace));
                perform(Action::Edit(Edit::Delete));
                true
            }
        }
    }
}

pub(super) fn insert(state: &mut App, format: Format) -> Task<Message> {
    if state.loading
        || state.dialog_open
        || state.pending_close.is_some()
        || state.show_settings
        || state.show_search
        || state.show_recent
        || state.handbook.is_some()
    {
        return Task::none();
    }
    state.show_format_menu = false;
    state.current_mut().reading = None;
    let tab = state.current();
    let content = if tab.image_blocks.is_empty() {
        &tab.editor
    } else {
        tab.image_blocks
            .get(tab.active_image_block)
            .map_or(&tab.editor, |block| &block.content)
    };
    let selection = content.selection();
    let (mut replacement, mut move_left) = if format == Format::Footnote {
        let label = next_footnote_label(&tab.editor.text());
        (
            format!(
                "{}[^{label}]\n\n[^{label}]: ",
                selection.as_deref().unwrap_or_default()
            ),
            0,
        )
    } else {
        format.replacement(selection.as_deref())
    };
    if format.needs_own_line() {
        let (before, after) = surrounding_text(content);
        if before {
            replacement.insert(0, '\n');
        }
        if after {
            replacement.push('\n');
            move_left += 1;
        }
    }
    let id = tab.id;
    let message = if tab.image_blocks.is_empty() {
        Message::Edit(id, Action::Edit(Edit::Paste(Arc::new(replacement))))
    } else {
        Message::EditImageBlock(
            id,
            tab.active_image_block,
            Action::Edit(Edit::Paste(Arc::new(replacement))),
        )
    };
    let task = update(state, message);
    for _ in 0..move_left {
        state
            .current_mut()
            .editor
            .perform(Action::Move(Motion::Left));
    }
    let cursor = state.current().editor.cursor();
    state.current_mut().move_image_cursor(cursor);
    Task::batch([task, operation::focus(EDITOR)])
}

fn surrounding_text(content: &Content) -> (bool, bool) {
    let cursor = content.cursor();
    let selection = cursor.selection.unwrap_or(cursor.position);
    let (start, end) =
        if (cursor.position.line, cursor.position.column) <= (selection.line, selection.column) {
            (cursor.position, selection)
        } else {
            (selection, cursor.position)
        };
    let before = content.line(start.line).is_some_and(|line| {
        line.text
            .get(..start.column)
            .is_some_and(|text| !text.is_empty())
    });
    let after = content.line(end.line).is_some_and(|line| {
        line.text
            .get(end.column..)
            .is_some_and(|text| !text.is_empty())
    });
    (before, after)
}

fn next_footnote_label(source: &str) -> usize {
    let mut label = 1usize;
    while source.contains(&format!("[^{label}]")) {
        let Some(next) = label.checked_add(1) else {
            break;
        };
        label = next;
    }
    label
}

pub(crate) fn auto_edit(content: &Content, action: &Action) -> Option<AutoEdit> {
    match action {
        Action::Edit(Edit::Insert(character))
            if matching_close(*character).is_some() || is_closer(*character) => {}
        Action::Edit(Edit::Backspace) => {}
        _ => return None,
    }
    let cursor = content.cursor();
    let selected = content.selection();
    let line = content.line(cursor.position.line)?;
    let bytes = line.text.as_bytes();
    let column = cursor.position.column;
    match action {
        Action::Edit(Edit::Insert(open)) => {
            if selected.is_none() && is_closer(*open) && bytes.get(column) == Some(&(*open as u8)) {
                return Some(AutoEdit::SkipCloser);
            }
            if let Some(close) = matching_close(*open) {
                if *open == '\''
                    && selected.is_none()
                    && column > 0
                    && line
                        .text
                        .get(..column)
                        .and_then(|text| text.chars().last())
                        .is_some_and(char::is_alphanumeric)
                {
                    return None;
                }
                return Some(AutoEdit::Replace(
                    format!("{open}{}{close}", selected.unwrap_or_default()),
                    1,
                ));
            }
        }
        Action::Edit(Edit::Backspace) if selected.is_none() && column > 0 => {
            let open = *bytes.get(column - 1)? as char;
            let close = *bytes.get(column)? as char;
            if matching_close(open) == Some(close) {
                return Some(AutoEdit::DeletePair);
            }
        }
        _ => {}
    }
    None
}

fn matching_close(open: char) -> Option<char> {
    match open {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '<' => Some('>'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '`' => Some('`'),
        _ => None,
    }
}

fn is_closer(character: char) -> bool {
    matches!(character, ')' | ']' | '}' | '>' | '"' | '\'' | '`')
}

#[cfg(test)]
mod tests;
