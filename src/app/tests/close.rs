use super::*;

fn edit_current(app: &mut App, text: &str) {
    app.current_mut().document.edit(text.into());
    app.current_mut().editor = text_editor::Content::with_text(text);
}

#[test]
fn dirty_tab_prompts_and_cancel_keeps_edits() {
    let mut app = blank();
    edit_current(&mut app, "Keep this draft");
    let _ = update(&mut app, Message::CloseTab(0));
    assert_eq!(app.pending_close, Some(CloseTarget::Tab(0)));

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Cancel").expect("cancel button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.pending_close, None);
    assert_eq!(app.current().document.contents, "Keep this draft");
    assert!(app.current().document.dirty);
}

#[test]
fn save_and_close_waits_for_successful_save() {
    let mut app = blank();
    let _ = update(&mut app, Message::New);
    let path = PathBuf::from("/tmp/nia-close-test.md");
    app.current_mut().document.path = Some(path.clone());
    app.current_mut().document.saved_contents = Some("old".into());
    edit_current(&mut app, "new");
    let id = app.current().id;
    let revision = app.current().document.revision;
    let active = app.active;
    let _ = update(&mut app, Message::CloseTab(active));

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Save and Close")
        .expect("save and close button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.tabs.iter().any(|tab| tab.id == id && tab.saving));
    assert_eq!(app.tabs.len(), 2);

    let _ = update(
        &mut app,
        Message::Saved(id, path, revision, "new".into(), Ok(())),
    );
    assert_eq!(app.tabs.len(), 1);
    assert!(app.tabs.iter().all(|tab| tab.id != id));
}

#[test]
fn failed_save_does_not_close_dirty_tab() {
    let mut app = blank();
    edit_current(&mut app, "unsaved");
    let path = PathBuf::from("/tmp/nia-close-failure.md");
    app.current_mut().document.path = Some(path.clone());
    app.current_mut().document.saved_contents = Some("old".into());
    let revision = app.current().document.revision;
    let _ = update(&mut app, Message::CloseTab(0));
    let _ = update(&mut app, Message::SaveAndClose);
    let _ = update(
        &mut app,
        Message::Saved(0, path, revision, String::new(), Err("disk full".into())),
    );
    assert_eq!(app.tabs.len(), 1);
    assert!(app.current().document.dirty);
    assert_eq!(app.confirmed_close, None);
    assert!(
        app.notice
            .as_deref()
            .is_some_and(|text| text.contains("disk full"))
    );
}

#[test]
fn discard_closes_only_the_selected_dirty_tab() {
    let mut app = blank();
    edit_current(&mut app, "first");
    let _ = update(&mut app, Message::New);
    edit_current(&mut app, "second");
    let _ = update(&mut app, Message::CloseTab(0));

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Discard and Close")
        .expect("discard and close button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.pending_close, None);
    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.current().document.contents, "second");
    assert!(app.current().document.dirty);
}

#[test]
fn quitting_with_unsaved_documents_prompts() {
    let mut app = blank();
    edit_current(&mut app, "draft");
    let id = iced::window::Id::unique();
    let _ = update(&mut app, Message::CloseRequested(id));
    assert_eq!(app.pending_close, Some(CloseTarget::Window(id)));

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Discard and Quit")
        .expect("discard and quit button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.pending_close, None);
}

#[test]
fn discard_waits_for_recovery_write() {
    let mut app = blank();
    edit_current(&mut app, "unsaved");
    app.current_mut().recovering = true;
    let _ = update(&mut app, Message::CloseTab(0));
    let _ = update(&mut app, Message::DiscardAndClose);
    assert_eq!(app.pending_close, Some(CloseTarget::Tab(0)));
    assert_eq!(app.current().document.contents, "unsaved");
}
