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
        Ok(Some(saved)) => {
            session::restore(&mut state, saved);
            log::info!("saved layout restored");
        }
        Err(error) => {
            state.notice = Some("Could not restore the saved layout.".into());
            log::warn!("layout restore failed: {:?}", error.kind());
        }
        Ok(None) => {}
    }
    match recovery::load() {
        Ok(loaded) => {
            let recovered = !loaded.drafts.is_empty();
            for draft in loaded.drafts {
                state.push_recovered_group(draft);
            }
            if loaded.skipped > 0 {
                state.notice = Some(
                    "Some recovery drafts could not be read. They remain on disk. Save the drafts that opened."
                        .into(),
                );
                log::warn!("some recovery drafts could not be read");
            } else if recovered {
                state.notice = Some("Recovered local drafts. Save each draft to keep it.".into());
            }
            if recovered {
                log::info!("local drafts recovered");
            }
        }
        Err(error) => {
            state.notice = Some("Could not read local recovery drafts.".into());
            log::warn!("recovery load failed: {:?}", error.kind());
        }
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
            log::warn!("keybindings load failed: {:?}", error.kind());
            (
                Keybindings::default(),
                Some(format!("Could not load keybindings: {error}")),
            )
        }
    };
    let (theme, theme_notice) = match theme::load() {
        Ok(theme) => (theme, None),
        Err(error) => {
            log::warn!("theme load failed: {:?}", error.kind());
            (
                Theme::CatppuccinMocha,
                Some(format!("Could not load theme: {error}")),
            )
        }
    };
    let ((transparent_background, background_opacity), transparency_notice) =
        match theme::load_transparency() {
            Ok(value) => (value, None),
            Err(error) => {
                log::warn!("transparency setting load failed: {:?}", error.kind());
                (
                    (false, theme::DEFAULT_OPACITY),
                    Some(format!("Could not load transparency setting: {error}")),
                )
            }
        };
    let recent = match recent::load() {
        Ok(paths) => paths,
        Err(error) => {
            log::warn!("recent documents load failed: {:?}", error.kind());
            Vec::new()
        }
    };
    let font_families = font::installed_families();
    let (font_family, font_notice) = match font::load() {
        Ok(Some(family)) if font_families.contains(&family) => (Some(family), None),
        Ok(Some(_)) => {
            log::warn!("saved font is unavailable");
            (None, Some("Saved font is no longer installed.".into()))
        }
        Ok(None) => (None, None),
        Err(error) => {
            log::warn!("font setting load failed: {:?}", error.kind());
            (None, Some(format!("Could not load font setting: {error}")))
        }
    };
    let editor_font = font_family
        .as_deref()
        .map_or(Font::MONOSPACE, font::font_for);
    let (editor_font_size, size_notice) = match font::load_size() {
        Ok(size) => (size, None),
        Err(error) => {
            log::warn!("font size load failed: {:?}", error.kind());
            (
                font::DEFAULT_SIZE,
                Some(format!("Could not load font size: {error}")),
            )
        }
    };
    let (focus, focus_notice) = match focus::Settings::load() {
        Ok(settings) => (settings, None),
        Err(error) => {
            log::warn!("focus settings load failed: {:?}", error.kind());
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
        show_format_menu: false,
        handbook: None,
        zen_mode: false,
        focus,
        recording: None,
        theme,
        transparent_background,
        background_opacity,
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
        async move {
            Document::open(path).map_err(|error| {
                log::warn!("document open failed: {:?}", error.kind());
                error.into()
            })
        },
        Message::Opened,
    )
}
