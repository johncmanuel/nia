mod storage;

pub(crate) use storage::save_file;

use std::fs;
use std::io;
use std::path::PathBuf;

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
        self.mark_edited();
        self.sync_contents(contents);
    }

    pub(crate) fn mark_edited(&mut self) {
        self.revision += 1;
        self.dirty = true;
    }

    pub(crate) fn sync_contents(&mut self, contents: String) {
        self.word_count = contents.split_whitespace().count();
        self.contents = contents;
        self.dirty = self.saved_contents.as_deref().unwrap_or_default() != self.contents;
    }

    pub(crate) fn saved(&mut self, path: PathBuf, revision: u64, contents: String) {
        let edited_during_save = self.revision != revision;
        self.path = Some(path);
        self.saved_contents = Some(contents);
        self.revision += 1;
        self.dirty = edited_during_save && self.saved_contents.as_deref() != Some(&self.contents);
    }

    pub(crate) fn reload(&mut self, contents: String) {
        self.word_count = contents.split_whitespace().count();
        self.saved_contents = Some(contents.clone());
        self.contents = contents;
        self.revision += 1;
        self.dirty = false;
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
}
