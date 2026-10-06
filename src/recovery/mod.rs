use crate::keybindings::config_dir;
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

pub(crate) fn new_key() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
    format!("{}-{nanos}-{sequence}", std::process::id())
}

fn directory() -> io::Result<PathBuf> {
    Ok(config_dir()?.join("recovery"))
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
    fs::write(&temporary, contents)?;
    fs::rename(temporary, path)
}

pub(crate) fn remove(key: &str) -> io::Result<()> {
    match fs::remove_file(path_in(&directory()?, key)?) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

pub(crate) fn load() -> io::Result<Vec<Draft>> {
    load_in(&directory()?)
}

fn load_in(directory: &Path) -> io::Result<Vec<Draft>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut drafts = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "md") {
            continue;
        }
        let Some(key) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if path_in(directory, key).is_err() {
            continue;
        }
        drafts.push(Draft {
            key: key.into(),
            contents: fs::read_to_string(path)?,
        });
    }
    drafts.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(drafts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_key_cannot_escape_directory() {
        assert!(path_in(Path::new("/tmp"), "../private").is_err());
        assert!(path_in(Path::new("/tmp"), &new_key()).is_ok());
    }

    #[test]
    fn written_draft_survives_reload() -> io::Result<()> {
        let directory = std::env::temp_dir().join(format!("nia-recovery-test-{}", new_key()));
        write_in(&directory, "draft-1", "Unsaved 🌍 markdown")?;
        let drafts = load_in(&directory)?;
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].key, "draft-1");
        assert_eq!(drafts[0].contents, "Unsaved 🌍 markdown");
        fs::remove_dir_all(directory)
    }
}
