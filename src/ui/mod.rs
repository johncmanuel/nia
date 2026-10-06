use crate::app::{App, Message, SEARCH};
use crate::theme::WritingTheme;
use iced::widget::{
    button, checkbox, column, combo_box, container, pane_grid, pick_list, row, scrollable, space,
    text, text_input,
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
                text(if active { "\u{25CF}" } else { "\u{25CB}" }).size(12),
                text(tab.document.file_name()).size(13),
                space::horizontal(),
                button("\u{00D7}")
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
        button(text("Font").size(12))
            .padding(0)
            .style(footer_button)
            .on_press(Message::ToggleFontPicker),
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
                if tab.document.dirty { " \u{2022}" } else { "" }
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
                button(text("\u{00D7}").size(13))
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
        let count = if state.search_all_tabs {
            state.tab_search_hits.len()
        } else {
            state.search_matches().len()
        };
        let position = state.search_index.map_or(0, |index| index + 1);
        header = header.push(
            row![
                text_input(
                    if state.search_all_tabs {
                        "Search open tabs"
                    } else {
                        "Find in document"
                    },
                    &state.search_query
                )
                .id(SEARCH)
                .on_input(Message::SearchChanged)
                .on_submit(Message::FindNext)
                .width(200),
                text(format!("{position}/{count}"))
                    .size(12)
                    .style(muted_text),
                checkbox(state.search_all_tabs)
                    .label("All tabs")
                    .on_toggle(Message::SearchAllTabs)
                    .text_size(12),
                checkbox(state.search_case_sensitive)
                    .label("Case")
                    .on_toggle(Message::SearchCaseChanged)
                    .text_size(12),
                button("\u{2191}")
                    .style(button::text)
                    .on_press(Message::FindPrevious),
                button("\u{2193}")
                    .style(button::text)
                    .on_press(Message::FindNext),
                button("\u{00D7}")
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
    if state.show_font_picker {
        header = header.push(
            row![
                combo_box(
                    &state.font_options,
                    "Choose installed font",
                    state.font_family.as_ref(),
                    Message::SetFont
                )
                .width(220)
                .size(12),
                button(text("Monospace").size(12))
                    .style(button::text)
                    .on_press(Message::ResetFont),
            ]
            .spacing(6),
        );
    }
    header = header.push(
        button(text("Layout").size(12))
            .style(button::text)
            .on_press(Message::ToggleLayout),
    );
    let mut page = column![container(header).width(Length::Fill).padding([8, 20])];
    if state.show_search && state.search_all_tabs && !state.search_query.is_empty() {
        let mut results = column![].spacing(2);
        let first = state.search_index.unwrap_or(0) / 100 * 100;
        for (index, hit) in state
            .tab_search_hits
            .iter()
            .enumerate()
            .skip(first)
            .take(100)
        {
            if let Some((tab_index, tab)) = state
                .tabs
                .iter()
                .enumerate()
                .find(|(_, tab)| tab.id == hit.tab_id)
            {
                let label = format!(
                    "{}: {}:{}  {}",
                    tab_index + 1,
                    tab.document.file_name(),
                    hit.start.line + 1,
                    hit.preview
                );
                results = results.push(
                    button(text(label).size(12))
                        .style(if state.search_index == Some(index) {
                            button::secondary
                        } else {
                            button::text
                        })
                        .width(Length::Fill)
                        .on_press(Message::JumpTabSearch(index)),
                );
            }
        }
        if state.tab_search_hits.len() > 100 {
            let last = (first + 100).min(state.tab_search_hits.len());
            results = results.push(
                text(format!(
                    "Showing {}-{} of {} matches. Use arrows for more.",
                    first + 1,
                    last,
                    state.tab_search_hits.len()
                ))
                .size(12),
            );
        } else if state.tab_search_hits.is_empty() {
            results = results.push(text("No matches in open tabs.").size(12));
        }
        page = page.push(
            container(scrollable(results).height(Length::Fixed(180.0)))
                .width(Length::Fill)
                .padding([4, 20]),
        );
    }
    page.push(writing_area)
        .push(container(footer).width(Length::Fill).padding([12, 20]))
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
