use super::muted_text;
use crate::app::{App, Message, SEARCH};
use iced::widget::{
    button, checkbox, column, container, pane_grid, row, rule, scrollable, text, text_input,
};
use iced::{Element, Length};

pub(super) fn header(state: &App) -> Element<'_, Message> {
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
            let tabs = if index == 0 {
                tabs
            } else {
                tabs.push(rule::vertical(1))
            };
            tabs.push(
                row![
                    button(text(name).size(12))
                        .style(if index == state.active {
                            button::secondary
                        } else {
                            button::text
                        })
                        .on_press(Message::SelectTab(index)),
                    button(text("\u{00D7}").size(13))
                        .style(button::text)
                        .on_press(Message::CloseTab(index)),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
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
    header = header.push(
        button(text("Layout").size(12))
            .style(button::text)
            .on_press(Message::ToggleLayout),
    );
    container(header)
        .width(Length::Fill)
        .padding([8, 20])
        .into()
}

pub(super) fn search_results(state: &App) -> Option<Element<'_, Message>> {
    if !state.show_search || !state.search_all_tabs || state.search_query.is_empty() {
        return None;
    }
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
    Some(
        container(scrollable(results).height(Length::Fixed(180.0)))
            .width(Length::Fill)
            .padding([4, 20])
            .into(),
    )
}
