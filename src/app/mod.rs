use crate::document::Document;
use crate::focus;
use crate::keybindings::{Keybindings, Slot};
use iced::keyboard::{Key, Modifiers};
use iced::widget::{combo_box, pane_grid, text_editor};
use iced::{Font, Theme, window};
use std::path::PathBuf;
use std::time::Instant;

mod file_failure;
mod formatting;
mod history;
mod keyboard;
mod layout;
mod lifecycle;
mod overlays;
mod preferences;
mod save;
mod search_tabs;
mod session;
mod shared;
mod startup;
mod state;
mod tab;
mod update;

use file_failure::FileFailure;
pub(crate) use formatting::Format;
pub(crate) use session::save_session;
#[cfg(test)]
pub(crate) use startup::blank;
pub(crate) use startup::boot;
use startup::open_document;
pub(crate) use tab::Tab;
pub(crate) use update::update;

pub(crate) const EDITOR: &str = "editor";
pub(crate) const SEARCH: &str = "search";
pub(crate) const OUTLINE: &str = "outline";
pub(crate) const RECENT: &str = "recent";

pub(crate) struct App {
    pub(crate) tabs: Vec<Tab>,
    pub(crate) active: usize,
    pub(crate) panes: pane_grid::State<u64>,
    pub(crate) active_pane: pane_grid::Pane,
    pub(crate) groups: Vec<TabGroup>,
    pub(crate) active_group: usize,
    next_id: u64,
    pub(crate) notice: Option<String>,
    pub(crate) loading: bool,
    pub(crate) dialog_open: bool,
    pub(crate) keybindings: Keybindings,
    pub(crate) show_settings: bool,
    pub(crate) show_format_menu: bool,
    pub(crate) handbook: Option<iced::widget::markdown::Content>,
    pub(crate) zen_mode: bool,
    pub(crate) focus: focus::Settings,
    pub(crate) recording: Option<Slot>,
    pub(crate) theme: Theme,
    pub(crate) transparent_background: bool,
    pub(crate) background_opacity: u8,
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

pub(crate) struct TabGroup {
    pub(crate) tabs: Vec<u64>,
    // The active group keeps its layout in App.panes instead.
    pub(crate) inactive: Option<GroupLayout>,
}

pub(crate) struct GroupLayout {
    pub(crate) panes: pane_grid::State<u64>,
    pub(crate) active_pane: pane_grid::Pane,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CloseTarget {
    Tab(u64),
    Group(u64),
    Window(window::Id),
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Edit(u64, text_editor::Action),
    ToggleFormatMenu,
    Format(Format),
    Undo,
    Redo,
    New,
    NewGroup,
    Open,
    OpenPicked(Option<PathBuf>),
    Opened(Result<Document, FileFailure>),
    Save,
    SavePicked(Option<PathBuf>),
    SaveAs,
    SaveAsPicked(Option<PathBuf>),
    Saved(u64, PathBuf, u64, String, Result<(), FileFailure>),
    RecoveryWritten(u64, u64, Result<(), FileFailure>),
    ExternalChecked(u64, PathBuf, u64, Result<String, FileFailure>),
    Tick(Instant),
    CloseRequested(window::Id),
    SaveAndClose,
    DiscardAndClose,
    CancelClose,
    FileDropped(PathBuf),
    SelectTab(usize),
    SelectGroup(usize),
    CloseCurrentTab,
    CloseCurrentGroup,
    NextPane,
    PreviousPane,
    ActivatePane(pane_grid::Pane),
    SinglePane,
    SplitPane(pane_grid::Axis),
    ClosePane(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    CloseTab(usize),
    ShowSettings,
    ShowHandbook,
    HideHandbook,
    ShowEditor,
    ToggleZen,
    SetHighlightLine(bool),
    SetInactivePaneDim(u8),
    ToggleReading,
    CopyLink(String),
    RecordShortcut(Slot),
    ResetKeybindings,
    SetTheme(Theme),
    SetTransparentBackground(bool),
    SetBackgroundOpacity(u8),
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

#[cfg(test)]
mod tests;
