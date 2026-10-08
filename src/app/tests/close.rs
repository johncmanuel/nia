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
fn closing_group_prompts_for_all_dirty_splits_and_keeps_other_groups() {
    let mut app = blank();
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    let first_group = app.groups[0].tabs.clone();
    for (index, id) in first_group.iter().enumerate() {
        let tab = app
            .tabs
            .iter_mut()
            .find(|tab| tab.id == *id)
            .expect("split tab");
        tab.editor = text_editor::Content::with_text(&format!("draft {index}"));
        tab.document.mark_edited();
    }
    let _ = update(&mut app, Message::NewGroup);
    let other_group = app.current().id;
    let _ = update(&mut app, Message::SelectGroup(0));
    let _ = update(&mut app, Message::CloseCurrentGroup);
    assert_eq!(app.pending_close, Some(CloseTarget::Group(first_group[0])));
    let _ = update(&mut app, Message::CancelClose);
    assert_eq!(app.groups.len(), 2);
    assert_eq!(app.tabs.len(), 3);
    let _ = update(&mut app, Message::CloseCurrentGroup);
    let _ = update(&mut app, Message::DiscardAndClose);
    assert_eq!(app.groups.len(), 1);
    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.current().id, other_group);
}

#[test]
fn closing_last_group_leaves_a_blank_document() {
    let mut app = blank();
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Horizontal));
    let _ = update(&mut app, Message::CloseCurrentGroup);
    assert_eq!(app.groups.len(), 1);
    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.panes.len(), 1);
    assert!(app.current().document.contents.is_empty());
}

#[test]
fn save_and_close_group_saves_each_dirty_document_before_removing_it() {
    let mut app = blank();
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    let ids = app.groups[0].tabs.clone();
    for (index, id) in ids.iter().enumerate() {
        let tab = app
            .tabs
            .iter_mut()
            .find(|tab| tab.id == *id)
            .expect("split tab");
        tab.editor = text_editor::Content::with_text(&format!("new {index}"));
        tab.document.path = Some(PathBuf::from(format!("/tmp/nia-group-{index}.md")));
        tab.document.saved_contents = Some(format!("old {index}"));
        tab.document.mark_edited();
    }
    let _ = update(&mut app, Message::NewGroup);
    let remaining = app.current().id;
    let _ = update(&mut app, Message::SelectGroup(0));
    let _ = update(&mut app, Message::CloseCurrentGroup);
    let _ = update(&mut app, Message::SaveAndClose);
    assert!(app.tabs[0].saving);
    assert!(!app.tabs[1].saving);
    let first_revision = app.tabs[0].document.revision;
    let _ = update(
        &mut app,
        Message::Saved(
            ids[0],
            PathBuf::from("/tmp/nia-group-0.md"),
            first_revision,
            "new 0".into(),
            Ok(()),
        ),
    );
    assert!(app.tabs[1].saving);
    let second_revision = app.tabs[1].document.revision;
    let _ = update(
        &mut app,
        Message::Saved(
            ids[1],
            PathBuf::from("/tmp/nia-group-1.md"),
            second_revision,
            "new 1".into(),
            Ok(()),
        ),
    );
    assert_eq!(app.groups.len(), 1);
    assert_eq!(app.current().id, remaining);
    assert!(app.tabs.iter().all(|tab| !ids.contains(&tab.id)));
}

#[test]
fn command_shift_w_closes_active_group_from_editor() {
    let mut app = blank();
    app.keybindings = crate::keybindings::Keybindings::default();
    let _ = update(&mut app, Message::NewGroup);
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("w".into()),
        Modifiers::COMMAND | Modifiers::SHIFT,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.groups.len(), 1);
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
