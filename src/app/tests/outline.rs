use super::*;

#[test]
fn outline_is_left_of_the_editor_and_tracks_the_active_document() {
    let mut app = blank();
    let mut first = Document::new();
    first.edit("# First\nText\n## Second\nMore".into());
    app.tabs[0] = Tab::new(0, first);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Outline").expect("outline button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.show_outline);

    let mut ui = simulator(crate::ui::view(&app));
    let outline = ui
        .find(iced::widget::Id::new(OUTLINE))
        .expect("outline filter visible")
        .bounds();
    let editor = ui
        .find(iced::widget::Id::new(EDITOR))
        .expect("editor visible beside outline")
        .bounds();
    assert!(outline.x < editor.x);
    ui.click("Filter headings").expect("outline filter visible");
    ui.typewrite("second");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.outline_items().len(), 1);
    let _ = update(&mut app, Message::JumpOutlineSelected);
    assert_eq!(app.current().editor.cursor().position.line, 2);
    assert!(app.show_outline);

    let _ = update(&mut app, Message::New);
    let mut second = Document::new();
    second.edit("# Another document".into());
    let id = app.current().id;
    app.tabs[1] = Tab::new(id, second);
    assert!(app.show_outline);
    let mut ui = simulator(crate::ui::view(&app));
    ui.find("Another document")
        .expect("outline follows the new active tab");
    assert!(ui.find("First").is_err());
    ui.click("Hide").expect("mouse close button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(!app.show_outline);
}

#[test]
fn outline_shortcut_opens_and_closes_the_panel() {
    let mut app = blank();
    let shortcut = Key::Character("O".into());
    let modifiers = Modifiers::COMMAND | Modifiers::SHIFT;
    for visible in [true, false] {
        let message = app
            .key_message(&shortcut, modifiers)
            .expect("default outline shortcut");
        let _ = update(&mut app, message);
        assert_eq!(app.show_outline, visible);
    }
}
