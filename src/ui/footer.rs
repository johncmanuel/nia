use super::{footer_button, muted_text};
use crate::app::{App, Message};
use iced::widget::{button, container, row, space, text};
use iced::{Element, Length};

pub(super) fn view(state: &App) -> Element<'_, Message> {
    let status = state.notice.as_deref().unwrap_or_else(|| {
        if state.loading {
            "Opening..."
        } else if state.current().saving {
            "Saving..."
        } else if state.current().document.dirty {
            "Unsaved"
        } else if state.current().document.path.is_some() {
            "Saved"
        } else {
            ""
        }
    });
    let reading_label = if state.current().reading.is_some() {
        "Write"
    } else {
        "Read"
    };
    let actions = [
        ("Open", Message::Open),
        ("Save", Message::Save),
        ("Save As", Message::SaveAs),
        (reading_label, Message::ToggleReading),
        ("Find", Message::ShowSearch),
        (
            "Outline",
            if state.show_outline {
                Message::HideOutline
            } else {
                Message::ShowOutline
            },
        ),
        ("Recent", Message::ShowRecent),
        ("Insert", Message::ToggleFormatMenu),
        ("Handbook", Message::ShowHandbook),
        ("Settings", Message::ShowSettings),
    ];
    let controls = actions
        .into_iter()
        .fold(row![], |controls, (label, message)| {
            controls.push(
                button(text(label).size(12))
                    .padding(0)
                    .style(footer_button)
                    .on_press(message),
            )
        });
    let footer = controls
        .push(text(status).size(12).style(muted_text))
        .push(space::horizontal())
        .push(
            text(format!("{} words", state.current().document.word_count))
                .size(12)
                .style(muted_text),
        )
        .spacing(16)
        .align_y(iced::Alignment::Center);
    container(footer)
        .width(Length::Fill)
        .padding([12, 20])
        .into()
}
