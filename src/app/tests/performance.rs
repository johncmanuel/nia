use super::*;

#[test]
fn long_unicode_document_remains_editable() {
    let contents = "one 🌍 two\n".repeat(20_000);
    let mut tab = Tab::new(0, Document::new());
    tab.editor = text_editor::Content::with_text(&contents);
    tab.editor.move_to(text_editor::Cursor {
        position: text_editor::Position {
            line: 10_000,
            column: 0,
        },
        selection: None,
    });
    let start = Instant::now();
    tab.editor
        .perform(text_editor::Action::Edit(text_editor::Edit::Insert('!')));
    tab.document.edit(tab.editor.text());
    eprintln!("long Unicode edit: {:?}", start.elapsed());
    assert_eq!(tab.document.word_count, 60_000);
    assert!(tab.document.contents.contains("!one 🌍 two"));
}

#[test]
#[ignore = "local debug-build performance check"]
fn large_document_edit_update_fits_frame_budget() {
    let mut app = blank();
    let contents = "one 🌍 two\n".repeat(20_000);
    app.tabs[0].editor = text_editor::Content::with_text(&contents);
    app.tabs[0].document.edit(contents);
    let start = Instant::now();
    let _ = update(
        &mut app,
        Message::Edit(0, text_editor::Action::Edit(text_editor::Edit::Insert('x'))),
    );
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(16),
        "keystroke exceeded a 60 Hz frame: {elapsed:?}"
    );
}
