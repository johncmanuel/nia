use super::{Glyph, Style, mark, paint};

pub(super) fn paint_inline(glyphs: &mut [Glyph], hidden: &mut [bool], start: usize, active: bool) {
    let mut index = start;
    while index < glyphs.len() {
        let ch = glyphs[index].ch;
        if ch == '\\' {
            index += 2;
            continue;
        }
        if ch == ':'
            && let Some((end, symbol)) = emoji(glyphs, index)
        {
            if !active {
                glyphs[index].ch = symbol;
                glyphs[index].end = glyphs[end].end;
                hidden[index + 1..=end].fill(true);
            }
            glyphs[index].style = Style::Emoji;
            index = end + 1;
            continue;
        }
        if ch == '!'
            && glyphs.get(index + 1).is_some_and(|glyph| glyph.ch == '[')
            && let Some((label_end, url_end)) = link(glyphs, index + 1)
        {
            glyphs[index].style = Style::Link;
            if !active {
                glyphs[index].ch = '▣';
            }
            mark(glyphs, hidden, index + 1..index + 2, Style::Marker, active);
            paint(glyphs, index + 2..label_end, Style::Link);
            mark(
                glyphs,
                hidden,
                label_end..url_end + 1,
                Style::Marker,
                active,
            );
            index = url_end + 1;
            continue;
        }
        if ch == '[' {
            if glyphs.get(index + 1).is_some_and(|glyph| glyph.ch == '^')
                && let Some(end) = find(glyphs, index + 2, "]")
            {
                mark(glyphs, hidden, index..index + 2, Style::Marker, active);
                paint(glyphs, index + 2..end, Style::Footnote);
                mark(glyphs, hidden, end..end + 1, Style::Marker, active);
                index = end + 1;
                continue;
            }
            if let Some((label_end, url_end)) = link(glyphs, index) {
                mark(glyphs, hidden, index..index + 1, Style::Marker, active);
                paint(glyphs, index + 1..label_end, Style::Link);
                mark(
                    glyphs,
                    hidden,
                    label_end..url_end + 1,
                    Style::Marker,
                    active,
                );
                index = url_end + 1;
                continue;
            }
        }
        let matched = if ch == '`' {
            Some(("`", Style::Code))
        } else if starts(glyphs, index, "**") || starts(glyphs, index, "__") {
            Some((if ch == '*' { "**" } else { "__" }, Style::Strong))
        } else if starts(glyphs, index, "~~") {
            Some(("~~", Style::Strike))
        } else if starts(glyphs, index, "==") {
            Some(("==", Style::Highlight))
        } else {
            match ch {
                '*' | '_' => Some((if ch == '*' { "*" } else { "_" }, Style::Emphasis)),
                '~' => Some(("~", Style::Subscript)),
                '^' => Some(("^", Style::Superscript)),
                _ => None,
            }
        };
        if let Some((marker, style)) = matched {
            let len = marker.chars().count();
            if let Some(end) = find(glyphs, index + len + 1, marker) {
                mark(glyphs, hidden, index..index + len, Style::Marker, active);
                paint(glyphs, index + len..end, style);
                mark(glyphs, hidden, end..end + len, Style::Marker, active);
                index = end + len;
                continue;
            }
        }
        index += 1;
    }
}

fn starts(glyphs: &[Glyph], at: usize, pattern: &str) -> bool {
    glyphs[at..]
        .iter()
        .map(|glyph| glyph.ch)
        .take(pattern.len())
        .eq(pattern.chars())
}

fn find(glyphs: &[Glyph], start: usize, pattern: &str) -> Option<usize> {
    (start..glyphs.len()).find(|at| starts(glyphs, *at, pattern))
}

fn link(glyphs: &[Glyph], open: usize) -> Option<(usize, usize)> {
    let label_end = find(glyphs, open + 2, "]")?;
    if !starts(glyphs, label_end + 1, "(") {
        return None;
    }
    let url_end = find(glyphs, label_end + 2, ")")?;
    Some((label_end, url_end))
}

fn emoji(glyphs: &[Glyph], start: usize) -> Option<(usize, char)> {
    let end = find(glyphs, start + 2, ":")?;
    let name: String = glyphs[start + 1..end]
        .iter()
        .map(|glyph| glyph.ch)
        .collect();
    let symbol = match name.as_str() {
        "joy" => '😂',
        "smile" => '😄',
        "heart" => '❤',
        "check" => '✅',
        _ => return None,
    };
    Some((end, symbol))
}
