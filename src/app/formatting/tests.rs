use super::*;
use iced::widget::text_editor::{Cursor, Position};

#[test]
fn every_insert_action_has_markdown_source() {
    for format in Format::ALL {
        let (source, _) = format.replacement(None);
        assert!(!source.is_empty(), "{} needs source text", format.label());
    }
    assert_eq!(Format::Bold.replacement(Some("word")).0, "**word**");
    assert_eq!(Format::Italic.replacement(Some("word")).0, "*word*");
    assert_eq!(Format::Code.replacement(Some("x")).0, "`x`");
    assert_eq!(
        Format::Strikethrough.replacement(Some("word")).0,
        "~~word~~"
    );
    assert_eq!(Format::Highlight.replacement(Some("word")).0, "==word==");
    assert_eq!(Format::Subscript.replacement(Some("2")).0, "~2~");
    assert_eq!(Format::Superscript.replacement(Some("2")).0, "^2^");
    assert_eq!(Format::Blockquote.replacement(Some("a\nb")).0, "> a\n> b");
    assert_eq!(
        Format::OrderedList.replacement(Some("a\nb")).0,
        "1. a\n2. b"
    );
    assert_eq!(
        Format::TaskList.replacement(Some("a\nb")).0,
        "- [ ] a\n- [ ] b"
    );
    assert_eq!(Format::Link.replacement(Some("title")).0, "[title](url)");
    assert_eq!(Format::Footnote.replacement(None).0, "[^1]\n\n[^1]: ");
    assert_eq!(
        Format::DefinitionList.replacement(Some("term")).0,
        "term\n: definition"
    );
}

#[test]
fn pairs_wrap_selection_and_leave_caret_inside() {
    let mut content = Content::with_text("alpha");
    content.move_to(Cursor {
        position: Position { line: 0, column: 5 },
        selection: Some(Position { line: 0, column: 0 }),
    });
    let action = Action::Edit(Edit::Insert('('));
    let auto = auto_edit(&content, &action).expect("opening parenthesis is paired");
    assert!(auto.perform(|action| content.perform(action)));
    assert_eq!(content.text(), "(alpha)");
    assert_eq!(content.cursor().position.column, 6);
}

#[test]
fn closing_and_backspacing_a_pair_do_not_leave_extra_characters() {
    let mut content = Content::with_text("()");
    content.move_to(Cursor {
        position: Position { line: 0, column: 1 },
        selection: None,
    });
    let close = Action::Edit(Edit::Insert(')'));
    let auto = auto_edit(&content, &close).expect("closer is skipped");
    assert!(!auto.perform(|action| content.perform(action)));
    assert_eq!(content.text(), "()");
    assert_eq!(content.cursor().position.column, 2);

    content.perform(Action::Move(Motion::Left));
    let backspace = Action::Edit(Edit::Backspace);
    let auto = auto_edit(&content, &backspace).expect("empty pair is deleted together");
    assert!(auto.perform(|action| content.perform(action)));
    assert_eq!(content.text(), "");
}

#[test]
fn apostrophes_inside_words_are_not_paired() {
    for word in ["don", "café"] {
        let mut content = Content::with_text(word);
        content.move_to(Cursor {
            position: Position {
                line: 0,
                column: word.len(),
            },
            selection: None,
        });
        let action = Action::Edit(Edit::Insert('\''));
        assert!(auto_edit(&content, &action).is_none());
    }
}

#[test]
fn bracket_quote_and_code_pairs_are_available() {
    for (open, expected) in [
        ('[', "[]"),
        ('{', "{}"),
        ('<', "<>"),
        ('"', "\"\""),
        ('\'', "''"),
        ('`', "``"),
    ] {
        let mut content = Content::new();
        let action = Action::Edit(Edit::Insert(open));
        auto_edit(&content, &action)
            .expect("opening character has a pair")
            .perform(|action| content.perform(action));
        assert_eq!(content.text(), expected);
        assert_eq!(content.cursor().position.column, 1);
    }
}

#[test]
fn footnote_labels_do_not_reuse_existing_numbers() {
    assert_eq!(next_footnote_label("text [^1]\n[^1]: note"), 2);
    assert_eq!(next_footnote_label("text [^1] and [^3]"), 2);
}
