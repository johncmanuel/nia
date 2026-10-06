use crate::keybindings::config_dir;
use iced::Theme;
use std::fs;
use std::io::{self, ErrorKind};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum WritingTheme {
    #[default]
    Catppuccin,
    TokyoNight,
    Nord,
    Gruvbox,
}

impl WritingTheme {
    pub(crate) const ALL: [Self; 4] = [
        Self::Catppuccin,
        Self::TokyoNight,
        Self::Nord,
        Self::Gruvbox,
    ];

    pub(crate) fn iced(self) -> Theme {
        match self {
            Self::Catppuccin => Theme::CatppuccinMocha,
            Self::TokyoNight => Theme::TokyoNight,
            Self::Nord => Theme::Nord,
            Self::Gruvbox => Theme::GruvboxDark,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Catppuccin => "Catppuccin",
            Self::TokyoNight => "Tokyo Night",
            Self::Nord => "Nord",
            Self::Gruvbox => "Gruvbox",
        }
    }

    pub(crate) fn load() -> io::Result<Self> {
        match fs::read_to_string(config_dir()?.join("theme.conf")) {
            Ok(value) => Self::ALL
                .into_iter()
                .find(|theme| theme.label() == value.trim())
                .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "invalid theme setting")),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn save(self) -> io::Result<()> {
        let directory = config_dir()?;
        fs::create_dir_all(&directory)?;
        fs::write(directory.join("theme.conf"), self.label())
    }
}

impl std::fmt::Display for WritingTheme {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.label())
    }
}
