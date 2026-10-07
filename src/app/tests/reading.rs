use super::*;

#[test]
fn reading_mode_renders_unsaved_text_and_restores_the_editor() {
    let mut app = blank();
    app.current_mut().editor =
        text_editor::Content::with_text("# Unsaved heading\nA **bold** line.");
    app.current_mut().document.mark_edited();
    let source = app.current().editor.text();
    let cursor = app.current().editor.cursor();

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Read").expect("reading control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.current().reading.is_some());
    let items = app.current().reading.as_ref().unwrap().items();
    assert!(matches!(
        items.first(),
        Some(iced::widget::markdown::Item::Heading(..))
    ));
    assert!(format!("{items:?}").contains("Unsaved heading"));
    let mut ui = simulator(crate::ui::view(&app));
    let snapshot = ui.snapshot(&app.theme).expect("render reading mode");
    if let Ok(path) = std::env::var("NIA_READING_SNAPSHOT") {
        snapshot.matches_image(path).expect("write reading preview");
    }

    ui.click("Write").expect("return to editor");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.current().reading.is_none());
    assert_eq!(app.current().editor.text(), source);
    assert_eq!(app.current().editor.cursor(), cursor);
}

#[test]
fn reading_mode_stays_with_its_tab() {
    let mut app = blank();
    app.current_mut().editor = text_editor::Content::with_text("# First");
    let _ = update(&mut app, Message::ToggleReading);
    assert!(app.current().reading.is_some());
    let _ = update(&mut app, Message::New);
    assert!(app.current().reading.is_none());
    let _ = update(&mut app, Message::SelectTab(0));
    assert!(app.current().reading.is_some());
    assert_eq!(app.current().editor.text(), "# First");
}
