// Throwaway prototype: source bytes remain authoritative; visible glyphs point back to them.
#[path = "projection/inline.rs"]
mod inline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Style {
    Plain,
    Heading,
    Emphasis,
    Strong,
    Marker,
    Quote,
    Code,
    Link,
    Strike,
    Highlight,
    Subscript,
    Superscript,
    Footnote,
    Border,
    Emoji,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LineKind {
    Plain,
    Heading,
    Quote,
    List,
    Task,
    Code,
    Fence,
    Rule,
    Table,
    TableRule,
    Definition,
    Footnote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Glyph {
    pub(super) ch: char,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) style: Style,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Line {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) kind: LineKind,
    pub(super) glyphs: Vec<Glyph>,
}

pub(super) fn project(source: &str, active_line: usize) -> Vec<Line> {
    let mut offset = 0;
    let mut fenced = false;
    source
        .split('\n')
        .enumerate()
        .map(|(line_index, text)| {
            let mut glyphs: Vec<_> = text
                .char_indices()
                .map(|(byte, ch)| Glyph {
                    ch,
                    start: offset + byte,
                    end: offset + byte + ch.len_utf8(),
                    style: Style::Plain,
                })
                .collect();
            let mut hidden = vec![false; glyphs.len()];
            let active = line_index == active_line;
            let (kind, inline_start) = block(text, &mut glyphs, &mut hidden, active, &mut fenced);
            if !matches!(
                kind,
                LineKind::Code | LineKind::Fence | LineKind::Rule | LineKind::TableRule
            ) {
                inline::paint_inline(&mut glyphs, &mut hidden, inline_start, active);
            }
            let line = Line {
                start: offset,
                end: offset + text.len(),
                kind,
                glyphs: glyphs
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, glyph)| (!hidden[index]).then_some(glyph))
                    .collect(),
            };
            offset += text.len() + 1;
            line
        })
        .collect()
}

fn block(
    text: &str,
    glyphs: &mut [Glyph],
    hidden: &mut [bool],
    active: bool,
    fenced: &mut bool,
) -> (LineKind, usize) {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    let start = text[..indent].chars().count();
    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        *fenced = !*fenced;
        mark(glyphs, hidden, start..start + 3, Style::Marker, active);
        paint(glyphs, start + 3..glyphs.len(), Style::Code);
        return (LineKind::Fence, glyphs.len());
    }
    if *fenced {
        paint(glyphs, 0..glyphs.len(), Style::Code);
        return (LineKind::Code, glyphs.len());
    }
    if table_rule(trimmed) {
        mark(glyphs, hidden, 0..glyphs.len(), Style::Border, active);
        return (LineKind::TableRule, glyphs.len());
    }
    if horizontal_rule(trimmed) {
        mark(glyphs, hidden, 0..glyphs.len(), Style::Marker, active);
        return (LineKind::Rule, glyphs.len());
    }
    let heading_prefix = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    if (1..=6).contains(&heading_prefix) && trimmed.as_bytes().get(heading_prefix) == Some(&b' ') {
        paint(glyphs, start..glyphs.len(), Style::Heading);
        mark(
            glyphs,
            hidden,
            start..start + heading_prefix + 1,
            Style::Marker,
            active,
        );
        return (LineKind::Heading, start + heading_prefix + 1);
    }
    if trimmed.starts_with("> ") {
        paint(glyphs, start..glyphs.len(), Style::Quote);
        mark(glyphs, hidden, start..start + 2, Style::Marker, active);
        return (LineKind::Quote, start + 2);
    }
    if trimmed.starts_with("- [x] ")
        || trimmed.starts_with("- [X] ")
        || trimmed.starts_with("- [ ] ")
    {
        paint(glyphs, start..start + 6, Style::Marker);
        if !active {
            glyphs[start + 2].ch = if trimmed.as_bytes()[3] == b' ' {
                '☐'
            } else {
                '☑'
            };
            glyphs[start + 2].end = glyphs[start + 4].end;
            hidden[start] = true;
            hidden[start + 1] = true;
            hidden[start + 3..start + 5].fill(true);
        }
        return (LineKind::Task, start + 6);
    }
    if trimmed.starts_with(['-', '*', '+']) && trimmed.as_bytes().get(1) == Some(&b' ') {
        glyphs[start].style = Style::Marker;
        glyphs[start + 1].style = Style::Marker;
        if !active {
            glyphs[start].ch = '•';
        }
        return (LineKind::List, start + 2);
    }
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if (1..=9).contains(&digits) && trimmed.as_bytes().get(digits..digits + 2) == Some(b". ") {
        paint(glyphs, start..start + digits + 1, Style::Marker);
        glyphs[start + digits + 1].style = Style::Marker;
        return (LineKind::List, start + digits + 2);
    }
    if trimmed.starts_with("[^")
        && let Some(end_byte) = trimmed.find("]:")
    {
        let end = trimmed[..end_byte].chars().count();
        paint(glyphs, start..start + end + 2, Style::Footnote);
        mark(glyphs, hidden, start..start + 2, Style::Marker, active);
        mark(
            glyphs,
            hidden,
            start + end..start + end + 1,
            Style::Marker,
            active,
        );
        return (LineKind::Footnote, start + end + 2);
    }
    if trimmed.starts_with(": ") {
        glyphs[start].style = Style::Marker;
        if !active {
            glyphs[start].ch = '→';
        }
        glyphs[start + 1].style = Style::Marker;
        return (LineKind::Definition, start + 2);
    }
    if trimmed.matches('|').count() >= 2 {
        for glyph in glyphs.iter_mut().filter(|glyph| glyph.ch == '|') {
            glyph.style = Style::Border;
            if !active {
                glyph.ch = '│';
            }
        }
        return (LineKind::Table, start);
    }
    (LineKind::Plain, start)
}

fn paint(glyphs: &mut [Glyph], range: std::ops::Range<usize>, style: Style) {
    for glyph in &mut glyphs[range] {
        glyph.style = style;
    }
}

fn mark(
    glyphs: &mut [Glyph],
    hidden: &mut [bool],
    range: std::ops::Range<usize>,
    style: Style,
    active: bool,
) {
    paint(glyphs, range.clone(), style);
    if !active {
        hidden[range].fill(true);
    }
}

fn horizontal_rule(text: &str) -> bool {
    let compact: Vec<_> = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    compact.len() >= 3
        && matches!(compact[0], '-' | '*' | '_')
        && compact.iter().all(|ch| *ch == compact[0])
}

fn table_rule(text: &str) -> bool {
    text.contains('|')
        && text.trim_matches('|').split('|').all(|cell| {
            let cell = cell.trim();
            cell.chars().filter(|ch| *ch == '-').count() >= 3
                && cell.chars().all(|ch| matches!(ch, '-' | ':'))
        })
}

#[cfg(test)]
#[path = "projection/tests.rs"]
mod tests;
