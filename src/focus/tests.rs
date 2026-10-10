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
        };
        settings.save_to(&path)?;
        assert_eq!(Settings::load_from(&path)?, settings);
        assert_eq!(fs::read_to_string(&path)?, "true\n");
        fs::write(&path, "true\nSentence\n")?;
        assert_eq!(Settings::load_from(&path)?, settings);
        fs::write(&path, "true\ntrue\n4\n6\nSentence\n")?;
        assert_eq!(Settings::load_from(&path)?, settings);
        fs::write(&path, "true\n65\n")?;
        assert_eq!(Settings::load_from(&path)?, settings);
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
