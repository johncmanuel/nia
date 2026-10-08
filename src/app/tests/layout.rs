use super::*;

#[test]
fn layout_shortcuts_work_without_a_layout_button() {
    let mut app = blank();
    app.keybindings = crate::keybindings::Keybindings::default();
    let mut ui = simulator(crate::ui::view(&app));
    assert!(ui.find("Split right").is_err());
    assert!(ui.find("Layout").is_err());
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("d".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.panes.len(), 2);
    if let Ok(path) = std::env::var("NIA_LAYOUT_SNAPSHOT") {
        if std::env::var("NIA_LAYOUT_SNAPSHOT_THEME").as_deref() == Ok("light") {
            app.theme = iced::Theme::Light;
        }
        simulator(crate::ui::view(&app))
            .snapshot(&app.theme)
            .expect("render split layout")
            .matches_image(path)
            .expect("write layout preview");
    }
}

#[test]
fn command_t_opens_a_group_with_its_own_layout_and_compact_bar() {
    let mut app = blank();
    app.keybindings = crate::keybindings::Keybindings::default();
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    app.tabs[0].editor = text_editor::Content::with_text("keep first document");
    app.tabs[0].document.mark_edited();
    let mut ui = simulator(crate::ui::view(&app));
    assert!(ui.find("1  Untitled").is_err(), "one group has no tab bar");
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("t".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.groups.len(), 2);
    assert_eq!(app.active_group, 1);
    assert_eq!(app.panes.len(), 1);
    assert_eq!(
        app.groups[0]
            .inactive
            .as_ref()
            .expect("first group layout")
            .panes
            .len(),
        2
    );
    let mut ui = simulator(crate::ui::view(&app));
    if let Ok(path) = std::env::var("NIA_GROUP_SNAPSHOT") {
        ui.snapshot(&app.theme)
            .expect("render tab groups")
            .matches_image(path)
            .expect("write group preview");
    }
    ui.find("1  Untitled").expect("first group in tab bar");
    ui.find("2  Untitled").expect("second group in tab bar");
    drop(ui);
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Horizontal));
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("1  Untitled").expect("first group selectable");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.active_group, 0);
    assert_eq!(app.tabs[0].editor.text(), "keep first document");
    assert!(matches!(
        app.panes.layout(),
        pane_grid::Node::Split {
            axis: pane_grid::Axis::Vertical,
            ..
        }
    ));
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("2".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.active_group, 1);
    assert!(matches!(
        app.panes.layout(),
        pane_grid::Node::Split {
            axis: pane_grid::Axis::Horizontal,
            ..
        }
    ));
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("+").expect("new group control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.groups.len(), 3);
    assert_eq!(app.panes.len(), 1);
    let _ = update(&mut app, Message::ToggleZen);
    assert!(
        simulator(crate::ui::view(&app))
            .find("1  Untitled")
            .is_err(),
        "Zen hides the group bar"
    );
}

#[test]
fn split_panes_show_and_edit_separate_documents() {
    let mut app = blank();
    app.keybindings = crate::keybindings::Keybindings::default();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("d".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.panes.len(), 2);
    assert_eq!(app.tabs.len(), 2);
    let first_pane = *app
        .panes
        .iter()
        .find(|(_, id)| **id == 0)
        .expect("first pane")
        .0;
    let second_pane = app.active_pane;

    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new("editor-0"))
        .expect("first editor visible beside second");
    ui.typewrite("first");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.active_pane, first_pane);
    assert_eq!(app.tabs[0].editor.text(), "first");
    assert!(app.tabs[1].document.contents.is_empty());
    let _ = update(
        &mut app,
        Message::Tick(Instant::now() + Duration::from_secs(1)),
    );
    assert_eq!(app.tabs[0].document.contents, "first");

    let _ = update(&mut app, Message::ClosePane(second_pane));
    assert_eq!(app.panes.len(), 1);
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.current().id, 0);
}

