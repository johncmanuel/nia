# nia

A small native Markdown editor built with Rust and Iced.

Run `cargo run` to open a blank document, or `cargo run -- path/to/article.md` to open a file directly. You can also use the native Open dialog or drop a file onto the window.

Use New, Open, and Save in the status bar, or the platform's Command/Ctrl+N, Command/Ctrl+O, and Command/Ctrl+S shortcuts. The first save of a new document opens a native Save dialog. Named documents autosave after a short pause in typing.

The editor refuses to overwrite a file that changed on disk since it was opened or last saved. If that happens, your edits remain in the window so you can copy them elsewhere.
