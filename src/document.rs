use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub(crate) struct Document {
    pub(crate) path: Option<PathBuf>,
    pub(crate) contents: String,
    pub(crate) saved_contents: Option<String>,
    pub(crate) revision: u64,
    pub(crate) dirty: bool,
    pub(crate) word_count: usize,
}

impl Document {
    pub(crate) fn new() -> Self {
        Self {
            path: None,
            contents: String::new(),
            saved_contents: None,
            revision: 0,
            dirty: false,
            word_count: 0,
        }
    }

    pub(crate) fn open(path: PathBuf) -> io::Result<Self> {
        let path = fs::canonicalize(path)?;
        let contents = fs::read_to_string(&path)?;
        Ok(Self {
            word_count: contents.split_whitespace().count(),
            saved_contents: Some(contents.clone()),
            contents,
            path: Some(path),
            revision: 0,
            dirty: false,
        })
    }

    pub(crate) fn edit(&mut self, contents: String) {
        // count the whole document until large-file profiling calls for incremental stats.
        self.word_count = contents.split_whitespace().count();
        self.contents = contents;
        self.revision += 1;
        self.dirty = self.saved_contents.as_deref().unwrap_or_default() != self.contents;
    }

    pub(crate) fn saved(&mut self, path: PathBuf, revision: u64, contents: String) {
        self.path = Some(path);
        self.saved_contents = Some(contents);
        self.dirty =
            self.revision != revision && self.saved_contents.as_deref() != Some(&self.contents);
    }

    pub(crate) fn file_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .map_or_else(
                || "Untitled".into(),
                |name| name.to_string_lossy().into_owned(),
            )
    }
}

pub(crate) fn save_file(path: &Path, contents: &str, expected: Option<&str>) -> io::Result<()> {
    match expected {
        Some(original) => {
            if fs::read_to_string(path)? != original {
                return Err(io::Error::new(
                    ErrorKind::AlreadyExists,
                    "the file changed on disk; your edits are still in the editor",
                ));
            }
        }
        None if path.exists() => {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                "a file already exists at that path",
            ));
        }
        None => {}
    }

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidInput,
            "choose a filename for the document",
        )
    })?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let temporary = parent.join(format!(
        ".{}.nia-{}-{nonce}.tmp",
        name.to_string_lossy(),
        std::process::id()
    ));

    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        if expected.is_some() {
            file.set_permissions(fs::metadata(path)?.permissions())?;
        }
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        drop(file);
        if expected.is_some() {
            fs::rename(&temporary, path)
        } else {
            fs::hard_link(&temporary, path)
        }
    })();

    let _ = fs::remove_file(&temporary);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_edits_saves_and_word_count() {
        let mut document = Document::new();
        assert!(!document.dirty);
        assert_eq!(document.word_count, 0);

        document.edit("Hello, world!\nNew paragraph".into());
        assert!(document.dirty);
        assert_eq!(document.word_count, 4);
        let revision = document.revision;

        document.saved(
            PathBuf::from("article.md"),
            revision,
            document.contents.clone(),
        );
        assert!(!document.dirty);
        assert_eq!(document.file_name(), "article.md");

        document.edit("Hello, world!".into());
        assert!(document.dirty);
        assert_eq!(document.word_count, 2);
        document.edit("Hello, world!\nNew paragraph".into());
        assert!(!document.dirty);
    }

    #[test]
    fn saving_an_older_revision_keeps_new_edits_dirty() {
        let mut document = Document::new();
        document.edit("First".into());
        let saved_revision = document.revision;
        document.edit("Second".into());

        document.saved(PathBuf::from("article.md"), saved_revision, "First".into());
        assert!(document.dirty);
        assert_eq!(document.contents, "Second");
    }

    #[test]
    fn save_preserves_edits_and_rejects_external_changes() -> io::Result<()> {
        let dir = std::env::temp_dir().join(format!(
            "nia-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos()
        ));
        fs::create_dir(&dir)?;
        let path = dir.join("article.md");
        let result = (|| {
            save_file(&path, "Hello 🌍", None)?;
            assert_eq!(fs::read_to_string(&path)?, "Hello 🌍");
            assert!(save_file(&path, "Overwrite", None).is_err());
            assert_eq!(fs::read_to_string(&path)?, "Hello 🌍");
            save_file(&path, "Hello 🌍\nAgain", Some("Hello 🌍"))?;
            assert_eq!(fs::read_to_string(&path)?, "Hello 🌍\nAgain");
            fs::write(&path, "External edit")?;
            assert!(save_file(&path, "Local edit", Some("Hello 🌍\nAgain")).is_err());
            assert_eq!(fs::read_to_string(&path)?, "External edit");
            Ok(())
        })();
        fs::remove_dir_all(dir)?;
        result
    }
}
