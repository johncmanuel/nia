use crate::app::{App, CloseTarget, Message};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

pub(super) fn close_confirmation(state: &App) -> Element<'_, Message> {
    let (title, warning, save_label, discard_label) = match state.pending_close {
        Some(CloseTarget::Tab(id)) => {
            let name = state
                .tabs
                .iter()
                .find(|tab| tab.id == id)
                .map_or_else(|| "this document".into(), |tab| tab.document.file_name());
            (
                format!("Close {name}?"),
                "If you close without saving, your changes will be deleted.",
                "Save and Close",
                "Discard and Close",
            )
        }
        Some(CloseTarget::Group(_)) => (
            "Close this tab group?".into(),
            "If you close without saving, unsaved changes in this group will be deleted.",
            "Save and Close",
            "Discard and Close",
        ),
        Some(CloseTarget::Window(_)) => (
            "Quit nia?".into(),
            "If you quit without saving, your unsaved changes will be deleted.",
            "Save and Quit",
            "Discard and Quit",
        ),
        None => unreachable!("confirmation only renders for a pending close"),
    };
    let buttons = row![
        button("Cancel").on_press(Message::CancelClose),
        button(save_label).on_press(Message::SaveAndClose),
        button(discard_label).on_press(Message::DiscardAndClose),
    ]
    .spacing(12);
    let mut content = column![text(title).size(24), text(warning), buttons]
        .spacing(18)
        .align_x(Alignment::Center);
    if let Some(notice) = &state.notice {
        content = content.push(text(notice));
    }
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
