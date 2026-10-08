use crate::app::{App, Message};
use iced::widget::{Stack, button, column, container, row, rule, text};
use iced::{Element, Length, Theme};

mod confirm;
mod editor;
mod footer;
mod formatting;
mod groups;
mod handbook;
mod images;
mod navigation;
mod outline;
mod reading;
mod settings;
mod theme;

pub(crate) fn view(state: &App) -> Element<'_, Message> {
    if state.pending_close.is_some() {
        return confirm::close_confirmation(state);
    }
    if state.show_settings {
        return settings::settings_page(state);
    }
    if state.handbook.is_some() {
        return handbook::page(state);
    }
    let content: Element<'_, Message> = if state.show_outline {
        row![
            outline::panel(state),
            rule::vertical(1),
            editor::panes_view(state)
        ]
        .height(Length::Fill)
        .into()
    } else {
        editor::panes_view(state)
    };
    let mut page = column![content];
    if !state.zen_mode {
        if state.groups.len() > 1 {
            page = column![groups::view(state), page].height(Length::Fill);
        }
        if state.show_format_menu {
            page = page.push(formatting::menu());
        }
        page = page.push(footer::view(state));
    } else if let Some(notice) = &state.notice {
        page = page.push(
            container(text(notice).size(12))
                .width(Length::Fill)
                .padding([8, 20]),
        );
    }
    let page: Element<'_, Message> = page.width(Length::Fill).height(Length::Fill).into();
    if let Some(search) = navigation::search_bar(state) {
        let mut panel = column![search];
        if let Some(results) = navigation::search_results(state) {
            panel = panel.push(results);
        }
        Stack::new()
            .push(page)
            .push(container(panel).width(Length::Fill).style(|theme: &Theme| {
                container::Style::default().background(theme.palette().background)
            }))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        page
    }
}

fn muted_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().text.scale_alpha(0.55)),
    }
}

fn footer_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(theme, status);
    style.text_color = theme.palette().text.scale_alpha(match status {
        button::Status::Hovered | button::Status::Pressed => 0.9,
        _ => 0.55,
    });
    style
}
