// Throwaway prototype: source bytes remain authoritative; visible glyphs point back to them.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Style {
    Plain,
    Heading,
    Emphasis,
    Strong,
    Marker,
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
    pub(super) heading: bool,
    pub(super) glyphs: Vec<Glyph>,
}

pub(super) fn project(source: &str, active_line: usize) -> Vec<Line> {
    let mut offset = 0;
    source
        .split('\n')
        .enumerate()
        .map(|(line_index, text)| {
            let chars: Vec<_> = text.char_indices().collect();
            let mut styles = vec![Style::Plain; chars.len()];
            let mut hidden = vec![false; chars.len()];
            let heading_prefix = text.bytes().take_while(|byte| *byte == b'#').count();
            let heading = (1..=6).contains(&heading_prefix)
                && text.as_bytes().get(heading_prefix) == Some(&b' ');
            if heading {
                styles.fill(Style::Heading);
                if line_index != active_line {
                    hidden[..heading_prefix + 1].fill(true);
                } else {
                    styles[..heading_prefix + 1].fill(Style::Marker);
                }
            }

            // This spike handles simple paired * and ** spans, with no nesting.
            let mut index = heading_prefix + usize::from(heading);
            while index < chars.len() {
                if chars[index].1 != '*' {
                    index += 1;
                    continue;
                }
                let marker_len = if chars.get(index + 1).is_some_and(|ch| ch.1 == '*') {
                    2
                } else {
                    1
                };
                let closing = (index + marker_len + 1..chars.len()).find(|&at| {
                    chars[at].1 == '*'
                        && (marker_len == 1 || chars.get(at + 1).is_some_and(|ch| ch.1 == '*'))
                });
                if let Some(end) = closing {
                    let style = if marker_len == 2 {
                        Style::Strong
                    } else {
                        Style::Emphasis
                    };
                    styles[index + marker_len..end].fill(style);
                    for marker in index..index + marker_len {
                        hidden[marker] = line_index != active_line;
                        styles[marker] = Style::Marker;
                    }
                    for marker in end..end + marker_len {
                        hidden[marker] = line_index != active_line;
                        styles[marker] = Style::Marker;
                    }
                    index = end + marker_len;
                } else {
                    index += marker_len;
                }
            }

            let glyphs = chars
                .iter()
                .enumerate()
                .filter(|(i, _)| !hidden[*i])
                .map(|(i, &(byte, ch))| Glyph {
                    ch,
                    start: offset + byte,
                    end: offset + chars.get(i + 1).map_or(text.len(), |next| next.0),
                    style: styles[i],
                })
                .collect();
            let line = Line {
                start: offset,
                end: offset + text.len(),
                heading,
                glyphs,
            };
            offset += text.len() + 1;
            line
        })
        .collect()
}

#[cfg(test)]
#[path = "projection/tests.rs"]
mod tests;
