use super::*;
use iced::widget::text::Highlighter;

#[test]
fn highlights_markdown_without_treating_fenced_headings_as_headings() {
    let settings = HighlightSettings {
        font: Font::MONOSPACE,
        active_line: None,
        context: None,
        focus: None,
        highlight_line: false,
    };
    let mut highlighter = MarkdownHighlighter::new(&settings);
    assert_eq!(
        highlighter.highlight_line("# Title").collect::<Vec<_>>(),
        vec![(0..7, (Kind::Heading, Font::MONOSPACE))]
    );
    assert_eq!(
        highlighter.highlight_line("```md").collect::<Vec<_>>(),
        vec![(0..5, (Kind::Code, Font::MONOSPACE))]
    );
    assert_eq!(
        highlighter.highlight_line("# source").collect::<Vec<_>>(),
        vec![(0..8, (Kind::Code, Font::MONOSPACE))]
    );
    highlighter.change_line(1);
    assert_eq!(
        highlighter
            .highlight_line("plain *word* [link](url)")
            .collect::<Vec<_>>(),
        vec![
            (6..12, (Kind::Emphasis, Font::MONOSPACE)),
            (13..24, (Kind::Link, Font::MONOSPACE))
        ]
    );
    let font = Font::with_name("Example Family");
    highlighter.update(&HighlightSettings { font, ..settings });
    let highlighted = highlighter.highlight_line("# Font").collect::<Vec<_>>();
    assert_eq!(highlighted[0].1.1, font);
    assert_eq!(
        format(&highlighted[0].1, &Theme::Dark).font.unwrap().family,
        font.family
    );
}

#[test]
fn focus_dimming_and_context_apply_after_markdown_colors() {
    let settings = HighlightSettings {
        font: Font::MONOSPACE,
        active_line: Some(0),
        context: Some((0, 0)),
        focus: Some(crate::focus::Span {
            start: crate::focus::Point { line: 0, byte: 6 },
            end: crate::focus::Point { line: 0, byte: 10 },
        }),
        highlight_line: true,
    };
    let mut highlighter = MarkdownHighlighter::new(&settings);
    let first = highlighter.highlight_line("hello word").collect::<Vec<_>>();
    assert!(first.contains(&(0..10, (Kind::Current, Font::MONOSPACE))));
    assert!(first.contains(&(0..6, (Kind::Dim, Font::MONOSPACE))));
    let second = highlighter.highlight_line("outside").collect::<Vec<_>>();
    assert_eq!(
        second.last(),
        Some(&(0..7, (Kind::Hidden, Font::MONOSPACE)))
    );
}
