use super::*;
use iced::widget::text::Highlighter;

#[test]
fn highlights_markdown_without_treating_fenced_headings_as_headings() {
    let mut highlighter = MarkdownHighlighter::new(&Font::MONOSPACE);
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
    highlighter.update(&font);
    let highlighted = highlighter.highlight_line("# Font").collect::<Vec<_>>();
    assert_eq!(highlighted[0].1.1, font);
    assert_eq!(
        format(&highlighted[0].1, &Theme::Dark).font.unwrap().family,
        font.family
    );
}
