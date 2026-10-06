use super::{footer_button, muted_text, theme};
use crate::app::{App, Message};
use crate::keybindings::Slot;
use iced::theme::Base;
use iced::widget::{button, checkbox, column, combo_box, container, row, scrollable, space, text};
use iced::{Element, Length};

pub(super) fn settings_page(state: &App) -> Element<'_, Message> {
    let mut rows = column![
        text("Settings").size(26),
        text("Appearance").size(18),
        text("Theme").size(15),
        button(text(state.theme.name()).size(14))
            .style(button::secondary)
            .width(220)
            .on_press(Message::ToggleThemePicker),
    ]
    .spacing(16)
    .width(Length::Fill);
    if state.show_theme_picker {
        rows = rows.push(theme::menu(&state.theme));
    }
    rows = rows
        .push(
            checkbox(state.transparent_background)
                .label("Transparent background")
                .on_toggle(Message::SetTransparentBackground)
                .text_size(14),
        )
        .push(
            text("Show the desktop through the writing window (macOS and Linux).")
                .size(12)
                .style(muted_text),
        )
        .push(text("Font").size(18))
        .push(
            row![
                combo_box(
                    &state.font_options,
                    "Choose installed font",
                    state.font_family.as_ref(),
                    Message::SetFont
                )
                .width(220)
                .size(13),
                button(text("Monospace").size(13))
                    .style(button::text)
                    .on_press(Message::ResetFont),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .push(
            row![
                text(format!("Size: {} pt", state.editor_font_size)).size(14),
                button("Smaller")
                    .style(button::secondary)
                    .on_press(Message::DecreaseFontSize),
                button("Larger")
                    .style(button::secondary)
                    .on_press(Message::IncreaseFontSize),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .push(text("Keybindings").size(18))
        .push(
        text("Click a shortcut, then press Command or Control with a letter, number, Tab, plus, minus, or comma. Escape cancels.")
            .size(13)
            .style(muted_text),
        );

    for slot in Slot::ALL
        .into_iter()
        .filter(|slot| !matches!(slot, Slot::AlternateSave | Slot::AlternateFind))
    {
        if slot == Slot::NewTab {
            rows = rows.push(text("Tabs").size(18));
        } else if slot == Slot::SinglePane {
            rows = rows.push(text("Layout").size(18));
        } else if slot == Slot::IncreaseFontSize {
            rows = rows.push(text("Editor").size(18));
        } else if slot == Slot::Settings {
            rows = rows.push(text("App").size(18));
        }
        let shortcut = if state.recording == Some(slot) {
            "Press keys...".into()
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
            "Cmd+1-9 selects a tab (9 selects the last tab)."
        } else {
            "Ctrl+1-9 selects a tab (9 selects the last tab)."
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

    let centered = container(container(rows).width(Length::Fill).max_width(620))
        .width(Length::Fill)
        .center_x(Length::Fill);
    container(scrollable(centered))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .padding([64, 32])
        .into()
}
