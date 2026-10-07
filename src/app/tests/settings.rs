use super::*;

#[test]
fn font_size_shortcuts_reach_editor_from_headless_ui() {
    let app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("+".into()),
        Modifiers::COMMAND,
    )]);
    assert!(
        ui.into_messages()
            .into_iter()
            .any(|message| matches!(message, Message::IncreaseFontSize))
    );
}

#[test]
fn settings_shortcut_opens_settings_from_editor() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
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
    assert!(app.show_settings);
}

#[test]
fn settings_page_has_font_and_keybindings_controls() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Settings").expect("settings control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.show_settings);
    let mut ui = simulator(crate::ui::view(&app));
    if let Ok(path) = std::env::var("NIA_SETTINGS_SNAPSHOT") {
        ui.snapshot(&app.theme)
            .expect("render settings")
            .matches_image(path)
            .expect("write settings preview");
    }
    ui.find("Font").expect("font heading visible");
    ui.find("Monospace").expect("font reset visible");
    ui.find("Keybindings").expect("shortcuts visible");
    assert!(ui.find("Save document (alternate)").is_err());
    assert!(ui.find("Find in document (alternate)").is_err());
    ui.click("Transparent background")
        .expect("transparency setting visible");
    assert!(
        ui.into_messages()
            .into_iter()
            .any(|message| matches!(message, Message::SetTransparentBackground(true)))
    );
}

#[test]
fn grouped_theme_dropdown_selects_a_light_theme() {
    let mut app = blank();
    app.theme = Theme::Dark;
    let _ = update(&mut app, Message::ShowSettings);
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Dark").expect("theme control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.show_theme_picker);
    let mut ui = simulator(crate::ui::view(&app));
    let snapshot = ui.snapshot(&app.theme).expect("render grouped theme menu");
    if let Ok(path) = std::env::var("NIA_THEME_SNAPSHOT") {
        snapshot.matches_image(path).expect("write theme preview");
    }
    ui.find("Light").expect("light theme heading visible");
    ui.find("Dark").expect("dark theme heading visible");
    ui.click("Solarized Light").expect("light theme visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.theme, Theme::SolarizedLight);
    assert!(!app.show_theme_picker);
}

#[test]
fn tab_shortcut_can_be_recorded_on_settings_page() {
    let mut app = blank();
    let _ = update(&mut app, Message::ShowSettings);
    let _ = update(&mut app, Message::RecordShortcut(Slot::NextTab));
    let mut ui = simulator(crate::ui::view(&app));
    let status = ui.simulate([shortcut_event(
        Key::Named(iced::keyboard::key::Named::Tab),
        Modifiers::CTRL,
    )])[0];
    assert_eq!(status, iced::event::Status::Ignored);
}
