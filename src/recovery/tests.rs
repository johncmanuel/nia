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
    assert_eq!(drafts.skipped, 0);
    assert_eq!(drafts.drafts.len(), 1);
    assert_eq!(drafts.drafts[0].key, "draft-1");
    assert_eq!(drafts.drafts[0].contents, "Unsaved 🌍 markdown");
    fs::remove_dir_all(directory)
}

#[test]
fn failed_recovery_rename_removes_temporary_file() -> io::Result<()> {
    let directory = std::env::temp_dir().join(format!("nia-recovery-test-{}", new_key()));
    fs::create_dir_all(directory.join("draft-1.md"))?;
    assert!(write_in(&directory, "draft-1", "Unsaved text").is_err());
    assert!(!directory.join("draft-1.tmp").exists());
    fs::remove_dir_all(directory)
}

#[test]
fn unreadable_draft_does_not_hide_other_drafts() -> io::Result<()> {
    let directory = std::env::temp_dir().join(format!("nia-recovery-test-{}", new_key()));
    write_in(&directory, "good", "Important text")?;
    fs::write(directory.join("bad.md"), [0xff])?;
    let loaded = load_in(&directory)?;
    assert_eq!(loaded.skipped, 1);
    assert_eq!(loaded.drafts.len(), 1);
    assert_eq!(loaded.drafts[0].contents, "Important text");
    fs::remove_dir_all(directory)
}
