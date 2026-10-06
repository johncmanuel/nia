use crate::app::{App, Message, Tab};
use iced::widget::{container, image, markdown, scrollable, text};
use iced::{Element, Length};
use std::path::{Path, PathBuf};

pub(super) fn reading_view<'a>(state: &'a App, tab: &'a Tab) -> Element<'a, Message> {
    let Some(content) = tab.reading.as_ref() else {
        return text("").into();
    };
    let mut settings =
        markdown::Settings::with_text_size(u32::from(state.editor_font_size), &state.theme);
    settings.style.font = state.editor_font;
    let viewer = Reader {
        document_path: tab.document.path.as_deref(),
    };
    scrollable(
        container(markdown::view_with(content.items(), settings, &viewer))
            .width(Length::Fill)
            .max_width(760),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

struct Reader<'a> {
    document_path: Option<&'a Path>,
}

impl<'a> markdown::Viewer<'a, Message> for Reader<'a> {
    fn on_link_click(url: markdown::Uri) -> Message {
        Message::CopyLink(url)
    }

    fn image(
        &self,
        _settings: markdown::Settings,
        url: &'a markdown::Uri,
        _title: &'a str,
        _alt: &markdown::Text,
    ) -> Element<'a, Message> {
        if let Some(path) = local_image_path(self.document_path, url)
            && path.is_file()
        {
            return image(path)
                .width(Length::Fill)
                .height(Length::Shrink)
                .into();
        }
        let message = if self.document_path.is_none() && !Path::new(url).is_absolute() {
            format!("Save this document to show image: {url}")
        } else {
            format!("Image unavailable: {url}")
        };
        text(message).size(14).into()
    }
}

fn local_image_path(document_path: Option<&Path>, url: &str) -> Option<PathBuf> {
    if url.starts_with("//") || url.contains("://") || url.starts_with("data:") {
        return None;
    }
    let path = url.split(['?', '#']).next()?;
    if path.is_empty() {
        return None;
    }
    let path = decode_url_path(path)?;
    if path.is_absolute() {
        Some(path)
    } else {
        Some(document_path?.parent()?.join(path))
    }
}

fn decode_url_path(value: &str) -> Option<PathBuf> {
    fn hex(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }

    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let byte = hex(*bytes.get(index + 1)?)? * 16 + hex(*bytes.get(index + 2)?)?;
            if byte == 0 {
                return None;
            }
            decoded.push(byte);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests;
