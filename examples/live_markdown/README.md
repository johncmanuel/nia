# Live Markdown canvas prototype

Run `cargo run --example live_markdown`. This is an isolated, disposable editor
experiment. It does not open or change any nia documents.

The left pane is editable. It tries headings, bold, italics, strikethrough,
highlight, inline and fenced code, links, image labels, quotes, ordered and
unordered lists, tasks, rules, tables, footnotes, definitions, four emoji
shortcodes, subscripts, and superscripts. Markdown markers disappear when the
caret moves to another line. The right pane shows the unchanged Markdown
source. Click and drag in the left pane to test source-to-display cursor and
selection mapping. `cargo test --example live_markdown` checks the mapping and
an `iced_test` click-and-type interaction.

We inspected `iced-code-editor` 0.5.2 before making this. Its canvas rendering
code positions syntax-colored spans from the original buffer, and its wrapping
logic uses source character columns. It has no public hook for replacing a
source line with a different visual line or changing font size for one span.
The prototype therefore uses Iced's canvas directly and does not keep
`iced-code-editor` as a dependency. Its source is a useful reference for
rendering, wrapping, and focus management.

This prototype deliberately omits persistence, paste, IME composition, undo,
actual image rendering, complete Markdown parsing, and exact glyph measurement.
Tables are styled rows, not aligned cells. Images display an icon and alt text.
Only `:joy:`, `:smile:`, `:heart:`, and `:check:` become emoji. Its approximate
character widths are enough to test the mapping idea but would need real text
shaping before production use. The production editor remains
`iced::widget::text_editor` while these gaps are evaluated.
