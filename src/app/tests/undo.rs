use super::*;
use iced::widget::text_editor::{Action, Edit};

#[test]
fn shortcuts_undo_and_redo_edits_in_active_document() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.typewrite("hi");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.current().editor.text(), "hi");

    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("z".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.current().editor.text(), "");

    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("z".into()),
        Modifiers::COMMAND | Modifiers::SHIFT,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.current().editor.text(), "hi");
}

#[test]
fn history_is_per_document_and_undo_restores_clean_state() {
    let mut app = blank();
    let _ = update(&mut app, Message::Edit(0, Action::Edit(Edit::Insert('a'))));
    let _ = update(&mut app, Message::New);
    let second = app.current().id;
    let _ = update(
        &mut app,
        Message::Edit(second, Action::Edit(Edit::Insert('b'))),
    );
    let _ = update(&mut app, Message::Undo);
    assert_eq!(app.current().editor.text(), "");
    assert!(!app.current().document.dirty);
    let _ = update(&mut app, Message::SelectGroup(0));
    assert_eq!(app.current().editor.text(), "a");
    let _ = update(&mut app, Message::Undo);
    assert_eq!(app.current().editor.text(), "");
    assert!(!app.current().document.dirty);
}

#[test]
fn auto_pair_is_one_undo_step() {
    let mut app = blank();
    let _ = update(&mut app, Message::Edit(0, Action::Edit(Edit::Insert('('))));
    assert_eq!(app.current().editor.text(), "()");
    let _ = update(&mut app, Message::Undo);
    assert_eq!(app.current().editor.text(), "");
    let _ = update(&mut app, Message::Redo);
    assert_eq!(app.current().editor.text(), "()");
}

#[test]
fn formatting_and_tab_switches_break_typing_groups() {
    let mut app = blank();
    for character in "word".chars() {
        let _ = update(
            &mut app,
            Message::Edit(0, Action::Edit(Edit::Insert(character))),
        );
    }
    let _ = update(&mut app, Message::Format(super::Format::Bold));
    assert_eq!(app.current().editor.text(), "word****");
    let _ = update(&mut app, Message::Undo);
    assert_eq!(app.current().editor.text(), "word");
    let _ = update(&mut app, Message::New);
    let _ = update(&mut app, Message::SelectGroup(0));
    let _ = update(&mut app, Message::Edit(0, Action::Edit(Edit::Insert('!'))));
    let _ = update(&mut app, Message::Undo);
    assert_eq!(app.current().editor.text(), "word");
    let _ = update(&mut app, Message::Undo);
    assert_eq!(app.current().editor.text(), "");
}
