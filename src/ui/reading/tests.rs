use super::*;
use crate::app::{self, Message};
use iced::widget::text_editor;
use iced_test::simulator;
use std::fs;

#[test]
fn images_resolve_from_document_directory_only() {
    let document = Path::new("/notes/draft.md");
    assert_eq!(
        local_image_path(Some(document), "./images/diagram.png"),
        Some(PathBuf::from("/notes/./images/diagram.png"))
    );
    assert_eq!(local_image_path(None, "images/diagram.png"), None);
    assert_eq!(
        local_image_path(Some(document), "images/my%20diagram.png"),
        Some(PathBuf::from("/notes/images/my diagram.png"))
    );
    assert_eq!(
        local_image_path(Some(document), "https://example.com/a.png"),
        None
    );
    assert_eq!(
        local_image_path(Some(document), "data:image/png;base64,x"),
        None
    );
}

#[test]
fn local_image_renders_in_reading_mode() -> std::io::Result<()> {
    let directory = std::env::temp_dir().join(format!(
        "nia-reading-image-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(std::io::Error::other)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let result = (|| {
        // A small red PPM image exercises Iced's decoder without another crate.
        let mut pixels = b"P6\n120 80\n255\n".to_vec();
        for _ in 0..120 * 80 {
            pixels.extend_from_slice(&[220, 50, 50]);
        }
        fs::write(directory.join("picture.ppm"), pixels)?;
        let mut app = app::blank();
        app.tabs[0].document.path = Some(directory.join("draft.md"));
        app.tabs[0].editor =
            text_editor::Content::with_text("Before\n\n![red](picture.ppm)\n\nAfter");
        let _ = app::update(&mut app, Message::ToggleReading);
        assert!(app.tabs[0].reading.is_some());
        let mut ui = simulator(crate::ui::view(&app));
        assert!(ui.find("Image unavailable: picture.ppm").is_err());
        let snapshot = ui.snapshot(&app.theme).expect("render local image");
        if let Ok(path) = std::env::var("NIA_IMAGE_SNAPSHOT") {
            snapshot.matches_image(path).expect("write image preview");
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(directory);
    result
}

#[test]
fn missing_relative_image_explains_untitled_document() {
    let mut app = app::blank();
    app.tabs[0].editor = text_editor::Content::with_text("![chart](images/chart.png)");
    let _ = app::update(&mut app, Message::ToggleReading);
    let mut ui = simulator(crate::ui::view(&app));
    assert!(
        ui.find("Save this document to show image: images/chart.png")
            .is_ok()
    );
    ui.snapshot(&app.theme)
        .expect("render missing-image fallback");
}
