use crate::app::{CloseTarget, Message, blank, update};
use crate::document::Document;
use iced::widget::pane_grid;
use iced::widget::text_editor::{Action, Edit, Motion};
use iced_test::simulator;
use std::fs;

#[test]
fn reopening_a_file_creates_two_live_editors_with_independent_cursors() {
    let path = std::env::temp_dir().join(format!("nia-shared-{}.md", crate::recovery::new_key()));
    fs::write(&path, "same text").expect("test document");
    let mut app = blank();
    let first = Document::open(path.clone()).expect("open first view");
    let _ = update(&mut app, Message::Opened(Ok(first)));
    let second = Document::open(path.clone()).expect("open second view");
    let _ = update(&mut app, Message::Opened(Ok(second)));

    assert_eq!(app.panes.len(), 2);
    assert_eq!(app.tabs.len(), 2);
    assert_ne!(app.tabs[0].id, app.tabs[1].id);
    assert_eq!(app.tabs[0].document.path, app.tabs[1].document.path);
    let mut ui = simulator(crate::ui::view(&app));
    ui.find(iced::widget::Id::new("editor-0"))
        .expect("first view visible");
    ui.find(iced::widget::Id::new(super::super::EDITOR))
        .expect("second view visible");
    drop(ui);

    app.tabs[1]
        .editor
        .perform(Action::Move(Motion::DocumentEnd));
    let second_cursor = app.tabs[1].editor.cursor();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new("editor-0"))
        .expect("first editor accepts focus");
    ui.typewrite("!");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.tabs[0].editor.text(), app.tabs[1].editor.text());
    assert_eq!(app.tabs[1].editor.cursor(), second_cursor);
    assert!(app.tabs.iter().all(|tab| tab.document.dirty));

    let second_id = app.tabs[1].id;
    let _ = update(
        &mut app,
        Message::Edit(second_id, Action::Edit(Edit::Insert('?'))),
    );
    assert_eq!(app.tabs[0].editor.text(), app.tabs[1].editor.text());
    let _ = update(&mut app, Message::Undo);
    assert_eq!(app.tabs[0].editor.text(), app.tabs[1].editor.text());
    let _ = update(&mut app, Message::Redo);
    assert_eq!(app.tabs[0].editor.text(), app.tabs[1].editor.text());
    let _ = update(&mut app, Message::CloseTab(1));
    assert_eq!(app.pending_close, None);
    assert_eq!(app.tabs.len(), 1);
    assert!(app.tabs[0].document.dirty);

    fs::remove_file(path).expect("remove test document");
}

#[test]
fn saving_one_view_marks_both_clean_and_last_view_still_prompts() {
    let path = std::env::temp_dir().join(format!("nia-shared-{}.md", crate::recovery::new_key()));
    fs::write(&path, "before").expect("test document");
    let mut app = blank();
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file"))),
    );
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file again"))),
    );
    let second_id = app.tabs[1].id;
    let _ = update(
        &mut app,
        Message::Edit(second_id, Action::Edit(Edit::Insert('!'))),
    );
    let contents = app.tabs[0].editor.text();
    let revision = app.tabs[0].document.revision;
    let saved_path = app.tabs[0].document.path.clone().expect("canonical path");
    let _ = update(&mut app, Message::Save);
    assert!(app.tabs[0].saving, "only the primary view writes the file");
    assert!(!app.tabs[1].saving);
    let _ = update(
        &mut app,
        Message::Saved(0, saved_path, revision, contents, Ok(())),
    );
    assert!(app.tabs.iter().all(|tab| !tab.document.dirty));

    let _ = update(
        &mut app,
        Message::Edit(second_id, Action::Edit(Edit::Insert('?'))),
    );
    let _ = update(&mut app, Message::CloseTab(1));
    assert_eq!(app.pending_close, None);
    let _ = update(&mut app, Message::CloseTab(0));
    assert_eq!(app.pending_close, Some(CloseTarget::Tab(0)));

    fs::remove_file(path).expect("remove test document");
}

#[test]
fn opening_an_existing_file_in_an_empty_split_reuses_that_split() {
    let path = std::env::temp_dir().join(format!("nia-shared-{}.md", crate::recovery::new_key()));
    fs::write(&path, "hello").expect("test document");
    let mut app = blank();
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file"))),
    );
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    let second_id = app.current().id;
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file again"))),
    );
    assert_eq!(app.panes.len(), 2);
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.current().id, second_id);
    assert_eq!(app.tabs[0].document.path, app.tabs[1].document.path);
    fs::remove_file(path).expect("remove test document");
}

