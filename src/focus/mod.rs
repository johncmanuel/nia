use crate::config;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

const DEFAULT_INACTIVE_PANE_DIM: u8 = 35;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) highlight_line: bool,
    pub(crate) inactive_pane_dim: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            highlight_line: false,
            inactive_pane_dim: DEFAULT_INACTIVE_PANE_DIM,
        }
    }
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
        let (highlight, dim) = match parts.as_slice() {
            [highlight] => (*highlight, DEFAULT_INACTIVE_PANE_DIM),
            [highlight, "Off" | "Paragraph" | "Sentence"] => {
                (*highlight, DEFAULT_INACTIVE_PANE_DIM)
            }
            [highlight, dim] => (
                *highlight,
                dim.parse::<u8>()
                    .ok()
                    .filter(|dim| *dim <= 100)
                    .ok_or_else(invalid)?,
            ),
            [_, highlight, _, _, _] => (*highlight, DEFAULT_INACTIVE_PANE_DIM),
            _ => return Err(invalid()),
        };
        let highlight_line = match highlight {
            "true" => true,
            "false" => false,
            _ => return Err(invalid()),
        };
        Ok(Self {
            highlight_line,
            inactive_pane_dim: dim,
        })
    }

    pub(crate) fn save(self) -> io::Result<()> {
        self.save_to(&config::directory()?.join("focus.conf"))
    }

    fn save_to(self, path: &Path) -> io::Result<()> {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(ErrorKind::InvalidInput, "invalid focus settings path")
        })?;
        fs::create_dir_all(parent)?;
        fs::write(
            path,
            format!("{}\n{}\n", self.highlight_line, self.inactive_pane_dim),
        )
    }
}

#[cfg(test)]
mod tests;
