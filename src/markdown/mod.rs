mod highlight;

pub(crate) use highlight::{HighlightSettings, MarkdownHighlighter, format};

pub(crate) fn standalone_images(contents: &str) -> Vec<(usize, String)> {
    let mut images = Vec::new();
    let mut fence = None;
    for (line_number, line) in contents.lines().enumerate() {
        if let Some(active) = fence {
            if closes_fence(line, active) {
                fence = None;
            }
            continue;
        }
        if let Some(start) = fenced_start(line) {
            fence = Some(start);
            continue;
        }
        if !line.contains("![") {
            continue;
        }
        let mut items = iced::widget::markdown::parse(line.trim());
        if let (Some(iced::widget::markdown::Item::Image { url, .. }), None) =
            (items.next(), items.next())
        {
            images.push((line_number, url.to_string()));
        }
    }
    images
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Heading {
    pub(crate) level: usize,
    pub(crate) line: usize,
    pub(crate) title: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Fence {
    marker: u8,
    count: usize,
}

fn fenced_start(line: &str) -> Option<Fence> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let marker = *trimmed.as_bytes().first()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let count = trimmed.bytes().take_while(|byte| *byte == marker).count();
    (count >= 3).then_some(Fence { marker, count })
}

fn closes_fence(line: &str, fence: Fence) -> bool {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return false;
    }
    let count = trimmed
        .bytes()
        .take_while(|byte| *byte == fence.marker)
        .count();
    count >= fence.count && trimmed[count..].trim().is_empty()
}

fn atx_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let level = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&level)
        || !trimmed[level..].starts_with([' ', '\t']) && trimmed.len() != level
    {
        return None;
    }
    let raw = trimmed[level..].trim();
    let without_closing = raw.trim_end_matches('#');
    let title = if without_closing.ends_with([' ', '\t']) {
        without_closing.trim_end()
    } else {
        raw
    };
    Some((level, title.into()))
}

fn setext_level(line: &str) -> Option<usize> {
    let trimmed = line.trim();
    if trimmed.len() < 2 {
        return None;
    }
    if trimmed.bytes().all(|byte| byte == b'=') {
        Some(1)
    } else if trimmed.bytes().all(|byte| byte == b'-') {
        Some(2)
    } else {
        None
    }
}

pub(crate) fn headings(contents: &str) -> Vec<Heading> {
    let mut result = Vec::new();
    let mut fence = None;
    let mut previous: Option<(usize, &str)> = None;
    for (line_number, line) in contents.lines().enumerate() {
        if let Some(active) = fence {
            if closes_fence(line, active) {
                fence = None;
            }
            previous = None;
            continue;
        }
        if let Some(start) = fenced_start(line) {
            fence = Some(start);
            previous = None;
            continue;
        }
        if let Some((level, title)) = atx_heading(line) {
            result.push(Heading {
                level,
                line: line_number,
                title,
            });
            previous = None;
        } else if let Some(level) = setext_level(line) {
            if let Some((title_line, title)) = previous.take() {
                result.push(Heading {
                    level,
                    line: title_line,
                    title: title.trim().into(),
                });
            }
        } else {
            previous = (!line.trim().is_empty()
                && !line.trim_start().starts_with(['>', '-', '*', '+']))
            .then_some((line_number, line));
        }
    }
    result
}

#[cfg(test)]
mod tests;
