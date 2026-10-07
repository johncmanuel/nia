use crate::app::{App, Message};
use iced::widget::{button, column, row, rule, text};
use iced::{Element, Length, Theme};

mod confirm;
mod editor;
mod footer;
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
    if state.zen_mode {
        return editor::editor_view(state, state.current(), true);
    }
    let mut page = column![navigation::header(state)];
    if let Some(results) = navigation::search_results(state) {
        page = page.push(results);
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
    page.push(content)
        .push(footer::view(state))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
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
