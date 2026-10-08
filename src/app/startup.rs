use super::{App, Message, Tab, TabGroup, session};
use crate::document::Document;
use crate::keybindings::Keybindings;
use crate::{focus, font, recent, recovery, theme};
use iced::widget::{combo_box, pane_grid};
use iced::{Font, Task, Theme};
use std::path::PathBuf;

pub(crate) fn boot() -> (App, Task<Message>) {
    let mut state = blank();
    match session::load() {
        Ok(Some(saved)) => session::restore(&mut state, saved),
        Err(_) => {
            state.notice = Some("Could not restore the saved layout.".into());
            log::warn!("layout restore failed");
        }
        Ok(None) => {}
    }
    match recovery::load() {
        Ok(drafts) if !drafts.is_empty() => {
            for draft in drafts {
                state.push_recovered_group(draft);
            }
            state.notice = Some("Recovered local drafts. Save each draft to keep it.".into());
            log::info!("local drafts recovered");
        }
        Err(_) => {
            state.notice = Some("Could not read local recovery drafts.".into());
            log::warn!("recovery load failed");
        }
        _ => {}
    }
    let task = if let Some(path) = std::env::args_os().nth(1) {
        state.loading = true;
        open_document(PathBuf::from(path))
    } else {
        Task::none()
    };
    (state, task)
}

pub(crate) fn blank() -> App {
    let (keybindings, notice) = match Keybindings::load() {
        Ok(bindings) => (bindings, None),
        Err(error) => {
            log::warn!("keybindings load failed");
            (
                Keybindings::default(),
                Some(format!("Could not load keybindings: {error}")),
            )
        }
    };
    let (theme, theme_notice) = match theme::load() {
        Ok(theme) => (theme, None),
        Err(error) => {
            log::warn!("theme load failed");
            (
                Theme::CatppuccinMocha,
                Some(format!("Could not load theme: {error}")),
            )
        }
    };
    let (transparent_background, transparency_notice) = match theme::load_transparency() {
        Ok(value) => (value, None),
        Err(error) => {
            log::warn!("transparency setting load failed");
            (
                false,
                Some(format!("Could not load transparency setting: {error}")),
            )
        }
    };
    let recent = match recent::load() {
        Ok(paths) => paths,
        Err(_) => {
            log::warn!("recent documents load failed");
            Vec::new()
        }
    };
    let font_families = font::installed_families();
    let (font_family, font_notice) = match font::load() {
        Ok(Some(family)) if font_families.contains(&family) => (Some(family), None),
        Ok(Some(_)) => (None, Some("Saved font is no longer installed.".into())),
        Ok(None) => (None, None),
        Err(error) => (None, Some(format!("Could not load font setting: {error}"))),
    };
    let editor_font = font_family
        .as_deref()
        .map_or(Font::MONOSPACE, font::font_for);
    let (editor_font_size, size_notice) = match font::load_size() {
        Ok(size) => (size, None),
        Err(error) => (
            font::DEFAULT_SIZE,
            Some(format!("Could not load font size: {error}")),
        ),
    };
    let (focus, focus_notice) = match focus::Settings::load() {
        Ok(settings) => (settings, None),
        Err(error) => {
            log::warn!("focus settings load failed");
            (
                focus::Settings::default(),
                Some(format!("Could not load focus settings: {error}")),
            )
        }
    };
    let (panes, active_pane) = pane_grid::State::new(0);
    let state = App {
        tabs: vec![Tab::new(0, Document::new())],
        active: 0,
        panes,
        active_pane,
        groups: vec![TabGroup {
            tabs: vec![0],
            inactive: None,
        }],
        active_group: 0,
        next_id: 1,
        notice: notice
            .or(theme_notice)
            .or(transparency_notice)
            .or(font_notice)
            .or(size_notice)
            .or(focus_notice),
        loading: false,
        dialog_open: false,
        keybindings,
        show_settings: false,
        handbook: None,
        zen_mode: false,
        focus,
        recording: None,
        theme,
        transparent_background,
        show_theme_picker: false,
        font_options: combo_box::State::new(font_families.clone()),
        font_families,
        font_family,
        editor_font,
        editor_font_size,
        show_search: false,
        search_all_tabs: false,
        search_query: String::new(),
        search_index: None,
        search_case_sensitive: false,
        search_hits: Vec::new(),
        tab_search_hits: Vec::new(),
        show_outline: false,
        outline_query: String::new(),
        outline_index: 0,
        recent,
        show_recent: false,
        recent_query: String::new(),
        recent_index: 0,
        pending_close: None,
        confirmed_close: None,
    };
    log::info!("editor started");
    state
}

pub(super) fn open_document(path: PathBuf) -> Task<Message> {
    Task::perform(
        async move { Document::open(path).map_err(|error| error.to_string()) },
        Message::Opened,
    )
}
