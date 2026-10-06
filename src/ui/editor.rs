use crate::app::{App, EDITOR, Message, OUTLINE, RECENT, Tab};
use crate::markdown::{self, MarkdownHighlighter};
use crate::ui::reading::reading_view;
use iced::widget::{button, column, container, row, scrollable, text, text_editor, text_input};
use iced::{Background, Border, Element, Length};

pub(super) fn editor_view<'a>(state: &'a App, tab: &'a Tab, active: bool) -> Element<'a, Message> {
    let mut editor = text_editor(&tab.editor)
        .id(if active {
            EDITOR.to_string()
        } else {
            format!("editor-{}", tab.id)
        })
        .highlight_with::<MarkdownHighlighter>(state.editor_font, markdown::format)
        .placeholder("# Start writing")
        .font(state.editor_font)
        .size(u32::from(state.editor_font_size))
        .line_height(text::LineHeight::Relative(1.55))
        .wrapping(text::Wrapping::Word)
        .padding(0)
        .height(Length::Fill)
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
        editor = editor.on_action(move |action| Message::Edit(tab.id, action));
    }

    let writing_area: Element<'_, Message> = if tab.reading.is_some() {
        reading_view(state, tab)
    } else {
        container(editor)
            .width(Length::Fill)
            .max_width(760)
            .height(Length::Fill)
            .into()
    };
    let mut content = column![]
        .spacing(12)
        .width(Length::Fill)
        .max_width(760)
        .height(Length::Fill);
    if tab.conflict {
        content = content.push(
            row![
                text("File changed or is unavailable on disk. Your edits are still here.").size(12),
                button("Save As").on_press(Message::SaveAs),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );
    }
    if active && state.show_outline {
        let headings = state.outline_items();
        let mut results = column![].spacing(2);
        for (index, heading) in headings.iter().enumerate() {
            let label = format!(
                "{} {}",
                "  ".repeat(heading.level.saturating_sub(1)),
                heading.title
            );
            results = results.push(
                button(text(label).size(13))
                    .style(if index == state.outline_index {
                        button::primary
                    } else {
                        button::text
                    })
                    .width(Length::Fill)
                    .on_press(Message::JumpOutline(index)),
            );
        }
        content = content.push(
            column![
                row![
                    text_input("Filter headings", &state.outline_query)
                        .id(OUTLINE)
                        .on_input(Message::OutlineChanged)
                        .on_submit(Message::JumpOutlineSelected)
                        .width(Length::Fill),
                    button("\u{00D7}").on_press(Message::HideOutline),
                ]
                .spacing(8),
                scrollable(results).height(220),
            ]
            .spacing(8),
        );
    }
    if active && state.show_recent {
        let mut results = column![].spacing(2);
        for (index, path) in state.recent_items().iter().enumerate() {
            results = results.push(
                button(text(path.to_string_lossy().to_string()).size(12))
                    .style(if index == state.recent_index {
                        button::primary
                    } else {
                        button::text
                    })
                    .width(Length::Fill)
                    .on_press(Message::OpenRecent((*path).clone())),
            );
        }
        content = content.push(
            column![
                row![
                    text_input("Find recent file", &state.recent_query)
                        .id(RECENT)
                        .on_input(Message::RecentChanged)
                        .on_submit(Message::OpenRecentSelected)
                        .width(Length::Fill),
                    button("\u{00D7}").on_press(Message::HideRecent),
                ]
                .spacing(8),
                scrollable(results).height(220),
            ]
            .spacing(8),
        );
    }
    content = content.push(writing_area);
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .padding(if state.panes.len() > 1 {
            [24, 20]
        } else {
            [48, 32]
        })
        .into()
}
