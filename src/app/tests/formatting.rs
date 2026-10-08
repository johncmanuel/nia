use super::*;
use iced::widget::text_editor::{Action, Cursor, Edit, Position};

#[test]
fn bold_shortcut_reaches_editor_and_wraps_selection() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("b".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.current().editor.text(), "****");
    assert_eq!(app.current().editor.cursor().position.column, 2);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("i".into()),
        Modifiers::COMMAND,
    )]);
    assert!(
        ui.into_messages()
            .into_iter()
            .any(|message| matches!(message, Message::Format(Format::Italic)))
    );

    app.current_mut().editor = text_editor::Content::with_text("word");
    app.current_mut().editor.move_to(Cursor {
        position: Position { line: 0, column: 4 },
        selection: Some(Position { line: 0, column: 0 }),
    });
    let _ = update(&mut app, Message::Format(Format::Italic));
    assert_eq!(app.current().editor.text(), "*word*");
    assert!(app.current().document.dirty);
}

#[test]
fn insert_menu_exposes_all_formats_and_inserts_table() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Insert").expect("Insert control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.show_format_menu);
    let mut ui = simulator(crate::ui::view(&app));
    for format in Format::ALL {
        ui.find(format.label()).expect("format listed in menu");
    }
    ui.click("Table").expect("Table action visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(!app.show_format_menu);
    assert!(app.current().editor.text().contains("| --- | --- |"));
}

#[test]
fn insert_menu_wraps_selected_text_without_losing_selection() {
    let mut app = blank();
    app.current_mut().editor = text_editor::Content::with_text("word");
    app.current_mut().editor.move_to(Cursor {
        position: Position { line: 0, column: 4 },
        selection: Some(Position { line: 0, column: 0 }),
    });
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Insert").expect("Insert control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Bold").expect("Bold action visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.current().editor.text(), "**word**");
}

#[test]
fn typed_pairs_skip_closers_and_delete_together() {
    let mut app = blank();
    for character in ['(', ')'] {
        let _ = update(
            &mut app,
            Message::Edit(0, Action::Edit(Edit::Insert(character))),
        );
    }
    assert_eq!(app.current().editor.text(), "()");
    assert_eq!(app.current().editor.cursor().position.column, 2);
    app.current_mut()
        .editor
        .perform(Action::Move(text_editor::Motion::Left));
    let _ = update(&mut app, Message::Edit(0, Action::Edit(Edit::Backspace)));
    assert_eq!(app.current().editor.text(), "");
}

#[test]
fn formatting_and_pairs_keep_image_blocks_in_sync() {
    let mut document = Document::new();
    document.edit("![photo](image.png)\nAfter".into());
    let mut app = blank();
    app.tabs[0] = Tab::new(0, document);
    app.tabs[0].move_image_cursor(Cursor {
        position: Position { line: 1, column: 5 },
        selection: None,
    });
    let block = app.tabs[0].active_image_block;
    let _ = update(&mut app, Message::Format(Format::Bold));
    assert_eq!(
        app.current().editor.text(),
        "![photo](image.png)\nAfter****"
    );
    assert_eq!(
        app.current().image_blocks[block].content.text(),
        "After****"
    );
    let _ = update(
        &mut app,
        Message::EditImageBlock(0, block, Action::Edit(Edit::Insert('('))),
    );
    assert_eq!(
        app.current().editor.text(),
        "![photo](image.png)\nAfter**()**"
    );
    assert_eq!(
        app.current().image_blocks[block].content.text(),
        "After**()**"
    );
}

#[test]
fn formatting_selected_text_after_an_image_keeps_source_and_preview_in_sync() {
    let mut document = Document::new();
    document.edit("![photo](image.png)\nAfter".into());
    let mut app = blank();
    app.tabs[0] = Tab::new(0, document);
    app.tabs[0].move_image_cursor(Cursor {
        position: Position { line: 1, column: 5 },
        selection: Some(Position { line: 1, column: 0 }),
    });
    let block = app.tabs[0].active_image_block;
    let _ = update(&mut app, Message::Format(Format::Bold));
    assert_eq!(
        app.current().editor.text(),
        "![photo](image.png)\n**After**"
    );
    assert_eq!(
        app.current().image_blocks[block].content.text(),
        "**After**"
    );
}

#[test]
fn block_formatting_stays_on_its_own_line_and_preserves_selected_text() {
    let mut app = blank();
    app.current_mut().editor = text_editor::Content::with_text("beforeafter");
    app.current_mut().editor.move_to(Cursor {
        position: Position { line: 0, column: 6 },
        selection: None,
    });
    let _ = update(&mut app, Message::Format(Format::Blockquote));
    assert_eq!(app.current().editor.text(), "before\n> \nafter");

    app.current_mut().editor = text_editor::Content::with_text("first\nsecond");
    app.current_mut().editor.move_to(Cursor {
        position: Position { line: 1, column: 6 },
        selection: Some(Position { line: 0, column: 0 }),
    });
    let _ = update(&mut app, Message::Format(Format::Table));
    assert!(
        app.current()
            .editor
            .text()
            .contains("| first |  |\n| second |  |")
    );
}

#[test]
fn repeated_footnotes_use_distinct_labels() {
    let mut app = blank();
    let _ = update(&mut app, Message::Format(Format::Footnote));
    let _ = update(&mut app, Message::Format(Format::Footnote));
    assert!(app.current().editor.text().contains("[^1]: "));
    assert!(app.current().editor.text().contains("[^2]: "));
}
