use crate::app::{Format, Message};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

pub(super) fn menu() -> Element<'static, Message> {
    let mut rows = column![].spacing(4);
    for formats in Format::ALL.chunks(3) {
        let mut buttons = row![].spacing(8);
        for &format in formats {
            buttons = buttons.push(
                button(text(format.label()).size(12))
                    .style(button::text)
                    .width(Length::Fill)
                    .padding([4, 8])
                    .on_press(Message::Format(format)),
            );
        }
        rows = rows.push(buttons);
    }
    container(
        container(scrollable(rows).height(190))
            .width(Length::Fill)
            .max_width(760)
            .padding(12)
            .style(container::bordered_box),
    )
    .width(Length::Fill)
    .center_x(Length::Fill)
    .into()
}
