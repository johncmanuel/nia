#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum Slot {
    New,
    Open,
    Save,
    AlternateSave,
    Find,
    AlternateFind,
    Outline,
    Recent,
    NewTab,
    CloseTab,
    NextTab,
    PreviousTab,
    SinglePane,
    SplitRight,
    SplitDown,
    FindTabs,
    IncreaseFontSize,
    DecreaseFontSize,
    Settings,
    Zen,
}

impl Slot {
    pub(crate) const ALL: [Self; 20] = [
        Self::New,
        Self::Open,
        Self::Save,
        Self::AlternateSave,
        Self::Find,
        Self::AlternateFind,
        Self::Outline,
        Self::Recent,
        Self::NewTab,
        Self::CloseTab,
        Self::NextTab,
        Self::PreviousTab,
        Self::SinglePane,
        Self::SplitRight,
        Self::SplitDown,
        Self::FindTabs,
        Self::IncreaseFontSize,
        Self::DecreaseFontSize,
        Self::Settings,
        Self::Zen,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::New => "New document",
            Self::Open => "Open document",
            Self::Save => "Save document",
            Self::AlternateSave => "Save document (alternate)",
            Self::Find => "Find in document",
            Self::AlternateFind => "Find in document (alternate)",
            Self::Outline => "Document outline",
            Self::Recent => "Recent documents",
            Self::NewTab => "New tab group",
            Self::CloseTab => "Close document",
            Self::NextTab => "Next split",
            Self::PreviousTab => "Previous split",
            Self::SinglePane => "Single pane",
            Self::SplitRight => "Split right",
            Self::SplitDown => "Split down",
            Self::FindTabs => "Find in open tabs",
            Self::IncreaseFontSize => "Increase editor font size",
            Self::DecreaseFontSize => "Decrease editor font size",
            Self::Settings => "Open settings",
            Self::Zen => "Toggle Zen mode",
        }
    }
}
