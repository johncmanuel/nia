use super::*;

#[test]
fn layout_control_splits_without_permanent_buttons() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    assert!(ui.find("Split right").is_err());
    ui.click("Layout").expect("layout control visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Split right").expect("split option visible");
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
fn split_panes_show_and_edit_separate_documents() {
    let mut app = blank();
    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("R".into()),
        Modifiers::CTRL | Modifiers::SHIFT,
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
fn existing_tabs_can_be_arranged_and_replaced_in_panes() {
    let mut app = blank();
    let _ = update(&mut app, Message::New);
    let _ = update(&mut app, Message::New);
    let _ = update(&mut app, Message::SelectTab(0));
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    assert_eq!(app.tabs.len(), 3);
    let second = app.active_pane;
    let first = *app
        .panes
        .iter()
        .find(|(_, id)| **id == 0)
        .expect("first pane")
        .0;
    assert_eq!(*app.panes.get(second).expect("second pane"), 1);

    let _ = update(&mut app, Message::ActivatePane(first));
    let _ = update(&mut app, Message::SelectTab(1));
    assert_eq!(*app.panes.get(first).expect("first pane"), 1);
    assert_eq!(*app.panes.get(second).expect("second pane"), 0);
    assert_eq!(app.active_pane, first);

    let _ = update(&mut app, Message::CloseTab(1));
    assert_eq!(app.panes.len(), 2);
    assert_eq!(*app.panes.get(first).expect("replacement pane"), 2);
    assert_eq!(app.current().id, 2);

    let mut ui = simulator(crate::ui::view(&app));
    ui.click(iced::widget::Id::new(EDITOR))
        .expect("editor visible");
    ui.simulate([shortcut_event(
        Key::Character("L".into()),
        Modifiers::CTRL | Modifiers::SHIFT,
    )]);
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert_eq!(app.panes.len(), 1);
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.panes.get(app.active_pane), Some(&2));
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
    // iced_test renders at 2x scale. Sample the right margin below the tab bar.
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
