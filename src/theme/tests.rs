use super::*;

#[test]
fn every_builtin_theme_round_trips_and_old_names_migrate() -> io::Result<()> {
    let path = std::env::temp_dir().join(format!(
        "nia-theme-{}-{}.conf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    let result = (|| {
        for theme in Theme::ALL {
            save_to(&path, theme)?;
            assert_eq!(load_from(&path)?, *theme);
        }
        fs::write(&path, "Catppuccin")?;
        assert_eq!(load_from(&path)?, Theme::CatppuccinMocha);
        fs::write(&path, "Gruvbox")?;
        assert_eq!(load_from(&path)?, Theme::GruvboxDark);
        Ok(())
    })();
    let _ = fs::remove_file(path);
    result
}

#[test]
fn picker_groups_each_builtin_theme_by_its_palette() {
    let choices: Vec<_> = choices(false).chain(choices(true)).collect();
    assert_eq!(choices.len(), Theme::ALL.len());
    for theme in Theme::ALL {
        assert_eq!(choices.iter().filter(|choice| **choice == theme).count(), 1);
    }
    let first_dark = choices
        .iter()
        .position(|choice| choice.extended_palette().is_dark)
        .expect("built-in dark themes");
    assert!(first_dark > 0);
    assert!(
        choices[..first_dark]
            .iter()
            .all(|choice| !choice.extended_palette().is_dark)
    );
    assert!(
        choices[first_dark..]
            .iter()
            .all(|choice| choice.extended_palette().is_dark)
    );
}

#[test]
fn transparency_setting_defaults_and_round_trips() -> io::Result<()> {
    let path = std::env::temp_dir().join(format!(
        "nia-transparency-{}-{}.conf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    let result = (|| {
        assert_eq!(load_transparency_from(&path)?, (false, DEFAULT_OPACITY));
        fs::write(&path, "true")?;
        assert_eq!(load_transparency_from(&path)?, (true, DEFAULT_OPACITY));
        save_transparency_to(&path, true, 25)?;
        assert_eq!(load_transparency_from(&path)?, (true, 25));
        save_transparency_to(&path, false, 100)?;
        assert_eq!(load_transparency_from(&path)?, (false, 100));
        fs::write(&path, "true\n101")?;
        assert_eq!(
            load_transparency_from(&path).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        fs::write(&path, "maybe")?;
        assert_eq!(
            load_transparency_from(&path).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        Ok(())
    })();
    let _ = fs::remove_file(path);
    result
}
