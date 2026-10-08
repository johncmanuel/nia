use iced::keyboard::{Key, Modifiers, key::Named};

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
    pub(super) fn new(key: char, modifiers: Modifiers) -> Option<Self> {
        let key = key.to_ascii_lowercase();
        if !key.is_ascii_alphanumeric() && !matches!(key, '+' | '-' | '=' | ',' | '[' | ']')
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

    pub(super) fn tab(modifiers: Modifiers) -> Option<Self> {
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

    pub(super) fn encode(self) -> String {
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

    pub(super) fn decode(value: &str) -> Option<Self> {
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
