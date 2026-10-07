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
        ui.snapshot(&app.theme)
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
fn long_word_stays_inside_writing_area() {
    let mut app = blank();
    app.current_mut().editor = text_editor::Content::with_text(&"superlongword".repeat(100));
    let mut ui = iced_test::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(700.0, 500.0),
        crate::ui::view(&app),
    );
    let directory = std::env::temp_dir().join(format!("nia-long-line-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    ui.snapshot(&app.theme)
        .expect("render long line")
        .matches_image(directory.join("preview.png"))
        .expect("write preview");
    let path = std::fs::read_dir(&directory)
        .expect("read preview directory")
        .next()
        .expect("preview exists")
        .expect("read preview entry")
        .path();
    let mut reader = png::Decoder::new(std::io::BufReader::new(
        std::fs::File::open(path).expect("open preview"),
    ))
    .read_info()
    .expect("decode preview");
    let mut pixels = vec![0; reader.output_buffer_size().expect("preview size")];
    reader.next_frame(&mut pixels).expect("read preview pixels");
    let background = &pixels[..3];
    assert!(
        (200..300).all(|y| (1360..1400).all(|x| {
            let index = (y * 1400 + x) * 4;
            pixels[index..index + 3] == *background
        })),
        "text painted into the right margin"
    );
    std::fs::remove_dir_all(directory).expect("remove preview");
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
