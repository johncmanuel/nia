use super::*;
use crate::document::save_file;
use std::io::{self, ErrorKind};

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
fn failed_recovery_write_stays_pending_and_retries_after_delay() {
    let mut app = blank();
    app.tabs[0].document.edit("unsaved".into());
    app.tabs[0].editor = text_editor::Content::with_text("unsaved");
    app.tabs[0].last_edit = Some(Instant::now() - Duration::from_secs(2));
    let _ = update(&mut app, Message::Tick(Instant::now()));
    assert!(app.current().recovering);
    let revision = app.current().document.revision;
    let _ = update(
        &mut app,
        Message::RecoveryWritten(
            0,
            revision,
            Err(io::Error::from(ErrorKind::StorageFull).into()),
        ),
    );
    assert!(!app.current().recovering);
    assert_ne!(app.current().recovered_revision, revision);
    assert!(
        app.notice
            .as_deref()
            .is_some_and(|notice| notice.contains("not backed up"))
    );
    let _ = update(&mut app, Message::Tick(Instant::now()));
    assert!(!app.current().recovering);
    app.tabs[0].last_recovery_failure = Some(Instant::now() - Duration::from_secs(6));
    let _ = update(&mut app, Message::Tick(Instant::now()));
    assert!(app.current().recovering);
}

#[test]
fn storage_full_save_preserves_edits_and_schedules_recovery() {
    let mut app = blank();
    let path = PathBuf::from("/tmp/nia-full-disk-test.md");
    app.tabs[0].document.path = Some(path.clone());
    app.tabs[0].document.saved_contents = Some("old".into());
    app.tabs[0].document.edit("new".into());
    app.tabs[0].editor = text_editor::Content::with_text("new");
    app.tabs[0].saving = true;
    let revision = app.current().document.revision;
    let _ = update(
        &mut app,
        Message::Saved(
            0,
            path,
            revision,
            String::new(),
            Err(io::Error::from(ErrorKind::StorageFull).into()),
        ),
    );
    assert_eq!(app.current().editor.text(), "new");
    assert!(app.current().document.dirty);
    assert!(!app.current().conflict);
    assert!(app.current().last_save_failure.is_some());
    let notice = app.notice.clone().expect("save failure notice");
    assert!(notice.contains("Storage is full"));
    let mut ui = simulator(crate::ui::view(&app));
    ui.find(notice.as_str())
        .expect("save failure is visible in the footer");
    drop(ui);
    let _ = update(&mut app, Message::ToggleZen);
    let mut ui = simulator(crate::ui::view(&app));
    ui.find(notice.as_str())
        .expect("save failure remains visible in Zen mode");
    drop(ui);
    let when = app.current().last_edit.expect("recovery remains scheduled");
    let _ = update(&mut app, Message::Tick(when + Duration::from_millis(600)));
    assert!(app.current().recovering);
    let _ = update(&mut app, Message::RecoveryWritten(0, revision, Ok(())));
    let _ = update(&mut app, Message::Tick(when + Duration::from_secs(1)));
    assert!(!app.current().saving, "autosave waits after a failure");
    let _ = update(&mut app, Message::Tick(when + Duration::from_secs(6)));
    assert!(app.current().saving, "autosave retries after the delay");
}

#[test]
fn save_conflict_requires_save_as_and_keeps_the_document() {
    let mut app = blank();
    let path = PathBuf::from("/tmp/nia-conflict-test.md");
    app.tabs[0].document.path = Some(path.clone());
    app.tabs[0].document.saved_contents = Some("old".into());
    app.tabs[0].document.edit("local".into());
    app.tabs[0].editor = text_editor::Content::with_text("local");
    app.tabs[0].saving = true;
    let revision = app.current().document.revision;
    let _ = update(
        &mut app,
        Message::Saved(
            0,
            path,
            revision,
            String::new(),
            Err(io::Error::from(ErrorKind::AlreadyExists).into()),
        ),
    );
    assert!(app.current().conflict);
    assert!(app.current().document.dirty);
    assert_eq!(app.current().editor.text(), "local");
    assert!(
        app.notice
            .as_deref()
            .is_some_and(|notice| notice.contains("Save As"))
    );
}

#[test]
fn external_file_errors_explain_the_problem_and_recovery_clears_conflict() {
    let mut app = blank();
    let path = PathBuf::from("/tmp/nia-external-kind-test.md");
    app.tabs[0].document.path = Some(path.clone());
    app.tabs[0].document.reload("original".into());
    let revision = app.current().document.revision;
    let _ = update(
        &mut app,
        Message::ExternalChecked(
            0,
            path.clone(),
            revision,
            Err(io::Error::from(ErrorKind::NotFound).into()),
        ),
    );
    assert!(app.current().conflict);
    assert!(
        app.notice
            .as_deref()
            .is_some_and(|notice| notice.contains("moved or deleted"))
    );
    let _ = update(
        &mut app,
        Message::ExternalChecked(
            0,
            path.clone(),
            revision,
            Err(io::Error::from(ErrorKind::PermissionDenied).into()),
        ),
    );
    assert!(
        app.notice
            .as_deref()
            .is_some_and(|notice| notice.contains("permission denied"))
    );
    let _ = update(
        &mut app,
        Message::ExternalChecked(0, path, revision, Ok("original".into())),
    );
    assert!(!app.current().conflict);
    assert_eq!(app.current().document.contents, "original");
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
