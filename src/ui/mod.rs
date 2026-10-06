use crate::app::{App, Message, SEARCH};
use crate::theme::WritingTheme;
use iced::widget::{
    button, checkbox, column, container, pane_grid, pick_list, row, scrollable, space, text,
    text_input,
};
use iced::{Element, Length, Theme};

mod confirm;
mod editor;
mod keybindings;

use confirm::close_confirmation;
use editor::editor_view;
use keybindings::keybindings_page;

pub(crate) fn view(state: &App) -> Element<'_, Message> {
    if state.pending_close.is_some() {
        return close_confirmation(state);
    }
    let writing_area: Element<'_, Message> = if state.show_keybindings {
        keybindings_page(state)
    } else {
        pane_grid::PaneGrid::new(&state.panes, |pane, tab_id, _| {
            let tab = state
                .tabs
                .iter()
                .find(|tab| tab.id == *tab_id)
                .unwrap_or_else(|| state.current());
            let active = pane == state.active_pane;
            let title = row![
                text(if active { "●" } else { "○" }).size(12),
                text(tab.document.file_name()).size(13),
                space::horizontal(),
                button("×")
                    .style(button::text)
                    .on_press(Message::ClosePane(pane)),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center);
            let content = pane_grid::Content::new(editor_view(state, tab, active));
            if state.panes.len() > 1 {
                content.title_bar(pane_grid::TitleBar::new(title).padding([8, 12]))
            } else {
                content
            }
        })
        .spacing(8)
        .on_click(Message::ActivatePane)
        .on_drag(Message::PaneDragged)
        .on_resize(8, Message::PaneResized)
        .into()
    };

    let status = state.notice.as_deref().unwrap_or_else(|| {
        if state.loading {
            "Opening…"
        } else if state.current().saving {
            "Saving…"
        } else if state.current().document.dirty {
            "Unsaved"
        } else if state.current().document.path.is_some() {
            "Saved"
        } else {
            ""
        }
    });
    let footer = row![
        button(text("New").size(12))
            .padding(0)
            .style(footer_button)
            .on_press(Message::New),
        button(text("Open").size(12))
            .padding(0)
            .style(footer_button)
            .on_press(Message::Open),
        button(text("Save").size(12))
            .padding(0)
            .style(footer_button)
            .on_press(Message::Save),
        button(text("Save As").size(12))
            .padding(0)
            .style(footer_button)
            .on_press(Message::SaveAs),
        button(text("Find").size(12))
            .padding(0)
            .style(footer_button)
            .on_press(Message::ShowSearch),
        button(text("Outline").size(12))
            .padding(0)
            .style(footer_button)
            .on_press(Message::ShowOutline),
        button(text("Recent").size(12))
            .padding(0)
            .style(footer_button)
            .on_press(Message::ShowRecent),
        button(
            text(if state.show_keybindings {
                "Write"
            } else {
                "Keybindings"
            })
            .size(12)
        )
        .padding(0)
        .style(footer_button)
        .on_press(if state.show_keybindings {
            Message::ShowEditor
        } else {
            Message::ShowKeybindings
        }),
        text(status).size(12).style(muted_text),
        space::horizontal(),
        pick_list(WritingTheme::ALL, Some(state.theme), Message::SetTheme)
            .text_size(12)
            .width(140),
        text(format!("{} words", state.current().document.word_count))
            .size(12)
            .style(muted_text),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center);

    let tabs = state
        .tabs
        .iter()
        .enumerate()
        .fold(row![].spacing(8), |tabs, (index, tab)| {
            let name = format!(
                "{}{}",
                tab.document.file_name(),
                if tab.document.dirty { " •" } else { "" }
            );
            tabs.push(
                button(text(name).size(12))
                    .style(if index == state.active {
                        button::secondary
                    } else {
                        button::text
                    })
                    .on_press(Message::SelectTab(index)),
            )
            .push(
                button(text("×").size(13))
                    .style(button::text)
                    .on_press(Message::CloseTab(index)),
            )
        });
    let tabs = scrollable(tabs)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ))
        .width(Length::Fill)
        .height(36);
    let mut header = row![tabs].spacing(8).align_y(iced::Alignment::Center);
    if state.show_search {
        let count = state.search_matches().len();
        let position = state.search_index.map_or(0, |index| index + 1);
        header = header.push(
            row![
                text_input("Find in document", &state.search_query)
                    .id(SEARCH)
                    .on_input(Message::SearchChanged)
                    .on_submit(Message::FindNext)
                    .width(200),
                text(format!("{position}/{count}"))
                    .size(12)
                    .style(muted_text),
                checkbox(state.search_case_sensitive)
                    .label("Case")
                    .on_toggle(Message::SearchCaseChanged)
                    .text_size(12),
                button("↑")
                    .style(button::text)
                    .on_press(Message::FindPrevious),
                button("↓").style(button::text).on_press(Message::FindNext),
                button("×")
                    .style(button::text)
                    .on_press(Message::HideSearch),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        );
    }
    if state.show_layout {
        header = header.push(
            row![
                button(text("Single").size(12))
                    .style(button::text)
                    .on_press(Message::SinglePane),
                button(text("Split right").size(12))
                    .style(button::text)
                    .on_press(Message::SplitPane(pane_grid::Axis::Vertical)),
                button(text("Split down").size(12))
                    .style(button::text)
                    .on_press(Message::SplitPane(pane_grid::Axis::Horizontal)),
            ]
            .spacing(4),
        );
    }
    header = header.push(
        button(text("Layout").size(12))
            .style(button::text)
            .on_press(Message::ToggleLayout),
    );
    column![
        container(header).width(Length::Fill).padding([8, 20]),
        writing_area,
        container(footer).width(Length::Fill).padding([12, 20]),
    ]
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
