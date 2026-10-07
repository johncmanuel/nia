use crate::config;
use iced::keyboard::{Key, Modifiers};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

mod shortcut;
mod slot;

pub(crate) use shortcut::Shortcut;
pub(crate) use slot::Slot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Keybindings {
    shortcuts: [Option<Shortcut>; Slot::ALL.len()],
}

impl Default for Keybindings {
    fn default() -> Self {
        // Iced maps COMMAND to Command on macOS and Control on Windows/Linux.
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
                Shortcut::new(',', Modifiers::COMMAND),
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

fn config_path() -> io::Result<PathBuf> {
    Ok(config::directory()?.join("keybindings.conf"))
}

#[cfg(test)]
mod tests;
