use super::*;
use std::fs;

#[test]
fn image_block_previews_when_caret_moves_to_other_text_and_source_stays_editable()
-> std::io::Result<()> {
    let directory = std::env::temp_dir().join(format!("nia-inline-image-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory)?;
    let result = (|| {
        let mut pixels = b"P6\n800 500\n255\n".to_vec();
        for _ in 0..800 * 500 {
            pixels.extend_from_slice(&[220, 50, 50]);
        }
        fs::write(directory.join("picture.ppm"), pixels)?;
        let mut document = Document::new();
        document.path = Some(directory.join("draft.md"));
        document.edit("Before\n![red](picture.ppm)\nAfter".into());
        let mut app = blank();
        app.tabs[0] = Tab::new(0, document);
        assert_eq!(app.tabs[0].image_blocks.len(), 3);
        app.tabs[0].move_image_cursor(text_editor::Cursor {
            position: text_editor::Position { line: 2, column: 0 },
            selection: None,
        });
        let mut ui = simulator(crate::ui::view(&app));
        assert!(
            ui.find("![red](picture.ppm)").is_err(),
            "image source hidden when caret is elsewhere"
        );
        let preview = directory.join("preview.png");
        ui.snapshot(&app.theme)
            .expect("render inline image")
            .matches_image(&preview)
            .expect("write image preview");
        let rendered = fs::read_dir(&directory)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| path.extension().is_some_and(|extension| extension == "png"))
            .ok_or_else(|| std::io::Error::other("missing preview"))?;
        if let Ok(path) = std::env::var("NIA_INLINE_IMAGE_SNAPSHOT") {
            fs::copy(&rendered, path)?;
        }
        let file = fs::File::open(rendered)?;
        let mut reader = png::Decoder::new(std::io::BufReader::new(file))
            .read_info()
            .map_err(std::io::Error::other)?;
        let mut output = vec![
            0;
            reader
                .output_buffer_size()
                .ok_or_else(|| std::io::Error::other("missing image size"))?
        ];
        let frame = reader
            .next_frame(&mut output)
            .map_err(std::io::Error::other)?;
        let (left, right) = output[..frame.buffer_size()]
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter_map(|(index, pixel)| {
                (pixel[0] > 190 && pixel[1] < 90 && pixel[2] < 90)
                    .then_some(index % frame.width as usize)
            })
            .fold((usize::MAX, 0), |(left, right), x| {
                (left.min(x), right.max(x))
            });
        assert!(
            right.saturating_sub(left) > 1000,
            "image preview should use the available writing width"
        );

        ui.click(iced::widget::Id::new(EDITOR))
            .expect("text after image editable");
        ui.typewrite("X");
        for message in ui.into_messages() {
            let _ = update(&mut app, message);
        }
        assert_eq!(
            app.tabs[0].editor.text(),
            "Before\n![red](picture.ppm)\nAfterX"
        );

        let mut ui = simulator(crate::ui::view(&app));
        ui.point_at(iced::Point::new(450.0, 200.0));
        ui.simulate(iced_test::simulator::click());
        for message in ui.into_messages() {
            let _ = update(&mut app, message);
        }
        let mut ui = simulator(crate::ui::view(&app));
        assert_eq!(app.tabs[0].active_image_block, 1);
        ui.find(iced::widget::Id::new(EDITOR))
            .expect("image source editor visible when selected");
        drop(ui);

        app.tabs[0].move_image_cursor(text_editor::Cursor {
            position: text_editor::Position { line: 2, column: 0 },
            selection: None,
        });
        let _ = update(&mut app, Message::SplitPane(pane_grid::Axis::Vertical));
        let mut ui = simulator(crate::ui::view(&app));
        let preview = ui
            .find(iced::widget::Id::new("image-preview-0-1"))
            .expect("image preview in first split")
            .bounds();
        let other_editor = ui
            .find(iced::widget::Id::new(EDITOR))
            .expect("editor in second split")
            .bounds();
        assert!(preview.x + preview.width <= other_editor.x);
        assert!(preview.height <= 560.0);
        Ok(())
    })();
    let _ = fs::remove_dir_all(directory);
    result
}

#[test]
fn image_source_edit_updates_the_saved_markdown_and_preview_url() {
    let mut document = Document::new();
    document.edit("Before\n![photo](first.png)\nAfter".into());
    let mut app = blank();
    app.tabs[0] = Tab::new(0, document);
    app.tabs[0].image_blocks[1]
        .content
        .move_to(text_editor::Cursor {
            position: text_editor::Position {
                line: 0,
                column: 18,
            },
            selection: Some(text_editor::Position { line: 0, column: 9 }),
        });
    let _ = update(
        &mut app,
        Message::EditImageBlock(
            0,
            1,
            text_editor::Action::Edit(text_editor::Edit::Paste(std::sync::Arc::new(
                "second.png".into(),
            ))),
        ),
    );
    assert_eq!(
        app.current().editor.text(),
        "Before\n![photo](second.png)\nAfter"
    );
    assert_eq!(
        app.current().image_blocks[1].url.as_deref(),
        Some("second.png")
    );
    assert!(app.current().document.dirty);
}

#[test]
fn typing_a_standalone_image_line_creates_image_blocks() {
    let mut app = blank();
    for character in "![photo](picture.png)".chars() {
        let _ = update(
            &mut app,
            Message::Edit(
                0,
                text_editor::Action::Edit(text_editor::Edit::Insert(character)),
            ),
        );
    }
    assert_eq!(app.current().editor.text(), "![photo](picture.png)");
    assert_eq!(app.current().image_blocks.len(), 3);
    assert_eq!(
        app.current().image_blocks[1].url.as_deref(),
        Some("picture.png")
    );
}

#[test]
fn arrow_keys_cross_image_blocks_without_changing_source() {
    let mut document = Document::new();
    document.edit("Before\n![photo](picture.png)\nAfter".into());
    let mut app = blank();
    app.tabs[0] = Tab::new(0, document);
    app.tabs[0].image_blocks[0]
        .content
        .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
    let source = app.current().editor.text();
    let _ = update(
        &mut app,
        Message::EditImageBlock(0, 0, text_editor::Action::Move(text_editor::Motion::Down)),
    );
    assert_eq!(app.current().active_image_block, 1);
    app.tabs[0].image_blocks[1]
        .content
        .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
    let _ = update(
        &mut app,
        Message::EditImageBlock(0, 1, text_editor::Action::Move(text_editor::Motion::Down)),
    );
    assert_eq!(app.current().active_image_block, 2);
    assert_eq!(app.current().editor.text(), source);
}

#[test]
fn splitting_multiple_images_keeps_exact_markdown_source() {
    let source = "α before\r\n\r\n![one](one.png)\r\ntext\r\n![two](two.png)\r\nend";
    let mut document = Document::new();
    document.edit(source.into());
    let tab = Tab::new(0, document);
    let blocks = tab
        .image_blocks
        .iter()
        .map(|block| block.content.text())
        .collect::<String>();
    assert_eq!(blocks, source);
}

#[test]
fn edits_around_image_keep_display_blocks_and_source_in_sync() {
    let mut document = Document::new();
    document.edit("Intro\n![x](x.png)\nTail".into());
    let mut tab = Tab::new(0, document);
    tab.image_blocks[0].content.move_to(text_editor::Cursor {
        position: text_editor::Position { line: 0, column: 5 },
        selection: None,
    });
    assert!(tab.perform_image_action(0, text_editor::Action::Edit(text_editor::Edit::Insert('!'))));
    tab.image_blocks[2].content.move_to(text_editor::Cursor {
        position: text_editor::Position { line: 0, column: 0 },
        selection: None,
    });
    assert!(!tab.perform_image_action(2, text_editor::Action::Edit(text_editor::Edit::Backspace)));
    assert!(tab.perform_image_action(2, text_editor::Action::Edit(text_editor::Edit::Insert('β'))));
    let blocks = tab
        .image_blocks
        .iter()
        .map(|block| block.content.text())
        .collect::<String>();
    assert_eq!(tab.editor.text(), "Intro!\n![x](x.png)\nβTail");
    assert_eq!(blocks, tab.editor.text());
}

#[test]
fn replacing_image_syntax_returns_to_the_regular_editor() {
    let mut document = Document::new();
    document.edit("Before\n![x](x.png)\nAfter".into());
    let mut tab = Tab::new(0, document);
    tab.image_blocks[1].content.move_to(text_editor::Cursor {
        position: text_editor::Position {
            line: 0,
            column: 11,
        },
        selection: Some(text_editor::Position { line: 0, column: 0 }),
    });
    assert!(tab.perform_image_action(
        1,
        text_editor::Action::Edit(text_editor::Edit::Paste(std::sync::Arc::new(
            "plain".into(),
        ))),
    ));
    assert_eq!(tab.editor.text(), "Before\nplain\nAfter");
    assert!(tab.image_blocks.is_empty());
}
