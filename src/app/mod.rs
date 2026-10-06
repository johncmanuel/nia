use crate::document::Document;
use crate::font;
use crate::keybindings::{Keybindings, Slot};
use crate::recent;
use crate::recovery;
use crate::theme;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{combo_box, markdown, pane_grid, text_editor};
use iced::{Font, Task, Theme, window};
use std::path::PathBuf;
use std::time::Instant;

mod layout;
mod lifecycle;
mod overlays;
mod save;
mod search_tabs;
mod state;
mod update;

pub(crate) use update::update;

pub(crate) const EDITOR: &str = "editor";
pub(crate) const SEARCH: &str = "search";
pub(crate) const OUTLINE: &str = "outline";
pub(crate) const RECENT: &str = "recent";

pub(crate) struct Tab {
    pub(crate) id: u64,
    pub(crate) document: Document,
    pub(crate) editor: text_editor::Content,
    pub(crate) reading: Option<markdown::Content>,
    pub(crate) saving: bool,
    pub(crate) checking: bool,
    pub(crate) conflict: bool,
    pub(crate) recovering: bool,
    pub(crate) recovered_revision: u64,
    pub(crate) recovery_key: String,
    last_edit: Option<Instant>,
    last_checked: Option<Instant>,
}

impl Tab {
    fn new(id: u64, document: Document) -> Self {
        let editor = text_editor::Content::with_text(&document.contents);
        Self {
            id,
            document,
            editor,
            reading: None,
            saving: false,
            checking: false,
            conflict: false,
            recovering: false,
            recovered_revision: 0,
            recovery_key: recovery::new_key(),
            last_edit: None,
            last_checked: None,
        }
    }

    fn recovered(id: u64, draft: recovery::Draft) -> Self {
        let mut document = Document::new();
        document.edit(draft.contents);
        document.dirty = true;
        let mut tab = Self::new(id, document);
        tab.recovery_key = draft.key;
        tab.recovered_revision = tab.document.revision;
        tab
    }

    fn sync_document(&mut self) {
        self.document.sync_contents(self.editor.text());
    }

    fn refresh_reading(&mut self) {
        if self.reading.is_some() {
            self.reading = Some(markdown::Content::parse(&self.editor.text()));
        }
    }
}

