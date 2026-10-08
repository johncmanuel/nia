use crate::app::{App, Message};
use iced::widget::{button, container, row, text};
use iced::{Background, Element, Length};

pub(super) fn view(state: &App) -> Element<'_, Message> {
    let mut tabs = row![].spacing(2).width(Length::Fill);
    for index in 0..state.groups.len() {
        let active = index == state.active_group;
        let name = state.group_label(index);
        let mut chars = name.chars();
        let short: String = chars.by_ref().take(24).collect();
        let label = if chars.next().is_some() {
            format!("{}  {short}...", index + 1)
        } else {
            format!("{}  {short}", index + 1)
        };
        let Some(tab_index) = state
            .group_active_id(index)
            .and_then(|id| state.tabs.iter().position(|tab| tab.id == id))
        else {
            continue;
        };
        tabs = tabs.push(
            button(text(label).size(11).wrapping(text::Wrapping::None))
                .width(Length::FillPortion(1))
                .padding([5, 8])
                .style(move |theme, status| {
                    let mut style = button::text(theme, status);
                    style.text_color =
                        theme
                            .palette()
                            .text
                            .scale_alpha(if active { 0.85 } else { 0.5 });
                    if active {
                        style.background = Some(Background::Color(
                            theme.extended_palette().background.weak.color,
                        ));
                    }
                    style
                })
                .on_press(Message::SelectTab(tab_index)),
        );
    }
    tabs = tabs.push(
        button(text("+").size(13))
            .padding([4, 8])
            .style(super::footer_button)
            .on_press(Message::NewGroup),
    );
    container(tabs).width(Length::Fill).padding([4, 12]).into()
}
