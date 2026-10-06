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