pub(crate) struct App {
    pub(crate) tabs: Vec<Tab>,
    pub(crate) active: usize,
    pub(crate) panes: pane_grid::State<u64>,
    pub(crate) active_pane: pane_grid::Pane,
    pub(crate) show_layout: bool,
    next_id: u64,
    pub(crate) notice: Option<String>,
    pub(crate) loading: bool,
    pub(crate) dialog_open: bool,
    pub(crate) keybindings: Keybindings,
    pub(crate) show_settings: bool,
    pub(crate) recording: Option<Slot>,
    pub(crate) theme: Theme,
    pub(crate) transparent_background: bool,
    pub(crate) show_theme_picker: bool,
    pub(crate) font_families: Vec<String>,
    pub(crate) font_options: combo_box::State<String>,
    pub(crate) font_family: Option<String>,
    pub(crate) editor_font: Font,
    pub(crate) editor_font_size: u16,
    pub(crate) show_search: bool,
    pub(crate) search_all_tabs: bool,
    pub(crate) search_query: String,
    pub(crate) search_index: Option<usize>,
    pub(crate) search_case_sensitive: bool,
    search_hits: Vec<(text_editor::Position, text_editor::Position)>,
    pub(crate) tab_search_hits: Vec<search_tabs::TabMatch>,
    pub(crate) show_outline: bool,
    pub(crate) outline_query: String,
    pub(crate) outline_index: usize,
    pub(crate) recent: Vec<PathBuf>,
    pub(crate) show_recent: bool,
    pub(crate) recent_query: String,
    pub(crate) recent_index: usize,
    pub(crate) pending_close: Option<CloseTarget>,
    confirmed_close: Option<CloseTarget>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CloseTarget {
    Tab(u64),
    Window(window::Id),
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Edit(u64, text_editor::Action),
    New,
    Open,
    OpenPicked(Option<PathBuf>),
    Opened(Result<Document, String>),
    Save,
    SavePicked(Option<PathBuf>),
    SaveAs,
    SaveAsPicked(Option<PathBuf>),
    Saved(u64, PathBuf, u64, String, Result<(), String>),
    RecoveryWritten(u64, u64, Result<(), String>),
    ExternalChecked(u64, PathBuf, u64, Result<String, String>),
    Tick(Instant),
    CloseRequested(window::Id),
    SaveAndClose,
    DiscardAndClose,
    CancelClose,
    FileDropped(PathBuf),
    SelectTab(usize),
    CloseCurrentTab,
    NextTab,
    PreviousTab,
    ActivatePane(pane_grid::Pane),
    SinglePane,
    ToggleLayout,
    SplitPane(pane_grid::Axis),
    ClosePane(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    CloseTab(usize),
    ShowSettings,
    ShowEditor,
    ToggleReading,
    CopyLink(String),
    RecordShortcut(Slot),
    ResetKeybindings,
    SetTheme(Theme),
    SetTransparentBackground(bool),
    ToggleThemePicker,
    HideThemePicker,
    SetFont(String),
    ResetFont,
    IncreaseFontSize,
    DecreaseFontSize,
    ShowSearch,
    ShowTabSearch,
    HideSearch,
    SearchAllTabs(bool),
    SearchChanged(String),
    SearchCaseChanged(bool),
    FindNext,
    FindPrevious,
    JumpTabSearch(usize),
    ShowOutline,
    HideOutline,
    OutlineChanged(String),
    OutlineNext,
    OutlinePrevious,
    JumpOutline(usize),
    JumpOutlineSelected,
    ShowRecent,
    HideRecent,
    RecentChanged(String),
    RecentNext,
    RecentPrevious,
    OpenRecentSelected,
    OpenRecent(PathBuf),
    KeyPressed(Key, Modifiers),
}

pub(crate) fn boot() -> (App, Task<Message>) {
    let mut state = blank();
    match recovery::load() {
        Ok(drafts) if !drafts.is_empty() => {
            state.tabs.clear();
            for draft in drafts {
                let id = state.next_id();
                state.tabs.push(Tab::recovered(id, draft));
            }
            state.active = state.tabs.len() - 1;
            if let Some(tab_id) = state.panes.get_mut(state.active_pane) {
                *tab_id = state.tabs[state.active].id;
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
    let (panes, active_pane) = pane_grid::State::new(0);
    let state = App {
        tabs: vec![Tab::new(0, Document::new())],
        active: 0,
        panes,
        active_pane,
        show_layout: false,
        next_id: 1,
        notice: notice
            .or(theme_notice)
            .or(transparency_notice)
            .or(font_notice)
            .or(size_notice),
        loading: false,
        dialog_open: false,
        keybindings,
        show_settings: false,
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

fn open_document(path: PathBuf) -> Task<Message> {
    Task::perform(
        async move { Document::open(path).map_err(|error| error.to_string()) },
        Message::Opened,
    )
}

pub(crate) fn shortcut_message(slot: Slot) -> Message {
    match slot {
        Slot::New | Slot::NewTab => Message::New,
        Slot::Open => Message::Open,
        Slot::Save | Slot::AlternateSave => Message::Save,
        Slot::Find | Slot::AlternateFind => Message::ShowSearch,
        Slot::FindTabs => Message::ShowTabSearch,
        Slot::Outline => Message::ShowOutline,
        Slot::Recent => Message::ShowRecent,
        Slot::CloseTab => Message::CloseCurrentTab,
        Slot::NextTab => Message::NextTab,
        Slot::PreviousTab => Message::PreviousTab,
        Slot::SinglePane => Message::SinglePane,
        Slot::SplitRight => Message::SplitPane(pane_grid::Axis::Vertical),
        Slot::SplitDown => Message::SplitPane(pane_grid::Axis::Horizontal),
        Slot::IncreaseFontSize => Message::IncreaseFontSize,
        Slot::DecreaseFontSize => Message::DecreaseFontSize,
        Slot::Settings => Message::ShowSettings,
    }
}

impl App {
    pub(crate) fn key_message(&self, key: &Key, modifiers: Modifiers) -> Option<Message> {
        if let Some(slot) = self.keybindings.matching(key, modifiers) {
            return Some(shortcut_message(slot));
        }
        if modifiers != Modifiers::COMMAND {
            return None;
        }
        let Key::Character(value) = key else {
            return None;
        };
        let number = value.parse::<usize>().ok()?;
        let index = match number {
            1..=8 => number - 1,
            9 => self.tabs.len().checked_sub(1)?,
            _ => return None,
        };
        (index < self.tabs.len()).then_some(Message::SelectTab(index))
    }
}

#[cfg(test)]
mod tests;
