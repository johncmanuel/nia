use crate::app::{App, EDITOR, Message, Tab};
use crate::markdown::{self, HighlightSettings, MarkdownHighlighter};
use crate::ui::reading::local_image_path;
use iced::widget::{button, column, container, image, scrollable, text, text_editor};
use iced::{Background, Border, ContentFit, Element, Length};

pub(super) fn writing_view<'a>(state: &'a App, tab: &'a Tab, active: bool) -> Element<'a, Message> {
    let mut blocks = column![].spacing(4).width(Length::Fill);
    for (index, block) in tab.image_blocks.iter().enumerate() {
        if block.url.is_some()
            && index != tab.active_image_block
            && let Some(path) = block
                .url
                .as_deref()
                .and_then(|url| local_image_path(tab.document.path.as_deref(), url))
            && path.is_file()
        {
            blocks = blocks.push(
                button(
                    container(
                        image(path)
                            .width(Length::Fill)
                            .height(Length::Shrink)
                            .content_fit(ContentFit::Contain),
                    )
                    .id(format!("image-preview-{}-{index}", tab.id))
                    .width(Length::Fill)
                    .max_height(560),
                )
                .width(Length::Fill)
                .padding(0)
                .style(button::text)
                .on_press(Message::ActivateImageBlock(tab.id, index)),
            );
            continue;
        }

        let selected = active && index == tab.active_image_block;
        let highlight = HighlightSettings {
            font: state.editor_font,
            active_line: selected.then_some(block.content.cursor().position.line),
            focus: None,
            highlight_line: selected && state.focus.highlight_line,
        };
        let mut editor = text_editor(&block.content)
            .id(if selected {
                EDITOR.to_string()
            } else {
                format!("image-editor-{}-{index}", tab.id)
            })
            .highlight_with::<MarkdownHighlighter>(highlight, markdown::format)
            .font(state.editor_font)
            .size(u32::from(state.editor_font_size))
            .line_height(text::LineHeight::Relative(1.55))
            .wrapping(text::Wrapping::WordOrGlyph)
            .padding(0)
            .height(Length::Shrink)
            .style(|theme, status| {
                let mut style = text_editor::default(theme, status);
                let mut background = theme.palette().background;
                if state.transparent_background {
                    background.a = 0.0;
                }
                style.background = Background::Color(background);
                style.border = Border::default();
                style.placeholder = theme.palette().text.scale_alpha(0.45);
                style.value = theme.palette().text;
                style
            })
            .key_binding(|event| {
                if matches!(event.status, text_editor::Status::Focused { .. })
                    && let Some(message) = state.key_message(&event.key, event.modifiers)
                {
                    return Some(text_editor::Binding::Custom(message));
                }
                text_editor::Binding::from_key_press(event)
            });
        if !state.loading && !state.dialog_open {
            editor = editor.on_action(move |action| Message::EditImageBlock(tab.id, index, action));
        }
        blocks = blocks.push(editor);
    }
    scrollable(blocks)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
