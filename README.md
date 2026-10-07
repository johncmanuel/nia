# nia

A small native Markdown editor built with Rust and Iced.

Run `cargo run` for a blank document or `cargo run -- path/to/article.md` to open a Markdown file. You can also use Open, the recent-file picker, or drop a file onto the window. Files are ordinary `.md` documents.

Open documents appear as tabs in one window. The compact **Layout** control reveals **Single**, **Split right**, and **Split down**; an already open tab fills a new pane when available. Click a pane, then a tab to place that document there. If it is visible in another pane, the two documents swap places. Drag a pane by its title bar to rearrange the layout, drag a divider to resize it, and use the close button on a pane to hide it without closing its tab. Closing an unsaved tab or quitting prompts you to save, discard, or cancel.

Run `cargo run --features debug-tools` and press **F12** to open Iced's debug metrics. Regular `cargo run` leaves the debugger detached. `cargo test` runs headless `iced_test` interaction and rendering checks without opening a desktop window.

The editor shows restrained color for Markdown headings, emphasis, links, lists, quotes, and code while leaving the source editable. Named files autosave after a short pause; the word count refreshes after that pause as well. **Find** searches the active document in the tab bar without moving the editor, with a case switch. **Outline** opens a left sidebar of the current document's headings; filter them, click one to jump, or use Enter for the selected heading. The sidebar stays open when you switch tabs. Click **Hide**, press Escape, or use the Outline shortcut again to close it. **Recent** filters previously opened files.

Click **Read** to see formatted Markdown and local images, then **Write** to return to the same editable text and cursor. Relative image paths resolve from the Markdown file's folder, so save an untitled document before using them. Remote images are not loaded. Clicking a link in Read mode copies its address.

On macOS, Cmd+F opens Find in the current tab, and Cmd+Shift+F searches all open tabs, including hidden tabs and unsaved edits. Click a result to open its tab and select the match; Enter and the arrow buttons cycle through matches. Ctrl+F is an alternate for current-tab Find. Cmd+Shift+O opens Outline, Cmd+P opens Recent, and Cmd+N/O/S create, open, and save. Ctrl+S also saves on macOS. Cmd+, opens Settings. Browser-style shortcuts use Cmd+T for a new tab, Cmd+W to close a tab, Ctrl+Tab and Ctrl+Shift+Tab to cycle tabs, and Cmd+1-9 to select a tab (9 selects the last). Ctrl+Shift+L/R/D switches to one pane, splits right, or splits down. Settings lets you change these shortcuts, except Cmd+1-9. One Escape closes Find or another focused picker.

Windows and Linux use Ctrl in place of Cmd for these defaults, including Ctrl+S, Ctrl+F, Ctrl+Shift+F, Ctrl+comma, and Ctrl+1-9. Ctrl+Tab and the layout shortcuts use Control on every platform. New installations and **Restore defaults** select bindings for the current OS; saved custom bindings are preserved.

Open **Settings** in the footer to choose a theme, an installed editor font, or the default monospace font. You can also change font size and turn on a translucent writing window to see your desktop wallpaper on macOS and Linux. These choices are saved for future launches. Markdown highlighting follows the selected font.

Use **Zen** in the footer or Cmd+Shift+J (Ctrl+Shift+J elsewhere) to show only the active document; press Escape to restore the window controls. Zen mode can also be toggled in Settings without leaving the Settings page. Typewriter scrolling (Cmd+Shift+Y, or Ctrl+Shift+Y elsewhere) keeps the current line near a chosen position, with controls for visible lines above and below it. You can highlight the current line and dim other paragraphs or sentences. Focus settings and shortcuts are saved. Iced currently scrolls by logical lines, so long wrapped lines and the beginning or end of a document can shift the exact caret position.

nia checks named files for external edits. It reloads a clean document automatically; when local edits exist, it preserves them and offers Save As. Unsaved drafts are stored locally for recovery after an interrupted session and are reopened as unsaved tabs. See [PRIVACY.md](PRIVACY.md) for where settings, recent paths, recovery drafts, and logs are stored.

## Builds and releases

[Build and release](.github/workflows/build.yml) checks formatting, compilation, Clippy, and tests (including `iced_test`) on Linux x64, Windows x64, and both Intel and Apple Silicon macOS. It runs for pull requests, pushes to `master`, version tags, and manual dispatches. Version tags and manual runs also build and upload optimized native archives and SHA-256 checksums. Pull requests and ordinary pushes skip these optimized builds; all checks and tests still run. CI omits debug symbols for development/test builds and caches Rust dependencies; production optimization settings are unchanged.

To publish a production GitHub Release, update the stable `X.Y.Z` version in `Cargo.toml`, refresh and commit `Cargo.lock` with the release changes, then push the matching tag:

```sh
git tag vX.Y.Z
git push origin vX.Y.Z
```

The tag must match the manifest. All four targets must pass before the `production` job creates a draft, uploads the verified archives, and publishes it. Only version-tag pushes publish; manual runs produce artifacts, while ordinary pushes run checks. The workflow uses the repository's `GITHUB_TOKEN`, so no personal token is required. Any protection rules configured on the `production` GitHub environment also apply. If publication fails after creating a draft, inspect and delete that incomplete draft before rerunning; existing releases are never overwritten.

macOS ZIPs include a `nia.app` bundle (macOS 11 or later); Windows ZIPs include `nia.exe`; Linux tarballs include `nia` (built on Ubuntu 24.04, requiring compatible system libraries and a graphical desktop). Archives include the README, license, and privacy documentation. Extract the archive before launching the app. macOS bundles and Windows executables are not signed or notarized; trusted distribution requires separately configured signing credentials.

To test a local production build, run `cargo build --locked --release --target YOUR_TARGET`. Use the workflow's manual dispatch to generate release archives without publishing a GitHub Release.

### Dependency scope

| Scope | Dependencies | Purpose |
| --- | --- | --- |
| Production | `iced`, `rfd`, `log`, `fontdb` | Editor UI/rendering, native file dialogs, logging, installed fonts |
| Tests only | `iced_test`, direct `png` dependency | Headless UI tests and screenshot pixel checks |
| Opt-in development | `debug-tools` feature | Enables Iced's debugging overlay; release builds leave it off |

Iced's GPU renderer, software fallback, image codecs, Markdown support, and Linux X11/Wayland backends are runtime features. PNG decoding also appears transitively in production through image rendering, even though our direct `png` dependency is test-only. Iced includes its `iced_debug` integration transitively; the enabled debugging overlay is controlled by `debug-tools`. Packaging uses native archive tools in the workflow; CI has no Python requirement.
