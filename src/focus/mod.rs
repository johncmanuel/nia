use crate::config;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) highlight_line: bool,
}

impl Settings {
    pub(crate) fn load() -> io::Result<Self> {
        Self::load_from(&config::directory()?.join("focus.conf"))
    }

    fn load_from(path: &Path) -> io::Result<Self> {
        let value = match fs::read_to_string(path) {
            Ok(value) => value,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let invalid = || io::Error::new(ErrorKind::InvalidData, "invalid focus settings");
        let parts: Vec<_> = value.lines().collect();
        let highlight = match parts.as_slice() {
            [highlight] | [highlight, _] | [_, highlight, _, _, _] => *highlight,
            _ => return Err(invalid()),
        };
        let highlight_line = match highlight {
            "true" => true,
            "false" => false,
            _ => return Err(invalid()),
        };
        Ok(Self { highlight_line })
    }

    pub(crate) fn save(self) -> io::Result<()> {
        self.save_to(&config::directory()?.join("focus.conf"))
    }

    fn save_to(self, path: &Path) -> io::Result<()> {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(ErrorKind::InvalidInput, "invalid focus settings path")
        })?;
        fs::create_dir_all(parent)?;
        fs::write(path, format!("{}\n", self.highlight_line))
    }
}

#[cfg(test)]
mod tests;
