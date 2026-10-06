use super::*;

#[test]
fn save_preserves_edits_and_rejects_external_changes() -> io::Result<()> {
    let dir = std::env::temp_dir().join(format!(
        "nia-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    fs::create_dir(&dir)?;
    let path = dir.join("article.md");
    let result = (|| {
        save_file(&path, "Hello 🌍", None)?;
        assert_eq!(fs::read_to_string(&path)?, "Hello 🌍");
        assert!(save_file(&path, "Overwrite", None).is_err());
        assert_eq!(fs::read_to_string(&path)?, "Hello 🌍");
        save_file(&path, "Hello 🌍\nAgain", Some("Hello 🌍"))?;
        assert_eq!(fs::read_to_string(&path)?, "Hello 🌍\nAgain");
        fs::write(&path, "External edit")?;
        assert!(save_file(&path, "Local edit", Some("Hello 🌍\nAgain")).is_err());
        assert_eq!(fs::read_to_string(&path)?, "External edit");
        Ok(())
    })();
    fs::remove_dir_all(dir)?;
    result
}
