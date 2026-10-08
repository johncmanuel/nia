use super::*;

#[test]
fn zen_mode_hides_footer_and_escape_restores_it() {
    let mut app = blank();
    app.keybindings = crate::keybindings::Keybindings::default();
    let mut ui = simulator(crate::ui::view(&app));
    assert!(ui.find("Zen").is_err(), "footer has no Zen button");
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("J".into()),
        Modifiers::COMMAND | Modifiers::SHIFT,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.zen_mode);
    let mut ui = simulator(crate::ui::view(&app));
    ui.find(iced::widget::Id::new(EDITOR))
        .expect("editor remains visible");
    assert!(ui.find("Settings").is_err());
    if let Ok(path) = std::env::var("NIA_ZEN_SNAPSHOT") {
        ui.snapshot(&app.theme)
            .expect("render Zen mode")
            .matches_image(path)
            .expect("write Zen preview");
    }
    drop(ui);
    let _ = update(
        &mut app,
        Message::KeyPressed(
            Key::Named(iced::keyboard::key::Named::Escape),
            Modifiers::NONE,
        ),
    );
    assert!(!app.zen_mode);
    simulator(crate::ui::view(&app))
        .find("Settings")
        .expect("footer restored");
}

#[test]
fn zen_can_be_toggled_without_closing_settings() {
    let mut app = blank();
    let _ = update(&mut app, Message::ShowSettings);
    let mut ui = simulator(crate::ui::view(&app));
    assert!(ui.find(iced::widget::Id::new(EDITOR)).is_err());
    assert!(
        ui.find("Untitled").is_err(),
        "Settings should hide the editor"
    );
    ui.click("Zen mode").expect("zen setting visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.zen_mode && app.show_settings);

    let mut ui = simulator(crate::ui::view(&app));
    ui.find("Zen mode").expect("settings remains visible");
    ui.click("Back to writing").expect("return control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.zen_mode && !app.show_settings);
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible in Zen mode");
    ui.simulate([shortcut_event(
        Key::Character(",".into()),
        if cfg!(target_os = "macos") {
            Modifiers::COMMAND
        } else {
            Modifiers::CTRL
        },
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.zen_mode && app.show_settings);
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Zen mode")
        .expect("zen setting visible in Zen mode");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(!app.zen_mode && app.show_settings);
}

#[test]
fn zen_keeps_panes_outline_and_search() {
    let mut app = blank();
    let _ = update(&mut app, Message::New);
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    let _ = update(&mut app, Message::ShowOutline);
    let _ = update(&mut app, Message::ShowSearch);
    let _ = update(&mut app, Message::ToggleZen);
    assert!(app.zen_mode && app.show_outline && app.show_search);
    assert_eq!(app.panes.len(), 2);
    let mut ui = simulator(crate::ui::view(&app));
    ui.find(iced::widget::Id::new(OUTLINE))
        .expect("outline remains visible");
    ui.find(iced::widget::Id::new(SEARCH))
        .expect("search remains visible");
    ui.find(iced::widget::Id::new(EDITOR))
        .expect("active pane remains visible");
    let inactive_id = app
        .panes
        .iter()
        .find_map(|(pane, id)| (*pane != app.active_pane).then_some(id))
        .expect("second pane");
    ui.find(iced::widget::Id::from(format!("editor-{inactive_id}")))
        .expect("second pane remains visible");
    assert!(ui.find("Settings").is_err(), "footer is hidden");
}
