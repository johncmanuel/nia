# nia

A small native Markdown editor built with Rust and Iced.

Run `cargo run` for a blank document or `cargo run -- path/to/article.md` to open a Markdown file. You can also use Open, the recent-file picker, or drop a file onto the window. Files are ordinary `.md` documents.

Open documents appear as tabs in one window. The compact **Layout** control reveals **Single**, **Split right**, and **Split down**; an already open tab fills a new pane when available. Click a pane, then a tab to place that document there. If it is visible in another pane, the two documents swap places. Drag a pane by its title bar to rearrange the layout, drag a divider to resize it, and use the close button on a pane to hide it without closing its tab. Closing an unsaved tab or quitting prompts you to save, discard, or cancel.

Run `cargo run --features debug-tools` and press **F12** to open Iced's debug metrics. Regular `cargo run` leaves the debugger detached. `cargo test` runs headless `iced_test` interaction checks, including pane splitting and editing, without taking screenshots.

The editor shows restrained color for Markdown headings, emphasis, links, lists, quotes, and code while leaving the source editable. Named files autosave after a short pause; the word count refreshes after that pause as well. **Find** searches the active document in the tab bar without moving the editor, with a case switch; **Outline** filters headings and jumps to the selected one; **Recent** filters previously opened files. Arrow keys and Enter work in the pickers.

On macOS, Cmd+F opens Find in the current tab, and Cmd+Shift+F searches all open tabs, including hidden tabs and unsaved edits. Click a result to open its tab and select the match; Enter and the arrow buttons cycle through matches. Ctrl+F is an alternate for current-tab Find. Cmd+Shift+O opens Outline, Cmd+P opens Recent, and Cmd+N/O/S create, open, and save. Ctrl+S also saves on macOS. Browser-style shortcuts use Cmd+T for a new tab, Cmd+W to close a tab, Ctrl+Tab and Ctrl+Shift+Tab to cycle tabs, and Cmd+1-9 to select a tab (9 selects the last). Ctrl+Shift+L/R/D switches to one pane, splits right, or splits down. The Keybindings page lets you change these shortcuts, except Cmd+1-9. One Escape closes Find or another focused picker.

Use **Font** in the footer to choose an installed font family for the editor, or return to the default monospace font. The setting is saved for future launches. Markdown highlighting follows the selected font.

nia checks named files for external edits. It reloads a clean document automatically; when local edits exist, it preserves them and offers Save As. Unsaved drafts are stored locally for recovery after an interrupted session and are reopened as unsaved tabs. See [PRIVACY.md](PRIVACY.md) for where settings, recent paths, recovery drafts, and logs are stored.
