use super::*;

#[test]
fn tab_and_find_controls_work_in_headless_ui() {
    let mut app = blank();
    app.loading = false;
    let mut first = Document::new();
    first.edit("alpha beta alpha".into());
    app.tabs[0] = Tab::new(0, first);

    let mut ui = simulator(crate::ui::view(&app));
    if let Ok(path) = std::env::var("NIA_SNAPSHOT") {
        ui.snapshot(&app.theme.iced())
            .expect("render editor")
            .matches_image(path)
            .expect("write preview");
    }
    ui.click("New").expect("New button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.active, 1);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Untitled \u{2022}").expect("dirty tab visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.active, 0);
    assert!(app.current().document.dirty);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Find").expect("Find button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.show_search);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Find in document").expect("search input visible");
    ui.typewrite("alpha");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.search_matches().len(), 2);
    assert_eq!(app.search_index, Some(0));

    let mut ui = simulator(crate::ui::view(&app));
    assert!(ui.find("1/2").is_ok());
    ui.click("\u{2193}").expect("next match button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.search_index, Some(1));
}

#[test]
fn search_positions_handle_unicode_and_line_breaks() {
    let mut app = blank();
    app.loading = false;
    let mut document = Document::new();
    document.edit("🌍 alpha\nalpha".into());
    app.tabs[0] = Tab::new(0, document);
    let _ = update(&mut app, Message::SearchChanged("alpha".into()));
    let matches = app.search_matches();
    assert_eq!(matches[0].0, text_editor::Position { line: 0, column: 5 });
    assert_eq!(matches[1].0, text_editor::Position { line: 1, column: 0 });
    assert_eq!(app.current().editor.selection().as_deref(), Some("alpha"));
}

#[test]
fn search_open_tabs_finds_unsaved_text_and_opens_hidden_matches() {
    let mut app = blank();
    let mut first = Document::new();
    first.edit("alpha first".into());
    app.tabs[0] = Tab::new(0, first);
    let _ = update(&mut app, Message::New);
    let second_id = app.current().id;
    app.current_mut().editor = text_editor::Content::with_text("alpha second");
    app.current_mut().document.mark_edited();

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Find").expect("find control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("All tabs").expect("all-tabs scope visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.show_search && app.search_all_tabs);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Search open tabs")
        .expect("global search field visible");
    ui.typewrite("alpha");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.tab_search_hits.len(), 2);
    assert_eq!(app.tab_search_hits[0].tab_id, 0);
    assert_eq!(app.tab_search_hits[1].tab_id, second_id);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("1: Untitled:1  alpha first")
        .expect("hidden first tab result visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.current().id, 0);
    assert_eq!(app.current().editor.selection().as_deref(), Some("alpha"));
    assert!(app.show_search && app.search_all_tabs);

    let _ = update(&mut app, Message::FindNext);
    assert_eq!(app.current().id, second_id);
    assert_eq!(app.current().editor.selection().as_deref(), Some("alpha"));
}

#[test]
fn global_search_shortcut_opens_the_all_tabs_scope() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("F".into()),
        Modifiers::COMMAND | Modifiers::SHIFT,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.show_search && app.search_all_tabs);
}

#[test]
fn font_picker_is_available_in_headless_ui() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Font").expect("font control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.show_font_picker);
    assert!(simulator(crate::ui::view(&app)).find("Monospace").is_ok());
}

#[test]
fn find_shortcuts_from_editor_open_search() {
    for modifiers in [Modifiers::COMMAND, Modifiers::CTRL] {
        let mut app = blank();
        app.loading = false;
        let mut ui = simulator(crate::ui::view(&app));
        ui.click(iced::widget::Id::new(EDITOR))
            .expect("editor visible");
        let mut press = iced_test::simulator::press_key(Key::Character("f".into()), None);
        if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            modifiers: pressed, ..
        }) = &mut press
        {
            *pressed = modifiers;
        }
        ui.simulate([press]);
        let messages: Vec<_> = ui.into_messages().collect();
        assert!(
            messages
                .iter()
                .any(|message| matches!(message, Message::ShowSearch))
        );
        for message in messages {
            let _ = update(&mut app, message);
        }
        assert!(app.show_search);
    }
}

#[test]
fn search_does_not_move_the_editor() {
    let mut app = blank();
    let before = simulator(crate::ui::view(&app))
        .find(iced::widget::Id::new(EDITOR))
        .expect("editor visible")
        .bounds();
    let _ = update(&mut app, Message::ShowSearch);
    let mut ui = simulator(crate::ui::view(&app));
    let after = ui
        .find(iced::widget::Id::new(EDITOR))
        .expect("editor visible with search")
        .bounds();
    assert_eq!(before, after);
    assert!(ui.find("Find in document").is_ok());
}

#[test]
fn browser_shortcuts_create_switch_and_close_tabs() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("t".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.active, 1);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Named(iced::keyboard::key::Named::Tab),
        Modifiers::CTRL,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.active, 0);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("9".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.active, 1);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("w".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.tabs.len(), 1);
}

#[test]
fn tab_shortcut_reaches_global_handler_from_search_field() {
    let mut app = blank();
    let _ = update(&mut app, Message::ShowSearch);
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Find in document").expect("search field visible");
    let status = ui.simulate([shortcut_event(
        Key::Named(iced::keyboard::key::Named::Tab),
        Modifiers::CTRL,
    )])[0];
    assert_eq!(status, iced::event::Status::Ignored);
}

#[test]
fn tab_shortcut_can_be_recorded_on_keybindings_page() {
    let mut app = blank();
    let _ = update(&mut app, Message::ShowKeybindings);
    let _ = update(&mut app, Message::RecordShortcut(Slot::NextTab));
    let mut ui = simulator(crate::ui::view(&app));
    let status = ui.simulate([shortcut_event(
        Key::Named(iced::keyboard::key::Named::Tab),
        Modifiers::CTRL,
    )])[0];
    assert_eq!(status, iced::event::Status::Ignored);
}

#[test]
fn outline_filters_and_jumps_in_headless_ui() {
    let mut app = blank();
    let mut document = Document::new();
    document.edit("# First\nText\n## Second\nMore".into());
    app.tabs[0] = Tab::new(0, document);
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Outline").expect("outline button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.show_outline);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Filter headings").expect("outline search visible");
    ui.typewrite("second");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.outline_items().len(), 1);
    let _ = update(&mut app, Message::JumpOutlineSelected);
    assert_eq!(app.current().editor.cursor().position.line, 2);
    assert!(!app.show_outline);
}
