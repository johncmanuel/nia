use crate::app::{App, EDITOR, Message};
use iced::widget::{button, column, container, row, space, text, text_editor};
use iced::{Background, Border, Element, Font, Length, Theme};

pub(crate) fn view(state: &App) -> Element<'_, Message> {
    let mut editor = text_editor(&state.editor)
        .id(EDITOR)
        .placeholder("# Start writing")
        .font(Font::MONOSPACE)
        .size(20)
        .line_height(text::LineHeight::Relative(1.55))
        .wrapping(text::Wrapping::Word)
        .padding(0)
        .height(Length::Fill)
        .style(|theme, status| {
            let mut style = text_editor::default(theme, status);
            style.background = Background::Color(theme.palette().background);
            style.border = Border::default();
            style.placeholder = theme.palette().text.scale_alpha(0.45);
            style.value = theme.palette().text;
            style
        });
    if !state.loading && !state.dialog_open {
        editor = editor.on_action(Message::Edit);
    }

    let writing_area = container(editor)
        .width(Length::Fill)
        .max_width(760)
        .height(Length::Fill);
    let writing_area = container(writing_area)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .padding([48, 32]);

    let status = state.notice.as_deref().unwrap_or_else(|| {
        if state.loading {
            "Opening…"
        } else if state.saving {
            "Saving…"
        } else if state.document.dirty {
            "Unsaved"
        } else if state.document.path.is_some() {
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
        text(status).size(12).style(muted_text),
        space::horizontal(),
        text(format!("{} words", state.document.word_count))
            .size(12)
            .style(muted_text),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center);

    column![
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
