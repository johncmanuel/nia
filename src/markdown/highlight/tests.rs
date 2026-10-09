use super::*;
use iced::widget::text::Highlighter;

#[test]
fn highlights_markdown_without_treating_fenced_headings_as_headings() {
    let settings = HighlightSettings {
        active_line: None,
        focus: None,
        highlight_line: false,
    };
    let mut highlighter = MarkdownHighlighter::new(&settings);
    assert_eq!(
        highlighter.highlight_line("# Title").collect::<Vec<_>>(),
        vec![(0..7, Kind::Heading)]
    );
    assert_eq!(
        highlighter.highlight_line("```md").collect::<Vec<_>>(),
        vec![(0..5, Kind::Code)]
    );
    assert_eq!(
        highlighter.highlight_line("# source").collect::<Vec<_>>(),
        vec![(0..8, Kind::Code)]
    );
    highlighter.change_line(1);
    assert_eq!(
        highlighter
            .highlight_line("plain *word* [link](url)")
            .collect::<Vec<_>>(),
        vec![(13..24, Kind::Link)]
    );
    let highlighted = highlighter.highlight_line("# Font").collect::<Vec<_>>();
    assert_eq!(highlighted[0], (0..6, Kind::Heading));
    assert!(format(&highlighted[0].1, &Theme::Dark).font.is_none());
}

#[test]
fn focus_dimming_does_not_hide_other_lines() {
    let settings = HighlightSettings {
        active_line: Some(0),
        focus: Some(crate::focus::Span {
            start: crate::focus::Point { line: 0, byte: 6 },
            end: crate::focus::Point { line: 0, byte: 10 },
        }),
        highlight_line: true,
    };
    let mut highlighter = MarkdownHighlighter::new(&settings);
    let first = highlighter.highlight_line("hello word").collect::<Vec<_>>();
    assert!(first.contains(&(0..10, Kind::Current)));
    assert!(first.contains(&(0..6, Kind::Dim)));
    let second = highlighter.highlight_line("outside").collect::<Vec<_>>();
    assert!(
        second.iter().all(|(_, highlight)| {
            format(highlight, &Theme::Dark)
                .color
                .is_none_or(|color| color.a > 0.0)
        }),
        "a line outside the cursor context can still be inside the editor viewport"
    );
}

#[test]
fn removing_focus_rehighlights_lines_before_the_old_focus() {
    let focused = HighlightSettings {
        active_line: Some(2),
        focus: Some(crate::focus::Span {
            start: crate::focus::Point { line: 2, byte: 0 },
            end: crate::focus::Point { line: 2, byte: 5 },
        }),
        highlight_line: false,
    };
    let mut highlighter = MarkdownHighlighter::new(&focused);
    assert!(
        highlighter
            .highlight_line("dimmed")
            .any(|(_, kind)| kind == Kind::Dim)
    );
    highlighter.highlight_line("dimmed").for_each(drop);
    highlighter.highlight_line("focus").for_each(drop);

    highlighter.update(&HighlightSettings {
        active_line: None,
        focus: None,
        ..focused
    });
    assert_eq!(highlighter.current_line(), 0);
    assert!(
        highlighter
            .highlight_line("visible")
            .all(|(_, kind)| kind != Kind::Dim)
    );
}
