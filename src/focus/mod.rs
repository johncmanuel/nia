use crate::config;
use iced::widget::text_editor;
use std::fmt;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

pub(crate) const MAX_CONTEXT: u8 = 30;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Dimming {
    #[default]
    Off,
    Paragraph,
    Sentence,
}

impl Dimming {
    pub(crate) const ALL: [Self; 3] = [Self::Off, Self::Paragraph, Self::Sentence];

    fn name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Paragraph => "Paragraph",
            Self::Sentence => "Sentence",
        }
    }
}

impl fmt::Display for Dimming {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) typewriter: bool,
    pub(crate) highlight_line: bool,
    pub(crate) lines_above: u8,
    pub(crate) lines_below: u8,
    pub(crate) dimming: Dimming,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            typewriter: false,
            highlight_line: false,
            lines_above: 8,
            lines_below: 12,
            dimming: Dimming::Off,
        }
    }
}

impl Settings {
    pub(crate) fn load() -> io::Result<Self> {
        Self::load_from(&config::directory()?.join("focus.conf"))
    }

    fn load_from(path: &Path) -> io::Result<Self> {
        let value = match fs::read_to_string(path) {
            Ok(value) => value,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let mut parts = value.lines();
        let invalid = || io::Error::new(ErrorKind::InvalidData, "invalid focus settings");
        let parse_bool = |value: Option<&str>| match value {
            Some("true") => Ok(true),
            Some("false") => Ok(false),
            _ => Err(invalid()),
        };
        let parse_count = |value: Option<&str>| {
            value
                .and_then(|value| value.parse::<u8>().ok())
                .filter(|count| *count <= MAX_CONTEXT)
                .ok_or_else(invalid)
        };
        let settings = Self {
            typewriter: parse_bool(parts.next())?,
            highlight_line: parse_bool(parts.next())?,
            lines_above: parse_count(parts.next())?,
            lines_below: parse_count(parts.next())?,
            dimming: match parts.next() {
                Some("Off") => Dimming::Off,
                Some("Paragraph") => Dimming::Paragraph,
                Some("Sentence") => Dimming::Sentence,
                _ => return Err(invalid()),
            },
        };
        if parts.next().is_some() {
            return Err(invalid());
        }
        Ok(settings)
    }

    pub(crate) fn save(self) -> io::Result<()> {
        self.save_to(&config::directory()?.join("focus.conf"))
    }

    fn save_to(self, path: &Path) -> io::Result<()> {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(ErrorKind::InvalidInput, "invalid focus settings path")
        })?;
        fs::create_dir_all(parent)?;
        fs::write(
            path,
            format!(
                "{}\n{}\n{}\n{}\n{}\n",
                self.typewriter,
                self.highlight_line,
                self.lines_above,
                self.lines_below,
                self.dimming.name()
            ),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Point {
    pub(crate) line: usize,
    pub(crate) byte: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) start: Point,
    pub(crate) end: Point,
}

pub(crate) fn active_span(editor: &text_editor::Content, dimming: Dimming) -> Option<Span> {
    if dimming == Dimming::Off {
        return None;
    }
    let cursor = editor.cursor().position;
    if editor.line(cursor.line)?.text.trim().is_empty() {
        let point = Point {
            line: cursor.line,
            byte: 0,
        };
        return Some(Span {
            start: point,
            end: point,
        });
    }
    let mut first = cursor.line;
    while first > 0
        && editor
            .line(first - 1)
            .is_some_and(|line| !line.text.trim().is_empty())
    {
        first -= 1;
    }
    let mut last = cursor.line;
    while last + 1 < editor.line_count()
        && editor
            .line(last + 1)
            .is_some_and(|line| !line.text.trim().is_empty())
    {
        last += 1;
    }
    let last_len = editor.line(last)?.text.len();
    if dimming == Dimming::Paragraph {
        return Some(Span {
            start: Point {
                line: first,
                byte: 0,
            },
            end: Point {
                line: last,
                byte: last_len,
            },
        });
    }

    let mut text = String::new();
    let mut starts = Vec::with_capacity(last - first + 1);
    let mut lengths = Vec::with_capacity(last - first + 1);
    let mut cursor_byte = 0;
    for line in first..=last {
        let content = editor.line(line)?.text;
        starts.push(text.len());
        lengths.push(content.len());
        if line == cursor.line {
            cursor_byte = text.len() + cursor.column.min(content.len());
        }
        text.push_str(&content);
        if line != last {
            text.push('\n');
        }
    }
    let mut sentence_start = 0;
    let mut sentence_end = text.len();
    for (index, character) in text.char_indices() {
        let end = index + character.len_utf8();
        if matches!(character, '.' | '!' | '?')
            && text[end..].chars().next().is_none_or(char::is_whitespace)
        {
            if cursor_byte <= end {
                sentence_end = end;
                break;
            }
            sentence_start = end
                + text[end..]
                    .len()
                    .saturating_sub(text[end..].trim_start().len());
        }
    }
    let to_point = |offset: usize| {
        let index = starts
            .partition_point(|start| *start <= offset)
            .saturating_sub(1);
        Point {
            line: first + index,
            byte: (offset - starts[index]).min(lengths[index]),
        }
    };
    Some(Span {
        start: to_point(sentence_start),
        end: to_point(sentence_end),
    })
}

#[cfg(test)]
mod tests;
