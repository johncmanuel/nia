use crate::app::{App, Message};
use iced::widget::{button, column, container, markdown, row, scrollable, space, text};
use iced::{Alignment, Element, Length};

pub(super) fn page(state: &App) -> Element<'_, Message> {
    let Some(content) = state.handbook.as_ref() else {
        return text("").into();
    };
    let mut settings = markdown::Settings::with_text_size(16, &state.theme);
    settings.style.font = state.editor_font;
    let header = row![
        text("Handbook").size(26),
        space::horizontal(),
        button("Back to writing")
            .style(button::secondary)
            .on_press(Message::HideHandbook),
    ]
    .align_y(Alignment::Center);
    let body = scrollable(
        container(markdown::view(content.items(), settings).map(Message::CopyLink))
            .width(Length::Fill),
    )
    .spacing(12)
    .height(Length::Fill);
    container(
        container(column![header, body].spacing(24).height(Length::Fill))
            .width(Length::Fill)
            .max_width(760),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .padding([40, 32])
    .into()
}
