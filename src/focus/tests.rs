use super::*;

#[test]
fn focus_settings_round_trip_and_migrate_old_settings() -> io::Result<()> {
    let path = std::env::temp_dir().join(format!(
        "nia-focus-{}-{}.conf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    let result = (|| {
        assert_eq!(Settings::load_from(&path)?, Settings::default());
        let settings = Settings {
            highlight_line: true,
            inactive_pane_dim: 65,
        };
        settings.save_to(&path)?;
        assert_eq!(Settings::load_from(&path)?, settings);
        let migrated = Settings {
            highlight_line: true,
            ..Settings::default()
        };
        fs::write(&path, "true\n")?;
        assert_eq!(Settings::load_from(&path)?, migrated);
        fs::write(&path, "true\nSentence\n")?;
        assert_eq!(Settings::load_from(&path)?, migrated);
        fs::write(&path, "true\ntrue\n4\n6\nSentence\n")?;
        assert_eq!(Settings::load_from(&path)?, migrated);
        fs::write(&path, "false\n0\n")?;
        assert_eq!(Settings::load_from(&path)?.inactive_pane_dim, 0);
        fs::write(&path, "false\n100\n")?;
        assert_eq!(Settings::load_from(&path)?.inactive_pane_dim, 100);
        fs::write(&path, "false\n101\n")?;
        assert_eq!(
            Settings::load_from(&path).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        fs::write(&path, "Unknown\n")?;
        assert_eq!(
            Settings::load_from(&path).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        Ok(())
    })();
    let _ = fs::remove_file(path);
    result
}
