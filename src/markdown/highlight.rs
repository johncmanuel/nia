use super::{Fence, atx_heading, closes_fence, fenced_start, setext_level};
use crate::focus::Span;
use iced::advanced::text::highlighter;
use iced::widget::text;
use iced::{Font, Theme, font};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Heading,
    Marker,
    Quote,
    Code,
    Emphasis,
    Strong,
    Link,
    Current,
    Dim,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HighlightSettings {
    pub(crate) font: Font,
    pub(crate) active_line: Option<usize>,
    pub(crate) focus: Option<Span>,
    pub(crate) highlight_line: bool,
}

pub(crate) struct MarkdownHighlighter {
    settings: HighlightSettings,
    line: usize,
    fence: Option<Fence>,
    fences_after: Vec<Option<Fence>>,
}

impl text::Highlighter for MarkdownHighlighter {
    type Settings = HighlightSettings;
    type Highlight = (Kind, Font);
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(settings: &HighlightSettings) -> Self {
        Self {
            settings: *settings,
            line: 0,
            fence: None,
            fences_after: Vec::new(),
        }
    }
    fn update(&mut self, settings: &HighlightSettings) {
        let first = if self.settings.font != settings.font
            || self.settings.focus.is_some() != settings.focus.is_some()
        {
            0
        } else {
            [self.settings.active_line, settings.active_line]
                .into_iter()
                .flatten()
                .chain(
                    [
                        self.settings.focus.map(|span| span.start.line),
                        settings.focus.map(|span| span.start.line),
                    ]
                    .into_iter()
                    .flatten(),
                )
                .min()
                .unwrap_or(0)
        };
        self.change_line(first);
        self.settings = *settings;
    }
    fn change_line(&mut self, line: usize) {
        self.fences_after.truncate(line);
        self.fence = line
            .checked_sub(1)
            .and_then(|previous| self.fences_after.get(previous).copied().flatten());
        self.line = line;
    }
    fn current_line(&self) -> usize {
        self.line
    }
    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let line_index = self.line;
        self.line += 1;
        let mut ranges = Vec::new();
        if let Some(fence) = self.fence {
            ranges.push((0..line.len(), Kind::Code));
            if closes_fence(line, fence) {
                self.fence = None;
            }
        } else if let Some(fence) = fenced_start(line) {
            self.fence = Some(fence);
            ranges.push((0..line.len(), Kind::Code));
        } else if atx_heading(line).is_some() || setext_level(line).is_some() {
            ranges.push((0..line.len(), Kind::Heading));
        } else {
            let trimmed = line.trim_start_matches(' ');
            let indent = line.len() - trimmed.len();
            let mut start = indent;
            if trimmed.starts_with('>') {
                ranges.push((indent..indent + 1, Kind::Quote));
                start = indent + 1;
            } else if trimmed.starts_with(['-', '*', '+'])
                && trimmed
                    .as_bytes()
                    .get(1)
                    .is_some_and(u8::is_ascii_whitespace)
            {
                ranges.push((indent..indent + 1, Kind::Marker));
                start = indent + 1;
            } else if let Some(dot) = trimmed.find('.')
                && dot > 0
                && dot <= 9
                && trimmed[..dot].bytes().all(|byte| byte.is_ascii_digit())
                && trimmed
                    .as_bytes()
                    .get(dot + 1)
                    .is_some_and(u8::is_ascii_whitespace)
            {
                ranges.push((indent..indent + dot + 1, Kind::Marker));
                start = indent + dot + 1;
            }
            ranges.extend(inline_ranges(line, start));
        }
        if self.settings.highlight_line && self.settings.active_line == Some(line_index) {
            ranges.push((0..line.len(), Kind::Current));
        }
        if let Some(focus) = self.settings.focus {
            if line_index < focus.start.line || line_index > focus.end.line {
                ranges.push((0..line.len(), Kind::Dim));
            } else {
                if line_index == focus.start.line && focus.start.byte > 0 {
                    ranges.push((0..focus.start.byte.min(line.len()), Kind::Dim));
                }
                if line_index == focus.end.line && focus.end.byte < line.len() {
                    ranges.push((focus.end.byte..line.len(), Kind::Dim));
                }
            }
        }
        self.fences_after.push(self.fence);
        ranges
            .into_iter()
            .map(|(range, kind)| (range, (kind, self.settings.font)))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

fn inline_ranges(line: &str, mut index: usize) -> Vec<(Range<usize>, Kind)> {
    let mut ranges = Vec::new();
    while index < line.len() {
        let rest = &line[index..];
        let matched = if let Some(after) = rest.strip_prefix('`') {
            after.find('`').map(|end| (end + 2, Kind::Code))
        } else if rest.starts_with('[') {
            rest.find("](").and_then(|middle| {
                rest[middle + 2..]
                    .find(')')
                    .map(|end| (middle + end + 3, Kind::Link))
            })
        } else if let Some(after) = rest.strip_prefix("**").or_else(|| rest.strip_prefix("__")) {
            let marker = &rest[..2];
            after.find(marker).map(|end| (end + 4, Kind::Strong))
        } else if let Some(after) = rest.strip_prefix('*').or_else(|| rest.strip_prefix('_')) {
            let marker = &rest[..1];
            after.find(marker).map(|end| (end + 2, Kind::Emphasis))
        } else {
            None
        };
        if let Some((length, kind)) = matched {
            ranges.push((index..index + length, kind));
            index += length;
        } else {
            index += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
    ranges
}

pub(crate) fn format((kind, base_font): &(Kind, Font), theme: &Theme) -> highlighter::Format<Font> {
    let palette = theme.palette();
    if matches!(kind, Kind::Current | Kind::Dim) {
        return highlighter::Format {
            color: Some(match kind {
                Kind::Current => palette.primary,
                Kind::Dim => palette.text.scale_alpha(0.25),
                _ => unreachable!(),
            }),
            font: None,
        };
    }
    let (color, weight) = match kind {
        Kind::Heading => (palette.primary, font::Weight::Bold),
        Kind::Marker | Kind::Quote => (palette.text.scale_alpha(0.58), font::Weight::Normal),
        Kind::Code => (palette.success, font::Weight::Normal),
        Kind::Emphasis => (palette.text, font::Weight::Medium),
        Kind::Strong => (palette.text, font::Weight::Bold),
        Kind::Link => (palette.primary, font::Weight::Normal),
        Kind::Current | Kind::Dim => unreachable!(),
    };
    highlighter::Format {
        color: Some(color),
        font: Some(Font {
            weight,
            ..*base_font
        }),
    }
}

#[cfg(test)]
mod tests;
