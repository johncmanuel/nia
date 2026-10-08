use super::*;
use std::sync::Arc;

fn edit(history: &mut History, content: &mut Content, action: Action) {
    let pending = History::begin(content, &action);
    content.perform(action);
    history.finish(content, pending);
}

#[test]
fn undo_redo_restores_text_and_cursor() {
    let mut content = Content::with_text("first\nsecond");
    let mut history = History::default();
    content.move_to(Cursor {
        position: Position { line: 1, column: 6 },
        selection: None,
    });
    edit(&mut history, &mut content, Action::Edit(Edit::Insert('!')));
    assert_eq!(content.text(), "first\nsecond!");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "first\nsecond");
    assert_eq!(content.cursor().position, Position { line: 1, column: 6 });
    assert!(history.redo(&mut content));
    assert_eq!(content.text(), "first\nsecond!");
    assert_eq!(content.cursor().position, Position { line: 1, column: 7 });
}

#[test]
fn multiline_selection_and_new_edit_clear_redo() {
    let mut content = Content::with_text("one\ntwo\nthree");
    let mut history = History::default();
    content.move_to(Cursor {
        position: Position { line: 2, column: 2 },
        selection: Some(Position { line: 0, column: 1 }),
    });
    edit(
        &mut history,
        &mut content,
        Action::Edit(Edit::Paste(Arc::new("é\nhi".into()))),
    );
    assert_eq!(content.text(), "oé\nhiree");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "one\ntwo\nthree");
    assert!(history.redo(&mut content));
    assert_eq!(content.text(), "oé\nhiree");
    assert!(history.undo(&mut content));
    edit(&mut history, &mut content, Action::Edit(Edit::Insert('!')));
    assert!(!history.redo(&mut content));
}

#[test]
fn deleting_newline_restores_both_lines() {
    let mut content = Content::with_text("left\nright\nlast");
    let mut history = History::default();
    content.move_to(Cursor {
        position: Position { line: 1, column: 0 },
        selection: None,
    });
    edit(&mut history, &mut content, Action::Edit(Edit::Backspace));
    assert_eq!(content.text(), "leftright\nlast");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "left\nright\nlast");
    assert!(history.redo(&mut content));
    assert_eq!(content.text(), "leftright\nlast");
}

#[test]
fn line_boundary_edits_round_trip() {
    for (source, cursor, action, edited) in [
        (
            "left\nright",
            Position { line: 0, column: 4 },
            Edit::Delete,
            "leftright",
        ),
        (
            "first\nsecond",
            Position { line: 1, column: 6 },
            Edit::Enter,
            "first\nsecond\n",
        ),
        (
            "first\nsecond",
            Position { line: 0, column: 0 },
            Edit::Paste(Arc::new("new\n".into())),
            "new\nfirst\nsecond",
        ),
    ] {
        let mut content = Content::with_text(source);
        let mut history = History::default();
        content.move_to(Cursor {
            position: cursor,
            selection: None,
        });
        edit(&mut history, &mut content, Action::Edit(action));
        assert_eq!(content.text(), edited);
        assert!(history.undo(&mut content));
        assert_eq!(content.text(), source);
        assert!(history.redo(&mut content));
        assert_eq!(content.text(), edited);
    }
}
