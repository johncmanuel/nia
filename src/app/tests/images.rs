use super::*;
use std::fs;

#[test]
fn writing_view_keeps_image_markdown_visible() -> std::io::Result<()> {
    let directory = std::env::temp_dir().join(format!("nia-image-source-{}", recovery::new_key()));
    fs::create_dir(&directory)?;
    let result = (|| {
        fs::write(directory.join("picture.ppm"), b"P6\n1 1\n255\n\xff\0\0")?;
        let mut document = Document::new();
        document.path = Some(directory.join("draft.md"));
        document.edit("Before\n![red](picture.ppm)\nAfter".into());
        let mut app = blank();
        app.tabs[0] = Tab::new(0, document);

        let mut ui = simulator(crate::ui::view(&app));
        ui.find(iced::widget::Id::new(EDITOR))
            .expect("source editor visible while writing");
        assert!(
            ui.find(iced::widget::Id::new("image-preview-0-1")).is_err(),
            "image links stay in the source editor"
        );
        if let Ok(path) = std::env::var("NIA_SOURCE_IMAGE_SNAPSHOT") {
            ui.snapshot(&app.theme)
                .expect("render source editor")
                .matches_image(path)
                .expect("write source editor snapshot");
        }
        drop(ui);

        app.current_mut().editor.move_to(text_editor::Cursor {
            position: text_editor::Position { line: 1, column: 7 },
            selection: None,
        });
        let _ = update(
            &mut app,
            Message::Edit(0, text_editor::Action::Edit(text_editor::Edit::Insert('X'))),
        );
        assert_eq!(
            app.current().editor.text(),
            "Before\n![red](Xpicture.ppm)\nAfter"
        );
        Ok(())
    })();
    let _ = fs::remove_dir_all(directory);
    result
}