#[test]
fn keyboard_splits_below_and_navigation_keeps_visible_panes_in_place() {
    let mut app = blank();
    app.keybindings = crate::keybindings::Keybindings::default();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("d".into()),
        Modifiers::COMMAND | Modifiers::SHIFT,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.tabs.len(), 2);
    assert!(matches!(
        app.panes.layout(),
        pane_grid::Node::Split {
            axis: pane_grid::Axis::Horizontal,
            ..
        }
    ));
    let second = app.active_pane;
    let first = *app
        .panes
        .iter()
        .find(|(_, id)| **id == 0)
        .expect("first pane")
        .0;
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("second editor visible");
    ui.simulate([shortcut_event(
        Key::Character("[".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.active_pane, first);
    assert_eq!(app.panes.get(first), Some(&0));
    assert_eq!(app.panes.get(second), Some(&1));

    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("first editor visible");
    ui.simulate([shortcut_event(
        Key::Character("]".into()),
        Modifiers::COMMAND,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.active_pane, second);
}

#[test]
fn bracket_shortcuts_follow_split_layout_within_the_selected_group() {
    let mut app = blank();
    app.keybindings = crate::keybindings::Keybindings::default();
    let first = app.active_pane;
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    let right = app.active_pane;
    let _ = update(&mut app, Message::ActivatePane(first));
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Horizontal));
    let lower_left = app.active_pane;
    let _ = update(&mut app, Message::NewGroup);
    assert_eq!(app.active_group, 1);

    for (number, group) in [("1", 0), ("2", 1), ("1", 0)] {
        let mut ui = simulator(crate::ui::view(&app));
        ui.click(iced::widget::Id::new(EDITOR))
            .expect("active editor visible");
        ui.simulate([shortcut_event(
            Key::Character(number.into()),
            Modifiers::COMMAND,
        )]);
        for message in ui.into_messages() {
            let _ = update(&mut app, message);
        }
        assert_eq!(app.active_group, group);
    }
    assert_eq!(app.active_pane, lower_left);

    for (key, expected) in [("]", right), ("]", first), ("]", lower_left), ("[", first)] {
        let mut ui = simulator(crate::ui::view(&app));
        ui.click(iced::widget::Id::new(EDITOR))
            .expect("active editor visible");
        ui.simulate([shortcut_event(
            Key::Character(key.into()),
            Modifiers::COMMAND,
        )]);
        for message in ui.into_messages() {
            let _ = update(&mut app, message);
        }
        assert_eq!(app.active_pane, expected);
        assert_eq!(app.active_group, 0);
        assert_eq!(app.panes.len(), 3);
    }
}

#[test]
fn groups_keep_their_own_layouts_when_documents_close() {
    let mut app = blank();
    let _ = update(&mut app, Message::NewGroup);
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Horizontal));
    assert_eq!(app.panes.len(), 2);
    let _ = update(&mut app, Message::SelectGroup(0));
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    assert_eq!(app.tabs.len(), 4);
    assert_eq!(app.active_group, 0);
    assert!(matches!(
        app.panes.layout(),
        pane_grid::Node::Split {
            axis: pane_grid::Axis::Vertical,
            ..
        }
    ));
    let _ = update(&mut app, Message::CloseTab(1));
    assert_eq!(
        app.active_group, 0,
        "closing a background group keeps the current group"
    );
    assert_eq!(app.groups.len(), 2);
    assert_eq!(app.tabs.len(), 3);
    assert_eq!(app.panes.len(), 2);
    let _ = update(&mut app, Message::SinglePane);
    assert_eq!(app.panes.len(), 1);
    let _ = update(&mut app, Message::SelectGroup(1));
    assert_eq!(app.panes.len(), 1, "closed document's pane was removed");
    assert_eq!(app.panes.get(app.active_pane), Some(&2));
}

#[test]
fn closing_the_last_document_closes_only_its_group() {
    let mut app = blank();
    let _ = update(&mut app, Message::NewGroup);
    app.current_mut().editor = text_editor::Content::with_text("keep this draft");
    app.current_mut().document.mark_edited();
    let _ = update(&mut app, Message::CloseCurrentTab);
    assert_eq!(app.pending_close, Some(CloseTarget::Tab(1)));
    let _ = update(&mut app, Message::CancelClose);
    assert_eq!(app.groups.len(), 2);
    assert_eq!(app.current().editor.text(), "keep this draft");
    let _ = update(&mut app, Message::CloseCurrentTab);
    let _ = update(&mut app, Message::DiscardAndClose);
    assert_eq!(app.groups.len(), 1);
    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.current().id, 0);
}

#[test]
fn closing_tabs_keeps_the_selected_document_and_unsaved_work() {
    let mut app = blank();
    app.loading = false;
    let _ = update(&mut app, Message::New);
    let _ = update(&mut app, Message::New);
    let selected_id = app.current().id;
    let _ = update(&mut app, Message::CloseTab(0));
    assert_eq!(app.current().id, selected_id);

    app.current_mut().document.edit("unsaved".into());
    app.current_mut().editor = text_editor::Content::with_text("unsaved");
    let active = app.active;
    let _ = update(&mut app, Message::CloseTab(active));
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.current().document.contents, "unsaved");
}

#[test]
fn closing_a_hidden_dirty_tab_brings_it_into_view() {
    let mut app = blank();
    app.tabs[0].document.edit("keep me".into());
    app.tabs[0].editor = text_editor::Content::with_text("keep me");
    let _ = update(&mut app, Message::New);
    let _ = update(&mut app, Message::CloseTab(0));
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.current().id, 0);
    assert_eq!(app.panes.get(app.active_pane), Some(&0));
    assert_eq!(app.current().document.contents, "keep me");
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
    // iced_test renders at 2x scale. Sample the right margin beside the editor.
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
