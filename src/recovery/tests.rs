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
