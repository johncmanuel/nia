use super::*;

#[test]
fn focus_settings_round_trip_and_migrate_typewriter_settings() -> io::Result<()> {
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
            dimming: Dimming::Sentence,
        };
        settings.save_to(&path)?;
        assert_eq!(Settings::load_from(&path)?, settings);
        fs::write(&path, "true\ntrue\n4\n6\nSentence\n")?;
        assert_eq!(Settings::load_from(&path)?, settings);
        fs::write(&path, "true\nUnknown\n")?;
        assert_eq!(
            Settings::load_from(&path).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        Ok(())
    })();
    let _ = fs::remove_file(path);
    result
}

#[test]
fn paragraph_and_sentence_focus_follow_the_caret_across_lines() {
    let mut editor =
        text_editor::Content::with_text("First sentence. Another\nsentence!\n\nNext paragraph.");
    editor.move_to(text_editor::Cursor {
        position: text_editor::Position { line: 1, column: 3 },
        selection: None,
    });
    assert_eq!(
        active_span(&editor, Dimming::Paragraph),
        Some(Span {
            start: Point { line: 0, byte: 0 },
            end: Point { line: 1, byte: 9 },
        })
    );
    assert_eq!(
        active_span(&editor, Dimming::Sentence),
        Some(Span {
            start: Point { line: 0, byte: 16 },
            end: Point { line: 1, byte: 9 },
        })
    );
    editor.move_to(text_editor::Cursor {
        position: text_editor::Position { line: 2, column: 0 },
        selection: None,
    });
    assert_eq!(
        active_span(&editor, Dimming::Paragraph),
        Some(Span {
            start: Point { line: 2, byte: 0 },
            end: Point { line: 2, byte: 0 },
        })
    );
}
