use crate::document::{Document, save_file};
use iced::keyboard::{self, Key};
use iced::widget::{operation, text_editor};
use iced::{Subscription, Task, window};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub(crate) const EDITOR: &str = "editor";

pub(crate) struct App {
    pub(crate) document: Document,
    pub(crate) editor: text_editor::Content,
    pub(crate) notice: Option<String>,
    pub(crate) saving: bool,
    pub(crate) loading: bool,
    pub(crate) dialog_open: bool,
    last_edit: Option<Instant>,
    close_after_save: Option<window::Id>,
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Edit(text_editor::Action),
    New,
    Open,
    OpenPicked(Option<PathBuf>),
    Opened(Result<Document, String>),
    Save,
    SavePicked(Option<PathBuf>),
    Saved(PathBuf, u64, String, Result<(), String>),
    Tick(Instant),
    CloseRequested(window::Id),
    FileDropped(PathBuf),
}

pub(crate) fn boot() -> (App, Task<Message>) {
    let mut state = App {
        document: Document::new(),
        editor: text_editor::Content::new(),
        notice: None,
        saving: false,
        loading: false,
        dialog_open: false,
        last_edit: None,
        close_after_save: None,
    };
    let task = if let Some(path) = std::env::args_os().nth(1) {
        state.loading = true;
        open_document(PathBuf::from(path))
    } else {
        Task::none()
    };
    (state, task)
}

pub(crate) fn update(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Edit(action) => {
            if state.loading || state.dialog_open {
                return Task::none();
            }
            let changed = matches!(action, text_editor::Action::Edit(_));
            state.editor.perform(action);
            if changed {
                state.document.edit(state.editor.text());
                state.last_edit = Some(Instant::now());
                state.notice = None;
            }
        }
        Message::New => {
            if !state.can_leave_document() {
                return Task::none();
            }
            state.document = Document::new();
            state.editor = text_editor::Content::new();
            state.last_edit = None;
            state.notice = None;
            return operation::focus(EDITOR);
        }
        Message::Open => {
            if !state.can_leave_document() || state.dialog_open {
                return Task::none();
            }
            state.dialog_open = true;
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Markdown", &["md", "markdown"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf())
                },
                Message::OpenPicked,
            );
        }
        Message::OpenPicked(path) => {
            state.dialog_open = false;
            if let Some(path) = path {
                if !state.can_leave_document() {
                    return Task::none();
                }
                state.loading = true;
                return open_document(path);
            }
            return operation::focus(EDITOR);
        }
        Message::Opened(result) => {
            state.loading = false;
            match result {
                Ok(document) => {
                    state.editor = text_editor::Content::with_text(&document.contents);
                    state.document = document;
                    state.last_edit = None;
                    state.notice = None;
                }
                Err(error) => state.notice = Some(format!("Could not open file: {error}")),
            }
            return operation::focus(EDITOR);
        }
        Message::Save => return state.save(),
        Message::SavePicked(path) => {
            state.dialog_open = false;
            if let Some(path) = path {
                return state.start_save(path);
            }
            state.close_after_save = None;
            return operation::focus(EDITOR);
        }
        Message::Saved(path, revision, contents, result) => {
            state.saving = false;
            match result {
                Ok(()) => {
                    let was_new = state.document.path.is_none();
                    state.document.saved(path, revision, contents);
                    state.notice = None;
                    if !state.document.dirty {
                        if let Some(id) = state.close_after_save.take() {
                            return window::close(id);
                        }
                    } else if state.close_after_save.is_some() {
                        return state.save();
                    }
                    if was_new {
                        return operation::focus(EDITOR);
                    }
                }
                Err(error) => {
                    state.last_edit = None;
                    state.close_after_save = None;
                    state.notice = Some(format!("Could not save file: {error}"));
                }
            }
        }
        Message::Tick(now) => {
            if state.document.dirty
                && !state.saving
                && state.document.path.is_some()
                && state
                    .last_edit
                    .is_some_and(|last| now.duration_since(last) >= Duration::from_millis(800))
            {
                return state.save();
            }
        }
        Message::CloseRequested(id) => {
            if !state.document.dirty && !state.saving {
                return window::close(id);
            }
            state.close_after_save = Some(id);
            if !state.saving && !state.dialog_open {
                return state.save();
            }
        }
        Message::FileDropped(path) => {
            if !state.can_leave_document() {
                return Task::none();
            }
            state.loading = true;
            return open_document(path);
        }
    }
    Task::none()
}

impl App {
    pub(crate) fn title(&self) -> String {
        format!(
            "{}{} — nia",
            self.document.file_name(),
            if self.document.dirty { " •" } else { "" }
        )
    }

    fn can_leave_document(&mut self) -> bool {
        if self.document.dirty || self.saving || self.loading {
            self.notice = Some("Save the current document before opening another.".into());
            false
        } else {
            true
        }
    }

    fn save(&mut self) -> Task<Message> {
        if self.saving || self.loading || self.dialog_open {
            return Task::none();
        }
        if let Some(path) = self.document.path.clone() {
            self.start_save(path)
        } else {
            self.dialog_open = true;
            Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Markdown", &["md", "markdown"])
                        .set_file_name("Untitled.md")
                        .save_file()
                        .await
                        .map(|file| file.path().to_path_buf())
                },
                Message::SavePicked,
            )
        }
    }

    fn start_save(&mut self, path: PathBuf) -> Task<Message> {
        self.saving = true;
        self.last_edit = None;
        let revision = self.document.revision;
        let contents = self.document.contents.clone();
        let expected = self.document.saved_contents.clone();
        let save_path = path.clone();
        Task::perform(
            async move {
                save_file(&save_path, &contents, expected.as_deref())
                    .map_err(|error| error.to_string())
                    .map(|()| contents)
            },
            move |result| match result {
                Ok(contents) => Message::Saved(path, revision, contents, Ok(())),
                Err(error) => Message::Saved(path, revision, String::new(), Err(error)),
            },
        )
    }
}

fn open_document(path: PathBuf) -> Task<Message> {
    Task::perform(
        async move { Document::open(path).map_err(|error| error.to_string()) },
        Message::Opened,
    )
}

pub(crate) fn subscription(state: &App) -> Subscription<Message> {
    let mut subscriptions = vec![
        window::close_requests().map(Message::CloseRequested),
        window::events().filter_map(|(_, event)| match event {
            window::Event::FileDropped(path) => Some(Message::FileDropped(path)),
            _ => None,
        }),
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => shortcut(&key, modifiers),
            _ => None,
        }),
    ];
    if state.document.dirty && state.document.path.is_some() && state.last_edit.is_some() {
        subscriptions.push(iced::time::every(Duration::from_millis(250)).map(Message::Tick));
    }
    Subscription::batch(subscriptions)
}

pub(crate) fn shortcut(key: &Key, modifiers: keyboard::Modifiers) -> Option<Message> {
    if !modifiers.command() {
        return None;
    }
    match key.as_ref() {
        Key::Character("n") => Some(Message::New),
        Key::Character("o") => Some(Message::Open),
        Key::Character("s") => Some(Message::Save),
        _ => None,
    }
}
