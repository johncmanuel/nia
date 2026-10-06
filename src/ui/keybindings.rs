use super::{footer_button, muted_text};
use crate::app::{App, Message};
use crate::keybindings::Slot;
use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Element, Length};

pub(super) fn keybindings_page(state: &App) -> Element<'_, Message> {
    let mut rows = column![
        text("Keybindings").size(26),
        text("Click a shortcut, then press Command or Control with a letter, number, or Tab. Escape cancels.")
            .size(13)
            .style(muted_text),
    ]
    .spacing(16)
    .width(Length::Fill);

    for slot in Slot::ALL {
        if slot == Slot::NewTab {
            rows = rows.push(text("Tabs").size(18));
        } else if slot == Slot::SinglePane {
            rows = rows.push(text("Layout").size(18));
        }
        let shortcut = if state.recording == Some(slot) {
            "Press keys…".into()
        } else {
            state
                .keybindings
                .get(slot)
                .map_or_else(|| "Unassigned".into(), |shortcut| shortcut.label())
        };
        rows = rows.push(
            row![
                text(slot.label()).size(15),
                space::horizontal(),
                button(text(shortcut).size(14))
                    .padding([8, 12])
                    .style(button::secondary)
                    .on_press(Message::RecordShortcut(slot)),
            ]
            .width(Length::Fill)
            .align_y(iced::Alignment::Center),
        );
    }

    rows = rows.push(
        text(if cfg!(target_os = "macos") {
            "Cmd+1–9 selects a tab (9 selects the last tab)."
        } else {
            "Ctrl+1–9 selects a tab (9 selects the last tab)."
        })
        .size(12)
        .style(muted_text),
    );
    rows = rows.push(
        button(text("Restore defaults").size(13))
            .padding(0)
            .style(footer_button)
            .on_press(Message::ResetKeybindings),
    );

    container(scrollable(
        container(rows).width(Length::Fill).max_width(620),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .padding([64, 32])
    .into()
}
