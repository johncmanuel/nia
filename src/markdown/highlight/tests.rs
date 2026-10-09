use super::*;
use iced::widget::text::Highlighter;

#[test]
fn highlights_markdown_without_treating_fenced_headings_as_headings() {
    let settings = HighlightSettings {
        active_line: None,
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
fn current_line_highlight_does_not_affect_other_lines() {
    let settings = HighlightSettings {
        active_line: Some(0),
        highlight_line: true,
    };
    let mut highlighter = MarkdownHighlighter::new(&settings);
    let first = highlighter.highlight_line("hello word").collect::<Vec<_>>();
    assert!(first.contains(&(0..10, Kind::Current)));
    let second = highlighter.highlight_line("outside").collect::<Vec<_>>();
    assert!(second.is_empty());
}

#[test]
fn switching_active_line_rehighlights_from_previous_line() {
    let highlighted = HighlightSettings {
        active_line: Some(2),
        highlight_line: true,
    };
    let mut highlighter = MarkdownHighlighter::new(&highlighted);
    highlighter.highlight_line("first").for_each(drop);
    highlighter.highlight_line("second").for_each(drop);
    assert!(
        highlighter
            .highlight_line("third")
            .any(|(_, kind)| kind == Kind::Current)
    );

    highlighter.update(&HighlightSettings {
        active_line: None,
        ..highlighted
    });
    assert_eq!(highlighter.current_line(), 2);
    assert!(
        highlighter
            .highlight_line("third")
            .all(|(_, kind)| kind != Kind::Current)
    );
}
