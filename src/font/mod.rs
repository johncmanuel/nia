use crate::keybindings::config_dir;
use iced::Font;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;
use std::sync::OnceLock;

static FAMILIES: OnceLock<Vec<String>> = OnceLock::new();

fn family_names() -> &'static [String] {
    FAMILIES.get_or_init(|| {
        let mut database = fontdb::Database::new();
        database.load_system_fonts();
        let mut families: Vec<_> = database
            .faces()
            .filter_map(|face| face.families.first().map(|(name, _)| name.clone()))
            .collect();
        families.sort_unstable_by_key(|name| name.to_lowercase());
        families.dedup();
        families
    })
}

pub(crate) fn installed_families() -> Vec<String> {
    family_names().to_vec()
}

pub(crate) fn font_for(family: &str) -> Font {
    family_names()
        .iter()
        .find(|name| name.as_str() == family)
        .map_or(Font::MONOSPACE, |name| Font::with_name(name.as_str()))
}

pub(crate) fn load() -> io::Result<Option<String>> {
    load_from(&config_dir()?.join("font.conf"))
}

fn load_from(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(value) => {
            let value = value.trim();
            if value.contains('\n') || value.contains('\r') {
                Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "invalid font setting",
                ))
            } else if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn save(family: Option<&str>) -> io::Result<()> {
    save_to(&config_dir()?.join("font.conf"), family)
}

fn save_to(path: &Path, family: Option<&str>) -> io::Result<()> {
    let directory = path
        .parent()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid font setting path"))?;
    fs::create_dir_all(directory)?;
    fs::write(path, family.unwrap_or(""))
}

#[cfg(test)]
mod tests;
