use super::*;

#[test]
fn zen_mode_hides_chrome_and_escape_restores_it() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Zen").expect("zen control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.zen_mode);
    let mut ui = simulator(crate::ui::view(&app));
    ui.find(iced::widget::Id::new(EDITOR))
        .expect("editor remains visible");
    assert!(ui.find("Layout").is_err());
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
        .find("Layout")
        .expect("chrome restored");
}

#[test]
fn zen_can_be_toggled_without_closing_settings() {
    let mut app = blank();
    let _ = update(&mut app, Message::ShowSettings);
    let mut ui = simulator(crate::ui::view(&app));
    assert!(ui.find(iced::widget::Id::new(EDITOR)).is_err());
    assert!(ui.find("Untitled").is_err(), "Settings should hide tabs");
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
fn typewriter_controls_are_available_in_settings() {
    let mut app = blank();
    app.focus = crate::focus::Settings::default();
    let _ = update(&mut app, Message::ShowSettings);
    let mut ui = simulator(crate::ui::view(&app));
    ui.find("Highlight current line")
        .expect("highlight option visible");
    ui.find("Lines above: 8").expect("upper context visible");
    ui.find("Lines below: 12").expect("lower context visible");
    ui.find("Dim unfocused").expect("dimming option visible");
    ui.click("Typewriter scrolling")
        .expect("typewriter control visible");
    assert!(
        ui.into_messages()
            .into_iter()
            .any(|message| matches!(message, Message::ToggleTypewriter))
    );
}

#[test]
fn typewriter_view_limits_visible_context_around_caret() {
    let mut app = blank();
    app.focus = crate::focus::Settings {
        typewriter: true,
        highlight_line: true,
        lines_above: 4,
        lines_below: 3,
        dimming: crate::focus::Dimming::Off,
    };
    let contents = (1..=30)
        .map(|number| format!("Line {number}"))
        .collect::<Vec<_>>()
        .join("\n");
    app.current_mut().editor = text_editor::Content::with_text(&contents);
    app.current_mut().editor.move_to(text_editor::Cursor {
        position: text_editor::Position {
            line: 14,
            column: 7,
        },
        selection: None,
    });
    let mut ui = simulator(crate::ui::view(&app));
    ui.snapshot(&app.theme).expect("layout typewriter editor");
    drop(ui);
    let _ = update(&mut app, Message::AlignTypewriter);
    let mut ui = simulator(crate::ui::view(&app));
    if let Ok(path) = std::env::var("NIA_TYPEWRITER_SNAPSHOT") {
        ui.snapshot(&app.theme)
            .expect("render typewriter mode")
            .matches_image(path)
            .expect("write typewriter preview");
    }
    let bounds = ui
        .find(iced::widget::Id::new(EDITOR))
        .expect("editor stays interactive")
        .bounds();
    assert!(bounds.height < 300.0);
    drop(ui);
    let id = app.current().id;
    let _ = update(
        &mut app,
        Message::Edit(id, text_editor::Action::Move(text_editor::Motion::Down)),
    );
    assert_eq!(app.current().editor.cursor().position.line, 15);
    if let Ok(path) = std::env::var("NIA_TYPEWRITER_MOVED_SNAPSHOT") {
        simulator(crate::ui::view(&app))
            .snapshot(&app.theme)
            .expect("render moved typewriter caret")
            .matches_image(path)
            .expect("write moved typewriter preview");
    }
}
