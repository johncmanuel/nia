use iced::keyboard::{Key, Modifiers, key::Named};
use std::env;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

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
    Typewriter,
}

impl Slot {
    pub(crate) const ALL: [Self; 21] = [
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
        Self::Typewriter,
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
            Self::NewTab => "New tab",
            Self::CloseTab => "Close tab",
            Self::NextTab => "Next tab",
            Self::PreviousTab => "Previous tab",
            Self::SinglePane => "Single pane",
            Self::SplitRight => "Split right",
            Self::SplitDown => "Split down",
            Self::FindTabs => "Find in open tabs",
            Self::IncreaseFontSize => "Increase editor font size",
            Self::DecreaseFontSize => "Decrease editor font size",
            Self::Settings => "Open settings",
            Self::Zen => "Toggle Zen mode",
            Self::Typewriter => "Toggle typewriter mode",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShortcutKey {
    Character(char),
    Tab,
    Plus,
    Minus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Shortcut {
    key: ShortcutKey,
    modifiers: Modifiers,
}

impl Shortcut {
    fn new(key: char, modifiers: Modifiers) -> Option<Self> {
        let key = key.to_ascii_lowercase();
        if !key.is_ascii_alphanumeric() && !matches!(key, '+' | '-' | '=' | ',')
            || !modifiers.intersects(Modifiers::CTRL | Modifiers::LOGO)
            || modifiers.alt()
        {
            return None;
        }
        let mut modifiers = modifiers;
        if key == '+' || key == '=' {
            modifiers.remove(Modifiers::SHIFT);
        }
        Some(Self {
            key: match key {
                '+' | '=' => ShortcutKey::Plus,
                '-' => ShortcutKey::Minus,
                _ => ShortcutKey::Character(key),
            },
            modifiers,
        })
    }

    fn tab(modifiers: Modifiers) -> Option<Self> {
        (modifiers.intersects(Modifiers::CTRL | Modifiers::LOGO) && !modifiers.alt()).then_some(
            Self {
                key: ShortcutKey::Tab,
                modifiers,
            },
        )
    }

    pub(crate) fn from_event(key: &Key, modifiers: Modifiers) -> Option<Self> {
        if *key == Key::Named(Named::Tab) {
            return Self::tab(modifiers);
        }
        let Key::Character(value) = key else {
            return None;
        };
        let mut characters = value.chars();
        let character = characters.next()?;
        if characters.next().is_some() {
            return None;
        }
        Self::new(character, modifiers)
    }

    pub(crate) fn label(self) -> String {
        let mut label = String::new();
        if self.modifiers.logo() {
            label.push_str(if cfg!(target_os = "macos") {
                "Cmd+"
            } else {
                "Super+"
            });
        }
        if self.modifiers.control() {
            label.push_str("Ctrl+");
        }
        if self.modifiers.shift() {
            label.push_str("Shift+");
        }
        match self.key {
            ShortcutKey::Character(key) => label.push(key.to_ascii_uppercase()),
            ShortcutKey::Tab => label.push_str("Tab"),
            ShortcutKey::Plus => label.push('+'),
            ShortcutKey::Minus => label.push('-'),
        }
        label
    }

    fn encode(self) -> String {
        let mut value = String::new();
        if self.modifiers.logo() {
            value.push_str("meta+");
        }
        if self.modifiers.control() {
            value.push_str("ctrl+");
        }
        if self.modifiers.shift() {
            value.push_str("shift+");
        }
        match self.key {
            ShortcutKey::Character(key) => value.push(key),
            ShortcutKey::Tab => value.push_str("tab"),
            ShortcutKey::Plus => value.push_str("plus"),
            ShortcutKey::Minus => value.push_str("minus"),
        }
        value
    }

    fn decode(value: &str) -> Option<Self> {
        let (prefix, key) = value.rsplit_once('+')?;
        let mut modifiers = Modifiers::NONE;
        for part in prefix.split('+') {
            let modifier = match part {
                "meta" => Modifiers::LOGO,
                "ctrl" => Modifiers::CTRL,
                "shift" => Modifiers::SHIFT,
                _ => return None,
            };
            if modifiers.contains(modifier) {
                return None;
            }
            modifiers.insert(modifier);
        }
        if key == "tab" {
            return Self::tab(modifiers);
        }
        if key == "plus" {
            return Self::new('+', modifiers);
        }
        if key == "minus" {
            return Self::new('-', modifiers);
        }
        let mut characters = key.chars();
        let character = characters.next()?;
        if characters.next().is_some() {
            return None;
        }
        Self::new(character, modifiers)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Keybindings {
    shortcuts: [Option<Shortcut>; Slot::ALL.len()],
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            shortcuts: [
                Shortcut::new('n', Modifiers::COMMAND),
                Shortcut::new('o', Modifiers::COMMAND),
                Shortcut::new('s', Modifiers::COMMAND),
                if cfg!(target_os = "macos") {
                    Shortcut::new('s', Modifiers::CTRL)
                } else {
                    None
                },
                Shortcut::new('f', Modifiers::COMMAND),
                if cfg!(target_os = "macos") {
                    Shortcut::new('f', Modifiers::CTRL)
                } else {
                    None
                },
                Shortcut::new('o', Modifiers::COMMAND | Modifiers::SHIFT),
                Shortcut::new('p', Modifiers::COMMAND),
                Shortcut::new('t', Modifiers::COMMAND),
                Shortcut::new('w', Modifiers::COMMAND),
                Shortcut::tab(Modifiers::CTRL),
                Shortcut::tab(Modifiers::CTRL | Modifiers::SHIFT),
                Shortcut::new('l', Modifiers::CTRL | Modifiers::SHIFT),
                Shortcut::new('r', Modifiers::CTRL | Modifiers::SHIFT),
                Shortcut::new('d', Modifiers::CTRL | Modifiers::SHIFT),
                Shortcut::new('f', Modifiers::COMMAND | Modifiers::SHIFT),
                Shortcut::new('+', Modifiers::COMMAND),
                Shortcut::new('-', Modifiers::COMMAND),
                Shortcut::new(
                    ',',
                    if cfg!(target_os = "macos") {
                        Modifiers::COMMAND
                    } else {
                        Modifiers::CTRL
                    },
                ),
                Shortcut::new('j', Modifiers::COMMAND | Modifiers::SHIFT),
                Shortcut::new('y', Modifiers::COMMAND | Modifiers::SHIFT),
            ],
        }
    }
}

impl Keybindings {
    pub(crate) fn get(&self, slot: Slot) -> Option<Shortcut> {
        self.shortcuts[slot as usize]
    }

    pub(crate) fn matching(&self, key: &Key, modifiers: Modifiers) -> Option<Slot> {
        let shortcut = Shortcut::from_event(key, modifiers)?;
        Slot::ALL
            .into_iter()
            .find(|slot| self.get(*slot) == Some(shortcut))
    }

    pub(crate) fn assign(&mut self, slot: Slot, shortcut: Shortcut) -> Result<(), Slot> {
        if let Some(conflict) = Slot::ALL
            .into_iter()
            .find(|other| *other != slot && self.get(*other) == Some(shortcut))
        {
            return Err(conflict);
        }
        self.shortcuts[slot as usize] = Some(shortcut);
        Ok(())
    }

    pub(crate) fn load() -> io::Result<Self> {
        Self::load_from(&config_path()?)
    }

    fn load_from(path: &Path) -> io::Result<Self> {
        match fs::read_to_string(path) {
            Ok(contents) => Self::decode(&contents)
                .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "invalid keybindings file")),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn save(&self) -> io::Result<()> {
        self.save_to(&config_path()?)
    }

    fn save_to(&self, path: &Path) -> io::Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid keybindings path"))?;
        fs::create_dir_all(parent)?;
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, self.encode())?;
        fs::rename(temporary, path)
    }

    fn encode(&self) -> String {
        let mut contents = String::new();
        for slot in Slot::ALL {
            if let Some(shortcut) = self.get(slot) {
                contents.push_str(&shortcut.encode());
            }
            contents.push('\n');
        }
        contents
    }

    fn decode(contents: &str) -> Option<Self> {
        let lines: Vec<_> = contents.lines().collect();
        if !(4..=Slot::ALL.len()).contains(&lines.len()) {
            return None;
        }
        let mut bindings = Self {
            shortcuts: [None; Slot::ALL.len()],
        };
        let line_count = lines.len();
        for (slot, line) in Slot::ALL.into_iter().zip(lines) {
            if !line.is_empty() {
                bindings.assign(slot, Shortcut::decode(line)?).ok()?;
            }
        }
        if line_count < Slot::ALL.len() {
            let defaults = Self::default();
            if cfg!(target_os = "macos")
                && line_count < 8
                && bindings.get(Slot::Find) == Shortcut::new('f', Modifiers::CTRL)
            {
                bindings.shortcuts[Slot::Find as usize] = defaults.get(Slot::Find);
                bindings.shortcuts[Slot::AlternateFind as usize] =
                    Shortcut::new('f', Modifiers::CTRL);
            }
            for slot in Slot::ALL.into_iter().skip(line_count) {
                if bindings.get(slot).is_none()
                    && let Some(shortcut) = defaults.get(slot)
                    && !bindings.shortcuts.contains(&Some(shortcut))
                {
                    bindings.shortcuts[slot as usize] = Some(shortcut);
                }
            }
        }
        Some(bindings)
    }
}

pub(crate) fn config_dir() -> io::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = env::var_os("APPDATA").map(PathBuf::from);

    #[cfg(target_os = "macos")]
    let base = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Application Support"));

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config"))
        });

    base.map(|base| base.join("nia"))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "configuration directory unavailable"))
}

fn config_path() -> io::Result<PathBuf> {
    Ok(config_dir()?.join("keybindings.conf"))
}

#[cfg(test)]
mod tests;
