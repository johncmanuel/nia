use super::{Fence, atx_heading, closes_fence, fenced_start, setext_level};
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
}

pub(crate) struct MarkdownHighlighter {
    line: usize,
    fence: Option<Fence>,
    fences_after: Vec<Option<Fence>>,
}

impl text::Highlighter for MarkdownHighlighter {
    type Settings = ();
    type Highlight = Kind;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Kind)>;

    fn new(_: &()) -> Self {
        Self {
            line: 0,
            fence: None,
            fences_after: Vec::new(),
        }
    }
    fn update(&mut self, _: &()) {}
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
        self.fences_after.push(self.fence);
        ranges.into_iter()
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

pub(crate) fn format(kind: &Kind, theme: &Theme) -> highlighter::Format<Font> {
    let palette = theme.palette();
    let (color, weight) = match kind {
        Kind::Heading => (palette.primary, font::Weight::Bold),
        Kind::Marker | Kind::Quote => (palette.text.scale_alpha(0.58), font::Weight::Normal),
        Kind::Code => (palette.success, font::Weight::Normal),
        Kind::Emphasis => (palette.text, font::Weight::Medium),
        Kind::Strong => (palette.text, font::Weight::Bold),
        Kind::Link => (palette.primary, font::Weight::Normal),
    };
    highlighter::Format {
        color: Some(color),
        font: Some(Font {
            weight,
            ..Font::MONOSPACE
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::text::Highlighter;

    #[test]
    fn highlights_markdown_without_treating_fenced_headings_as_headings() {
        let mut highlighter = MarkdownHighlighter::new(&());
        assert_eq!(
            highlighter.highlight_line("# Title").collect::<Vec<_>>(),
            vec![(0..7, Kind::Heading)]
        );
        assert_eq!(
            highlighter.highlight_line("```md").collect::<Vec<_>>(),
            vec![(0..5, Kind::Code)]
        );
        assert_eq!(
            highlighter.highlight_line("# source").collect::<Vec<_>>(),
            vec![(0..8, Kind::Code)]
        );
        highlighter.change_line(1);
        assert_eq!(
            highlighter
                .highlight_line("plain *word* [link](url)")
                .collect::<Vec<_>>(),
            vec![(6..12, Kind::Emphasis), (13..24, Kind::Link)]
        );
    }
}
