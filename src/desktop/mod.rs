use crate::{app, ui};
use iced::keyboard::{self, Key, key::Named};
use iced::widget::text;
use iced::{Element, Subscription, Task, Theme, event, window};
use std::time::{Duration, Instant};

pub(crate) struct Desktop {
    editor: app::App,
    window: window::Id,
    startup: Option<Task<app::Message>>,
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Window(window::Id, app::Message),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    Tick(Instant),
}

pub(crate) fn boot() -> (Desktop, Task<Message>) {
    let (editor, startup) = app::boot();
    let (window, opening) = window::open(window::Settings {
        size: iced::Size::new(1000.0, 760.0),
        maximized: true,
        transparent: cfg!(any(target_os = "macos", target_os = "linux")),
        exit_on_close_request: false,
        ..Default::default()
    });
    (
        Desktop {
            editor,
            window,
            startup: Some(startup),
        },
        opening.map(Message::WindowOpened),
    )
}

pub(crate) fn update(state: &mut Desktop, message: Message) -> Task<Message> {
    match message {
        Message::Window(id, message) if id == state.window => {
            let save_layout = matches!(
                message,
                app::Message::New
                    | app::Message::NewGroup
                    | app::Message::Opened(Ok(_))
                    | app::Message::Saved(.., Ok(()))
                    | app::Message::SelectTab(_)
                    | app::Message::SelectGroup(_)
                    | app::Message::ToggleReading
                    | app::Message::ShowEditor
                    | app::Message::ShowSearch
                    | app::Message::ShowTabSearch
                    | app::Message::JumpOutline(_)
                    | app::Message::JumpTabSearch(_)
                    | app::Message::Format(_)
                    | app::Message::NextPane
                    | app::Message::PreviousPane
                    | app::Message::ActivatePane(_)
                    | app::Message::SinglePane
                    | app::Message::SplitPane(_)
                    | app::Message::ClosePane(_)
                    | app::Message::PaneDragged(_)
                    | app::Message::PaneResized(_)
                    | app::Message::CloseTab(_)
                    | app::Message::CloseCurrentTab
                    | app::Message::CloseCurrentGroup
                    | app::Message::CloseRequested(_)
                    | app::Message::SaveAndClose
                    | app::Message::DiscardAndClose
            );
            let task = app::update(&mut state.editor, message);
            if save_layout && let Err(error) = app::save_session(&state.editor) {
                log::warn!("layout save failed: {:?}", error.kind());
                state.editor.notice = Some(
                    "Could not save the window layout. It may not be restored at the next start."
                        .into(),
                );
            }
            task.map(move |message| Message::Window(id, message))
        }
        Message::WindowOpened(id) if id == state.window => {
            log::info!("window opened");
            let startup = state.startup.take().map_or_else(Task::none, |task| {
                task.map(move |message| Message::Window(id, message))
            });
            Task::batch([window::gain_focus(id), startup])
        }
        Message::WindowClosed(id) if id == state.window => {
            log::info!("window closed");
            iced::exit()
        }
        Message::Tick(now) => {
            let id = state.window;
            app::update(&mut state.editor, app::Message::Tick(now))
                .map(move |message| Message::Window(id, message))
        }
        _ => Task::none(),
    }
}

pub(crate) fn view(state: &Desktop, id: window::Id) -> Element<'_, Message> {
    if id == state.window {
        ui::view(&state.editor).map(move |message| Message::Window(id, message))
    } else {
        text("").into()
    }
}

pub(crate) fn theme(state: &Desktop, _id: window::Id) -> Theme {
    state.editor.theme.clone()
}

pub(crate) fn style(state: &Desktop, theme: &Theme) -> iced::theme::Style {
    let mut style = iced::theme::default(theme);
    if state.editor.transparent_background {
        style.background_color.a = f32::from(state.editor.background_opacity) / 100.0;
    }
    style
}

pub(crate) fn title(state: &Desktop, _id: window::Id) -> String {
    state.editor.title()
}

pub(crate) fn subscription(state: &Desktop) -> Subscription<Message> {
    let mut subscriptions = vec![
        window::close_requests().map(|id| Message::Window(id, app::Message::CloseRequested(id))),
        window::events().filter_map(|(id, event)| match event {
            window::Event::FileDropped(path) => {
                Some(Message::Window(id, app::Message::FileDropped(path)))
            }
            window::Event::Closed => Some(Message::WindowClosed(id)),
            _ => None,
        }),
        event::listen_with(keyboard_message),
    ];
    if state.editor.needs_tick() {
        subscriptions.push(iced::time::every(Duration::from_millis(250)).map(Message::Tick));
    }
    Subscription::batch(subscriptions)
}

fn keyboard_message(event: iced::Event, status: event::Status, id: window::Id) -> Option<Message> {
    match event {
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat: false,
            ..
        }) if status == event::Status::Ignored
            || (status == event::Status::Captured && key == Key::Named(Named::Escape)) =>
        {
            Some(Message::Window(
                id,
                app::Message::KeyPressed(key, modifiers),
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
