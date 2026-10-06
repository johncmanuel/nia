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
