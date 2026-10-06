use super::*;

#[test]
fn font_setting_defaults_and_round_trips() -> io::Result<()> {
    let path = std::env::temp_dir().join(format!(
        "nia-font-{}-{}.conf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    let result = (|| {
        assert_eq!(load_from(&path)?, None);
        save_to(&path, Some("Example Family"))?;
        assert_eq!(load_from(&path)?, Some("Example Family".into()));
        save_to(&path, None)?;
        assert_eq!(load_from(&path)?, None);
        Ok(())
    })();
    let _ = fs::remove_file(path);
    result
}

#[test]
fn font_size_defaults_round_trips_and_rejects_invalid_values() -> io::Result<()> {
    let path = std::env::temp_dir().join(format!(
        "nia-font-size-{}-{}.conf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    let result = (|| {
        assert_eq!(load_size_from(&path)?, DEFAULT_SIZE);
        save_size_to(&path, 24)?;
        assert_eq!(load_size_from(&path)?, 24);
        fs::write(&path, "200")?;
        assert_eq!(
            load_size_from(&path).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        Ok(())
    })();
    let _ = fs::remove_file(path);
    result
}
