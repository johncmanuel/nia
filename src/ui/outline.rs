use crate::app::{App, Message, OUTLINE};
use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, row, scrollable, space, text, text_input};
use iced::{Element, Length};

pub(super) fn panel(state: &App) -> Element<'_, Message> {
    let headings = state.outline_items();
    let mut items = column![].spacing(2);
    for (index, heading) in headings.iter().enumerate() {
        let label = format!(
            "{}{}",
            "  ".repeat(heading.level.saturating_sub(1)),
            heading.title
        );
        items = items.push(
            button(
                text(label)
                    .size(13)
                    .width(Length::Fill)
                    .wrapping(Wrapping::WordOrGlyph),
            )
            .style(if index == state.outline_index {
                button::secondary
            } else {
                button::text
            })
            .width(Length::Fill)
            .on_press(Message::JumpOutline(index)),
        );
    }
    if headings.is_empty() {
        items = items.push(
            text(if state.outline_query.is_empty() {
                "No headings in this document"
            } else {
                "No matching headings"
            })
            .size(12),
        );
    }
    container(
        column![
            row![
                text("Outline").size(14),
                space::horizontal(),
                button("Hide")
                    .style(button::text)
                    .on_press(Message::HideOutline),
            ]
            .align_y(iced::Alignment::Center),
            text_input("Filter headings", &state.outline_query)
                .id(OUTLINE)
                .on_input(Message::OutlineChanged)
                .on_submit(Message::JumpOutlineSelected),
            scrollable(items).height(Length::Fill),
        ]
        .spacing(10),
    )
    .width(240)
    .height(Length::Fill)
    .padding([12, 14])
    .into()
}
