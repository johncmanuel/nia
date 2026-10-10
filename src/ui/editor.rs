use crate::app::{App, EDITOR, Message, RECENT, Tab};
use crate::markdown::{self, HighlightSettings, MarkdownHighlighter};
use crate::ui::reading::reading_view;
use iced::widget::{
    button, column, container, pane_grid, row, scrollable, space, text, text_editor, text_input,
};
use iced::{Background, Border, Element, Length, Theme};

pub(super) fn editor_view<'a>(state: &'a App, tab: &'a Tab, active: bool) -> Element<'a, Message> {
    let line = tab.editor.cursor().position.line;
    let highlight = HighlightSettings {
        active_line: active.then_some(line),
        highlight_line: active && state.focus.highlight_line,
    };
    let mut editor = text_editor(&tab.editor)
        .id(if active {
            EDITOR.to_string()
        } else {
            format!("editor-{}", tab.id)
        })
        .highlight_with::<MarkdownHighlighter>(highlight, markdown::format)
        .placeholder("# Start writing")
        .font(state.editor_font)
        .size(u32::from(state.editor_font_size))
        .line_height(text::LineHeight::Relative(1.55))
        .wrapping(text::Wrapping::WordOrGlyph)
        .padding(0)
        .height(Length::Fill)
        .style(|theme, status| {
            let mut style = text_editor::default(theme, status);
            let mut background = theme.palette().background;
            if state.transparent_background {
                background.a = 0.0;
            }
            style.background = Background::Color(background);
            style.border = Border::default();
            style.placeholder = theme.palette().text.scale_alpha(0.45);
            style.value = theme.palette().text;
            style
        })
        .key_binding(|event| {
            if matches!(event.status, text_editor::Status::Focused { .. })
                && let Some(message) = state.key_message(&event.key, event.modifiers)
            {
                return Some(text_editor::Binding::Custom(message));
            }
            text_editor::Binding::from_key_press(event)
        });
    if !state.loading && !state.dialog_open {
        editor = editor.on_action(move |action| Message::Edit(tab.id, action));
    }

    let writing_area: Element<'_, Message> = if tab.reading.is_some() {
        reading_view(state, tab)
    } else {
        container(editor)
            .width(Length::Fill)
            .max_width(760)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .into()
    };
    let mut content = column![]
        .spacing(12)
        .width(Length::Fill)
        .max_width(760)
        .height(Length::Fill);
    if tab.conflict {
        content = content.push(
            row![
                text("File changed or is unavailable on disk. Your edits are still here.").size(12),
                button("Save As").on_press(Message::SaveAs),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );
    }
    if active && state.show_recent {
        let mut results = column![].spacing(2);
        for (index, path) in state.recent_items().iter().enumerate() {
            results = results.push(
                button(text(path.to_string_lossy().to_string()).size(12))
                    .style(if index == state.recent_index {
                        button::primary
                    } else {
                        button::text
                    })
                    .width(Length::Fill)
                    .on_press(Message::OpenRecent((*path).clone())),
            );
        }
        content = content.push(
            column![
                row![
                    text_input("Find recent file", &state.recent_query)
                        .id(RECENT)
                        .on_input(Message::RecentChanged)
                        .on_submit(Message::OpenRecentSelected)
                        .width(Length::Fill),
                    button("\u{00D7}").on_press(Message::HideRecent),
                ]
                .spacing(8),
                scrollable(results).height(220),
            ]
            .spacing(8),
        );
    }
    content = content.push(writing_area);
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .padding(if state.panes.len() > 1 {
            [24, 20]
        } else {
            [48, 32]
        })
        .into()
}

pub(super) fn panes_view(state: &App) -> Element<'_, Message> {
    pane_grid::PaneGrid::new(&state.panes, |pane, tab_id, _| {
        let tab = state
            .tabs
            .iter()
            .find(|tab| tab.id == *tab_id)
            .unwrap_or_else(|| state.current());
        let active = pane == state.active_pane;
        let title = row![
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
            content
                .title_bar(pane_grid::TitleBar::new(title).padding([8, 12]))
                .style(pane_border)
        } else {
            content
        }
    })
    .spacing(8)
    .on_click(Message::ActivatePane)
    .on_drag(Message::PaneDragged)
    .on_resize(8, Message::PaneResized)
    .into()
}

fn pane_border(theme: &Theme) -> container::Style {
    container::Style::default().border(Border {
        color: theme.extended_palette().background.strong.color,
        width: 1.0,
        ..Border::default()
    })
}
