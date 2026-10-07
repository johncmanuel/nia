use crate::config;
use iced::Theme;
use iced::theme::Base;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

pub(crate) fn choices(dark: bool) -> impl Iterator<Item = &'static Theme> {
    Theme::ALL
        .iter()
        .filter(move |theme| theme.extended_palette().is_dark == dark)
}

pub(crate) fn load() -> io::Result<Theme> {
    load_from(&config::directory()?.join("theme.conf"))
}

fn load_from(path: &Path) -> io::Result<Theme> {
    match fs::read_to_string(path) {
        Ok(value) => {
            let name = value.trim();
            // Keep settings written by the original four-theme picker working.
            let name = match name {
                "Catppuccin" => "Catppuccin Mocha",
                "Gruvbox" => "Gruvbox Dark",
                _ => name,
            };
            Theme::ALL
                .iter()
                .find(|theme| theme.name() == name)
                .cloned()
                .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "invalid theme setting"))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Theme::CatppuccinMocha),
        Err(error) => Err(error),
    }
}

pub(crate) fn save(theme: &Theme) -> io::Result<()> {
    save_to(&config::directory()?.join("theme.conf"), theme)
}

pub(crate) fn load_transparency() -> io::Result<bool> {
    load_transparency_from(&config::directory()?.join("transparent.conf"))
}

fn load_transparency_from(path: &Path) -> io::Result<bool> {
    match fs::read_to_string(path) {
        Ok(value) => match value.trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                "invalid transparency setting",
            )),
        },
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub(crate) fn save_transparency(enabled: bool) -> io::Result<()> {
    save_transparency_to(&config::directory()?.join("transparent.conf"), enabled)
}

fn save_transparency_to(path: &Path, enabled: bool) -> io::Result<()> {
    let directory = path
        .parent()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid transparency path"))?;
    fs::create_dir_all(directory)?;
    fs::write(path, enabled.to_string())
}

fn save_to(path: &Path, theme: &Theme) -> io::Result<()> {
    let directory = path
        .parent()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid theme path"))?;
    fs::create_dir_all(directory)?;
    fs::write(path, theme.name())
}

#[cfg(test)]
mod tests;
