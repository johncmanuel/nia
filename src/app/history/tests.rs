use super::*;
use std::sync::Arc;

fn edit(history: &mut History, content: &mut Content, action: Action) {
    let pending = history.begin(content, &action);
    content.perform(action);
    history.finish(content, pending);
}

fn edit_at(history: &mut History, content: &mut Content, action: Action, at: Instant) {
    let pending = history.begin(content, &action);
    content.perform(action);
    history.finish_at(content, pending, at);
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

#[test]
fn typing_undoes_by_word_and_punctuation_boundary() {
    let mut content = Content::new();
    let mut history = History::default();
    let start = Instant::now();
    for character in "hello world!".chars() {
        edit_at(
            &mut history,
            &mut content,
            Action::Edit(Edit::Insert(character)),
            start,
        );
    }
    assert_eq!(history.undo.len(), 2);
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "hello ");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "");
    assert!(history.redo(&mut content));
    assert!(history.redo(&mut content));
    assert_eq!(content.text(), "hello world!");
}

#[test]
fn pauses_and_navigation_start_new_undo_units() {
    let mut content = Content::new();
    let mut history = History::default();
    let start = Instant::now();
    for (character, at) in [
        ('a', start),
        ('b', start + Duration::from_millis(100)),
        ('c', start + GROUP_PAUSE + Duration::from_millis(200)),
    ] {
        edit_at(
            &mut history,
            &mut content,
            Action::Edit(Edit::Insert(character)),
            at,
        );
    }
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "ab");
    edit(
        &mut history,
        &mut content,
        Action::Move(iced::widget::text_editor::Motion::Left),
    );
    edit(
        &mut history,
        &mut content,
        Action::Move(iced::widget::text_editor::Motion::Right),
    );
    edit(&mut history, &mut content, Action::Edit(Edit::Insert('x')));
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "ab");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "");
}

#[test]
fn backspaces_group_separately_and_stop_at_word_boundary() {
    let mut content = Content::with_text("one two");
    let mut history = History::default();
    content.move_to(Cursor {
        position: Position { line: 0, column: 7 },
        selection: None,
    });
    for _ in 0..4 {
        edit(&mut history, &mut content, Action::Edit(Edit::Backspace));
    }
    assert_eq!(content.text(), "one");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "one two");
    assert!(!history.undo(&mut content));
}

#[test]
fn selection_replacement_and_paste_stay_atomic() {
    let mut content = Content::with_text("hello");
    let mut history = History::default();
    content.move_to(Cursor {
        position: Position { line: 0, column: 5 },
        selection: Some(Position { line: 0, column: 0 }),
    });
    edit(&mut history, &mut content, Action::Edit(Edit::Insert('X')));
    edit(&mut history, &mut content, Action::Edit(Edit::Insert('y')));
    edit(
        &mut history,
        &mut content,
        Action::Edit(Edit::Paste(Arc::new("!".into()))),
    );
    assert_eq!(content.text(), "Xy!");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "Xy");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "X");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "hello");
}

#[test]
fn forward_deletes_group_and_new_typing_after_undo_is_separate() {
    let mut content = Content::with_text("abcdef");
    let mut history = History::default();
    content.move_to(Cursor {
        position: Position { line: 0, column: 0 },
        selection: None,
    });
    for _ in 0..3 {
        edit(&mut history, &mut content, Action::Edit(Edit::Delete));
    }
    assert_eq!(content.text(), "def");
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "abcdef");
    edit(&mut history, &mut content, Action::Edit(Edit::Insert('X')));
    assert_eq!(content.text(), "Xabcdef");
    assert!(!history.redo(&mut content));
    assert!(history.undo(&mut content));
    assert_eq!(content.text(), "abcdef");
}
