use super::*;
use crate::document::save_file;

#[test]
fn external_changes_reload_clean_tabs_and_preserve_dirty_tabs() {
    let mut app = blank();
    let path = PathBuf::from("/tmp/nia-external-test.md");
    let mut document = Document::new();
    document.path = Some(path.clone());
    document.reload("disk original".into());
    app.tabs[0] = Tab::new(0, document);
    let revision = app.current().document.revision;
    let _ = update(
        &mut app,
        Message::ExternalChecked(0, path.clone(), revision, Ok("disk updated".into())),
    );
    assert_eq!(app.current().editor.text(), "disk updated");
    assert!(!app.current().document.dirty);

    app.current_mut().document.edit("local edits".into());
    let revision = app.current().document.revision;
    let _ = update(
        &mut app,
        Message::ExternalChecked(0, path, revision, Ok("other edits".into())),
    );
    assert_eq!(app.current().document.contents, "local edits");
    assert!(app.current().conflict);

    let stale_revision = app.current().document.revision;
    let saved_contents = app.current().document.contents.clone();
    app.current_mut().document.saved(
        PathBuf::from("/tmp/nia-external-test.md"),
        stale_revision,
        saved_contents,
    );
    let _ = update(
        &mut app,
        Message::ExternalChecked(
            0,
            PathBuf::from("/tmp/nia-external-test.md"),
            stale_revision,
            Ok("older disk read".into()),
        ),
    );
    assert_eq!(app.current().document.contents, "local edits");
}

#[test]
fn idle_edits_write_recovery_before_autosave() {
    let mut app = blank();
    app.tabs[0].document.path = Some(PathBuf::from("/tmp/nia-autosave-test.md"));
    app.tabs[0].document.saved_contents = Some("old".into());
    app.tabs[0].document.edit("new".into());
    app.tabs[0].editor = text_editor::Content::with_text("new");
    app.tabs[0].last_edit = Some(Instant::now() - Duration::from_secs(2));
    let _ = update(&mut app, Message::Tick(Instant::now()));
    assert!(app.current().recovering);
    assert!(!app.current().saving);

    let _ = update(&mut app, Message::RecoveryWritten(0, 1, Ok(())));
    let _ = update(&mut app, Message::Tick(Instant::now()));
    assert!(app.current().saving);
}

#[test]
fn live_edits_are_synced_before_saving_and_recovery() {
    let mut app = blank();
    let _ = update(
        &mut app,
        Message::Edit(0, text_editor::Action::Edit(text_editor::Edit::Insert('x'))),
    );
    assert_eq!(app.current().editor.text(), "x");
    assert!(app.current().document.dirty);
    let _save = app.start_save_at(0, PathBuf::from("/tmp/nia-sync-test.md"), true);
    assert_eq!(app.current().document.contents, "x");
    assert_eq!(app.current().document.word_count, 1);

    let mut app = blank();
    let _ = update(
        &mut app,
        Message::Edit(0, text_editor::Action::Edit(text_editor::Edit::Insert('y'))),
    );
    app.current_mut().last_edit = Some(Instant::now() - Duration::from_secs(1));
    let _recovery = update(&mut app, Message::Tick(Instant::now()));
    assert_eq!(app.current().document.contents, "y");
    assert!(app.current().recovering);
}

#[test]
fn opened_markdown_renders_and_survives_edit_save_reopen() -> std::io::Result<()> {
    let path = std::env::temp_dir().join(format!("nia-roundtrip-{}.md", recovery::new_key()));
    std::fs::write(&path, "# Heading\nBody 🌍")?;
    let mut app = blank();
    app.tabs[0] = Tab::new(0, Document::open(path.clone())?);
    if let Ok(preview) = std::env::var("NIA_OPENED_SNAPSHOT") {
        let mut ui = simulator(crate::ui::view(&app));
        let _ = ui
            .snapshot(&app.theme)
            .expect("render opened Markdown")
            .matches_image(preview)
            .expect("write opened preview");
    }

    let _ = update(
        &mut app,
        Message::Edit(0, text_editor::Action::Edit(text_editor::Edit::Insert('!'))),
    );
    assert!(app.current().document.dirty);
    assert!(app.current().editor.text().contains('!'));
    app.current_mut().sync_document();
    let revision = app.current().document.revision;
    let contents = app.current().document.contents.clone();
    assert!(contents.contains('!'));
    save_file(
        &path,
        &contents,
        app.current().document.saved_contents.as_deref(),
    )?;
    app.current_mut()
        .document
        .saved(path.clone(), revision, contents.clone());
    assert!(!app.current().document.dirty);

    let reopened = Document::open(path.clone())?;
    assert_eq!(reopened.contents, contents);
    std::fs::remove_file(path)
}

#[test]
fn save_failure_keeps_unsaved_text_in_the_editor() {
    let mut app = blank();
    app.current_mut()
        .document
        .edit("Important unsaved text".into());
    app.current_mut().saving = true;
    let revision = app.current().document.revision;
    let _ = update(
        &mut app,
        Message::Saved(
            0,
            PathBuf::from("/tmp/failure.md"),
            revision,
            String::new(),
            Err("disk unavailable".into()),
        ),
    );
    assert_eq!(app.current().document.contents, "Important unsaved text");
    assert!(app.current().document.dirty);
    assert!(!app.current().saving);
    assert!(
        app.notice
            .as_deref()
            .is_some_and(|notice| notice.contains("disk unavailable"))
    );
}
