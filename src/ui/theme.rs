use crate::app::Message;
use crate::theme;
use iced::theme::Base;
use iced::widget::{button, column, container, rule, scrollable, text};
use iced::{Element, Length, Theme};

pub(super) fn menu(selected: &Theme) -> Element<'static, Message> {
    let mut items = column![].spacing(4).padding(8);
    for (label, dark) in [("Light", false), ("Dark", true)] {
        items = items.push(text(label).size(12)).push(rule::horizontal(1));
        for choice in theme::choices(dark) {
            items = items.push(
                button(text(choice.name()).size(12))
                    .style(if choice == selected {
                        button::secondary
                    } else {
                        button::text
                    })
                    .padding([4, 8])
                    .width(Length::Fill)
                    .on_press(Message::SetTheme(choice.clone())),
            );
        }
    }
    container(scrollable(items).height(480).width(220))
        .style(container::bordered_box)
        .into()
}
