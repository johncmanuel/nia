use crate::config;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_KEY: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Draft {
    pub(crate) key: String,
    pub(crate) contents: String,
}

pub(crate) struct LoadedDrafts {
    pub(crate) drafts: Vec<Draft>,
    pub(crate) skipped: usize,
}

pub(crate) fn new_key() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
    format!("{}-{nanos}-{sequence}", std::process::id())
}

fn directory() -> io::Result<PathBuf> {
    Ok(config::directory()?.join("recovery"))
}

fn path_in(directory: &Path, key: &str) -> io::Result<PathBuf> {
    if key.is_empty()
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "invalid recovery key",
        ));
    }
    Ok(directory.join(format!("{key}.md")))
}

pub(crate) fn write(key: &str, contents: &str) -> io::Result<()> {
    write_in(&directory()?, key, contents)
}

fn write_in(directory: &Path, key: &str, contents: &str) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    let path = path_in(directory, key)?;
    let temporary = path.with_extension("tmp");
    let result = fs::write(&temporary, contents).and_then(|()| fs::rename(&temporary, path));
    if result.is_err()
        && let Err(error) = fs::remove_file(temporary)
        && error.kind() != ErrorKind::NotFound
    {
        log::warn!("temporary recovery file cleanup failed: {:?}", error.kind());
    }
    result
}

pub(crate) fn remove(key: &str) -> io::Result<()> {
    match fs::remove_file(path_in(&directory()?, key)?) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

pub(crate) fn load() -> io::Result<LoadedDrafts> {
    load_in(&directory()?)
}

fn load_in(directory: &Path) -> io::Result<LoadedDrafts> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(LoadedDrafts {
                drafts: Vec::new(),
                skipped: 0,
            });
        }
        Err(error) => return Err(error),
    };
    let mut drafts = Vec::new();
    let mut skipped = 0;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                skipped += 1;
                log::warn!("recovery directory entry skipped: {:?}", error.kind());
                continue;
            }
        };
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "md") {
            continue;
        }
        let Some(key) = path.file_stem().and_then(|stem| stem.to_str()) else {
            skipped += 1;
            continue;
        };
        if path_in(directory, key).is_err() {
            skipped += 1;
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(contents) => drafts.push(Draft {
                key: key.into(),
                contents,
            }),
            Err(error) => {
                skipped += 1;
                log::warn!("recovery draft skipped: {:?}", error.kind());
            }
        }
    }
    drafts.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(LoadedDrafts { drafts, skipped })
}

#[cfg(test)]
mod tests;
