use super::*;

#[test]
fn defaults_include_control_s_on_macos_and_round_trip() {
    let defaults = Keybindings::default();
    assert_eq!(
        Keybindings::decode(&defaults.encode()),
        Some(defaults.clone())
    );
    if cfg!(target_os = "macos") {
        let control_s = Shortcut::new('s', Modifiers::CTRL);
        assert_eq!(defaults.get(Slot::AlternateSave), control_s);
        assert_eq!(
            defaults.matching(&Key::Character("s".into()), Modifiers::CTRL),
            Some(Slot::AlternateSave)
        );
    }
    assert_eq!(
        defaults.matching(
            &Key::Character("o".into()),
            Modifiers::COMMAND | Modifiers::SHIFT
        ),
        Some(Slot::Outline)
    );
    assert_eq!(
        defaults.matching(&Key::Character("f".into()), Modifiers::COMMAND),
        Some(Slot::Find)
    );
    assert_eq!(
        defaults.matching(
            &Key::Character("f".into()),
            Modifiers::COMMAND | Modifiers::SHIFT
        ),
        Some(Slot::FindTabs)
    );
    if cfg!(target_os = "macos") {
        assert_eq!(
            defaults.matching(&Key::Character("f".into()), Modifiers::CTRL),
            Some(Slot::AlternateFind)
        );
    }
}

#[test]
fn assigned_shortcuts_match_exactly_and_reject_conflicts() {
    let mut bindings = Keybindings::default();
    let custom = Shortcut::new('k', Modifiers::COMMAND | Modifiers::SHIFT).expect("valid shortcut");
    bindings.assign(Slot::New, custom).expect("unused shortcut");
    assert_eq!(
        bindings.matching(
            &Key::Character("K".into()),
            Modifiers::COMMAND | Modifiers::SHIFT
        ),
        Some(Slot::New)
    );
    assert_eq!(
        bindings.matching(&Key::Character("k".into()), Modifiers::COMMAND),
        None
    );
    assert_eq!(bindings.assign(Slot::Open, custom), Err(Slot::New));
}

#[test]
fn old_four_shortcut_settings_keep_default_find() {
    let old = Keybindings::default().encode();
    let old = old.lines().take(4).collect::<Vec<_>>().join("\n");
    let loaded = Keybindings::decode(&old).expect("valid previous settings");
    assert_eq!(
        loaded.get(Slot::Find),
        Keybindings::default().get(Slot::Find)
    );
}

#[test]
fn previous_control_f_setting_gains_command_f_on_macos() {
    if !cfg!(target_os = "macos") {
        return;
    }
    let old = "meta+n\nmeta+o\nmeta+s\nctrl+s\nctrl+f\n";
    let loaded = Keybindings::decode(old).expect("valid previous settings");
    assert_eq!(
        loaded.matching(&Key::Character("f".into()), Modifiers::LOGO),
        Some(Slot::Find)
    );
    assert_eq!(
        loaded.matching(&Key::Character("f".into()), Modifiers::CTRL),
        Some(Slot::AlternateFind)
    );
    assert_eq!(
        loaded.get(Slot::Outline),
        Keybindings::default().get(Slot::Outline)
    );
    assert_eq!(
        loaded.get(Slot::Recent),
        Keybindings::default().get(Slot::Recent)
    );
}

#[test]
fn browser_tab_shortcuts_survive_old_settings_and_round_trip() {
    let defaults = Keybindings::default();
    let old = defaults
        .encode()
        .lines()
        .take(8)
        .collect::<Vec<_>>()
        .join("\n");
    let loaded = Keybindings::decode(&old).expect("valid previous settings");
    assert_eq!(loaded.get(Slot::NewTab), defaults.get(Slot::NewTab));
    assert_eq!(loaded.get(Slot::CloseTab), defaults.get(Slot::CloseTab));
    assert_eq!(loaded.get(Slot::NextTab), defaults.get(Slot::NextTab));
    assert_eq!(Keybindings::decode(&loaded.encode()), Some(loaded.clone()));
    assert_eq!(
        loaded.matching(&Key::Named(Named::Tab), Modifiers::CTRL | Modifiers::SHIFT),
        Some(Slot::PreviousTab)
    );
}

#[test]
fn old_settings_gain_global_search_without_reassigning_existing_slots() {
    let defaults = Keybindings::default();
    let old = defaults
        .encode()
        .lines()
        .take(15)
        .collect::<Vec<_>>()
        .join("\n");
    let loaded = Keybindings::decode(&old).expect("valid previous settings");
    assert_eq!(loaded.get(Slot::Find), defaults.get(Slot::Find));
    assert_eq!(loaded.get(Slot::FindTabs), defaults.get(Slot::FindTabs));
}

#[test]
fn recorded_shortcut_survives_reload() -> io::Result<()> {
    let path = std::env::temp_dir().join(format!("nia-shortcuts-{}.conf", std::process::id()));
    let mut bindings = Keybindings::default();
    let shortcut = Shortcut::new('g', Modifiers::COMMAND | Modifiers::SHIFT)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid test shortcut"))?;
    bindings
        .assign(Slot::Outline, shortcut)
        .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "test shortcut conflict"))?;
    bindings.save_to(&path)?;
    let restored = Keybindings::load_from(&path)?;
    fs::remove_file(path)?;
    assert_eq!(
        restored.matching(
            &Key::Character("g".into()),
            Modifiers::COMMAND | Modifiers::SHIFT
        ),
        Some(Slot::Outline)
    );
    if cfg!(target_os = "macos") {
        assert_eq!(
            restored.matching(&Key::Character("s".into()), Modifiers::CTRL),
            Some(Slot::AlternateSave)
        );
    }
    Ok(())
}