#[test]
fn closing_a_group_keeps_dirty_document_open_in_another_group() {
    let path = std::env::temp_dir().join(format!("nia-shared-{}.md", crate::recovery::new_key()));
    fs::write(&path, "hello").expect("test document");
    let mut app = blank();
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file"))),
    );
    let _ = update(&mut app, Message::NewGroup);
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file again"))),
    );
    let _ = update(&mut app, Message::Edit(0, Action::Edit(Edit::Insert('!'))));
    let text = app.tabs[0].editor.text();
    let _ = update(&mut app, Message::SelectGroup(0));
    let _ = update(&mut app, Message::CloseCurrentGroup);
    assert_eq!(app.pending_close, None);
    assert_eq!(app.groups.len(), 1);
    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.current().editor.text(), text);
    assert!(app.current().document.dirty);
    fs::remove_file(path).expect("remove test document");
}

#[test]
fn save_as_detaches_only_the_selected_view() {
    let path = std::env::temp_dir().join(format!("nia-shared-{}.md", crate::recovery::new_key()));
    let new_path = std::env::temp_dir().join(format!("nia-copy-{}.md", crate::recovery::new_key()));
    fs::write(&path, "hello").expect("test document");
    let mut app = blank();
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file"))),
    );
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file again"))),
    );
    let second_id = app.tabs[1].id;
    let _ = update(
        &mut app,
        Message::Edit(second_id, Action::Edit(Edit::Insert('!'))),
    );
    let revision = app.tabs[1].document.revision;
    let contents = app.tabs[1].editor.text();
    let _ = update(&mut app, Message::SaveAsPicked(Some(new_path.clone())));
    let _ = update(
        &mut app,
        Message::Saved(second_id, new_path.clone(), revision, contents, Ok(())),
    );
    assert_eq!(app.tabs[1].document.path.as_ref(), Some(&new_path));
    assert_ne!(app.tabs[0].document.path, app.tabs[1].document.path);
    assert!(app.tabs[0].document.dirty);
    assert!(!app.tabs[1].document.dirty);
    let copy = app.tabs[1].editor.text();
    let _ = update(&mut app, Message::Edit(0, Action::Edit(Edit::Insert('?'))));
    assert_eq!(app.tabs[1].editor.text(), copy);
    fs::remove_file(path).expect("remove test document");
}

#[test]
fn restored_layout_keeps_both_views_of_the_same_file() {
    let directory = std::env::temp_dir().join(format!("nia-shared-{}", crate::recovery::new_key()));
    fs::create_dir_all(&directory).expect("test directory");
    let path = directory.join("document.md");
    let layout = directory.join("layout.conf");
    fs::write(&path, "hello").expect("test document");
    let mut app = blank();
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file"))),
    );
    let _ = update(
        &mut app,
        Message::Opened(Ok(Document::open(path.clone()).expect("open file again"))),
    );
    super::super::session::save_to(&app, &layout).expect("save layout");
    let session = super::super::session::load_from(&layout)
        .expect("load layout")
        .expect("saved layout");
    let mut restored = blank();
    super::super::session::restore(&mut restored, session);
    assert_eq!(restored.panes.len(), 2);
    assert_eq!(
        restored.tabs[0].document.path,
        restored.tabs[1].document.path
    );
    let _ = update(
        &mut restored,
        Message::Edit(0, Action::Edit(Edit::Insert('!'))),
    );
    assert_eq!(
        restored.tabs[0].editor.text(),
        restored.tabs[1].editor.text()
    );
    fs::remove_dir_all(directory).expect("remove test directory");
}

#[test]
fn image_source_edits_update_the_other_view() {
    let path = std::env::temp_dir().join(format!("nia-shared-{}.md", crate::recovery::new_key()));
    fs::write(&path, "![alt](image.png)\n").expect("test document");
    let mut app = blank();
    for _ in 0..2 {
        let _ = update(
            &mut app,
            Message::Opened(Ok(Document::open(path.clone()).expect("open file"))),
        );
    }
    let second = app.tabs[1].id;
    let _ = update(
        &mut app,
        Message::Edit(second, Action::Edit(Edit::Insert('x'))),
    );
    assert_eq!(app.tabs[0].editor.text(), app.tabs[1].editor.text());
    fs::remove_file(path).expect("remove test document");
}

#[test]
fn external_reload_updates_each_view() {
    let path = std::env::temp_dir().join(format!("nia-shared-{}.md", crate::recovery::new_key()));
    fs::write(&path, "before").expect("test document");
    let mut app = blank();
    for _ in 0..2 {
        let _ = update(
            &mut app,
            Message::Opened(Ok(Document::open(path.clone()).expect("open file"))),
        );
    }
    let canonical = app.tabs[0].document.path.clone().expect("canonical path");
    let revision = app.tabs[0].document.revision;
    let _ = update(
        &mut app,
        Message::ExternalChecked(0, canonical, revision, Ok("after".into())),
    );
    assert!(app.tabs.iter().all(|tab| tab.editor.text() == "after"));
    assert!(app.tabs.iter().all(|tab| !tab.document.dirty));
    fs::remove_file(path).expect("remove test document");
}
