# Live Markdown canvas prototype

Run `cargo run --example live_markdown`. This is an isolated, disposable editor
experiment. It does not open or change any nia documents.

The left pane is editable. It draws headings at a larger size and draws simple
`*emphasis*` and `**strong**` text with style. Markdown markers disappear when
the caret moves to another line. The right pane shows the unchanged Markdown
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
images, complete Markdown parsing, viewport scrolling, and exact glyph
measurement. Its approximate character widths are enough to test the mapping
idea but would need real text shaping before production use. The production
editor remains `iced::widget::text_editor` while these gaps are evaluated.
