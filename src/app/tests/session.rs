use super::*;
use crate::app::session;
use iced::widget::pane_grid::Node;

#[test]
fn open_tabs_and_nested_panes_survive_session_restore() -> std::io::Result<()> {
    let directory = std::env::temp_dir().join(format!("nia-layout-test-{}", recovery::new_key()));
    std::fs::create_dir_all(&directory)?;
    let first = directory.join("first.md");
    let second = directory.join("second.md");
    std::fs::write(&first, "first document")?;
    std::fs::write(&second, "second document")?;
    let layout_path = directory.join("layout.conf");

    let mut app = blank();
    app.tabs[0] = Tab::new(0, Document::open(first.clone())?);
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Horizontal));
    app.tabs[2] = Tab::new(2, Document::open(second.clone())?);
    if let Node::Split { id, .. } = app.panes.layout() {
        let split = *id;
        let _ = update(
            &mut app,
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio: 0.35 }),
        );
    }
    session::save_to(&app, &layout_path)?;

    let saved = session::load_from(&layout_path)?.expect("saved layout");
    let mut restored = blank();
    session::restore(&mut restored, saved);
    assert_eq!(restored.tabs.len(), 3);
    assert_eq!(restored.panes.len(), 3);
    assert_eq!(restored.tabs[0].document.contents, "first document");
    assert_eq!(restored.tabs[2].document.contents, "second document");
    assert_eq!(restored.active, 2);
    assert_eq!(restored.panes.get(restored.active_pane), Some(&2));
    assert!(
        matches!(restored.panes.layout(), Node::Split { axis: pane_grid::Axis::Vertical, ratio, .. } if (*ratio - 0.35).abs() < 0.001)
    );
    let mut ui = simulator(crate::ui::view(&restored));
    ui.find(iced::widget::Id::new(EDITOR))
        .expect("restored active editor");
    ui.find(iced::widget::Id::new("editor-0"))
        .expect("restored first editor");
    std::fs::remove_dir_all(directory)
}

#[test]
fn corrupt_or_duplicate_layout_is_rejected() {
    assert!(session::decode("nia-layout-1\n2\n0\n-\n-\nS:V:0.5\nP:0\nP:0\n").is_none());
    assert!(session::decode("nia-layout-1\n2\n1\n-\n-\nP:0\n").is_none());
    assert!(session::decode("nia-layout-3\n1\n0\n-\nX\n1\n1\n0\n0\nP:0\n").is_none());
}

#[test]
fn selected_group_and_each_tab_mode_survive_session_restore() -> std::io::Result<()> {
    let directory = std::env::temp_dir().join(format!("nia-modes-{}", recovery::new_key()));
    std::fs::create_dir_all(&directory)?;
    let layout = directory.join("layout.conf");
    let paths = ["shared.md", "other.md"].map(|name| directory.join(name));
    for (index, path) in paths.iter().enumerate() {
        std::fs::write(path, format!("# Document {index}"))?;
    }

    let mut app = blank();
    app.tabs[0] = Tab::new(0, Document::open(paths[0].clone())?);
    let _ = update(&mut app, Message::ToggleReading);
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    app.tabs[1] = Tab::new(1, Document::open(paths[0].clone())?);
    let _ = update(&mut app, Message::NewGroup);
    app.tabs[2] = Tab::new(2, Document::open(paths[1].clone())?);
    let _ = update(&mut app, Message::ToggleReading);
    let _ = update(&mut app, Message::SelectGroup(0));
    assert_eq!(app.active_group, 0);
    assert_eq!(app.active, 1);
    session::save_to(&app, &layout)?;

    let mut restored = blank();
    session::restore(
        &mut restored,
        session::load_from(&layout)?.expect("saved modes"),
    );
    assert_eq!(restored.active_group, 0);
    assert_eq!(restored.active, 1);
    assert_eq!(
        restored
            .tabs
            .iter()
            .map(|tab| tab.reading.is_some())
            .collect::<Vec<_>>(),
        [true, false, true]
    );
    let mut ui = simulator(crate::ui::view(&restored));
    ui.find("Read")
        .expect("selected group restores writing mode");
    drop(ui);
    let _ = update(&mut restored, Message::SelectGroup(1));
    let mut ui = simulator(crate::ui::view(&restored));
    ui.find("Write").expect("other group restores reading mode");
    std::fs::remove_dir_all(directory)
}

#[test]
fn separate_group_layouts_and_legacy_session_restore() -> std::io::Result<()> {
    let path = std::env::temp_dir().join(format!("nia-groups-{}.conf", recovery::new_key()));
    let mut app = blank();
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
    let _ = update(&mut app, Message::NewGroup);
    let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Horizontal));
    session::save_to(&app, &path)?;
    let mut restored = blank();
    session::restore(
        &mut restored,
        session::load_from(&path)?.expect("saved groups"),
    );
    assert_eq!(restored.groups.len(), 2);
    assert_eq!(restored.active_group, 1);
    assert!(matches!(
        restored.panes.layout(),
        Node::Split {
            axis: pane_grid::Axis::Horizontal,
            ..
        }
    ));
    let _ = update(&mut restored, Message::SelectGroup(0));
    assert!(matches!(
        restored.panes.layout(),
        Node::Split {
            axis: pane_grid::Axis::Vertical,
            ..
        }
    ));
    std::fs::remove_file(path)?;

    let old = "nia-layout-1\n2\n1\n-\n-\nS:V:0.5\nP:0\nP:1\n";
    let mut migrated = blank();
    session::restore(&mut migrated, session::decode(old).expect("old layout"));
    assert_eq!(migrated.groups.len(), 1);
    assert_eq!(migrated.panes.len(), 2);
    assert_eq!(migrated.current().id, 1);
    assert!(migrated.tabs.iter().all(|tab| tab.reading.is_none()));
    let old = "nia-layout-2\n1\n0\n-\n1\n1\n0\n0\nP:0\n";
    let mut migrated = blank();
    session::restore(&mut migrated, session::decode(old).expect("old groups"));
    assert!(migrated.current().reading.is_none());
    Ok(())
}
