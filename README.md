# nia

A small native Markdown editor built with Rust and Iced. It supports tab groups with movable panes, search, a document outline, reading mode, focus controls, themes, and installed fonts.

Run `cargo run` for a blank document or `cargo run -- path/to/article.md` to open a Markdown file. The [Handbook](HANDBOOK.md) explains how to use the editor. It is also available offline through the **Handbook** button in the app.

## Development

`cargo test` runs headless `iced_test` interaction and rendering checks. Run `cargo run` and press F12 to view Iced's debug metrics. Use `NIA_LOG=debug cargo run` to see more diagnostic events on standard error. Production builds use `cargo build --release --no-default-features` to omit the debugger.

### Environment variables

The app reads these variables:

| Variable | Use |
| --- | --- |
| `NIA_LOG=debug` | Print additional diagnostic events on standard error. Other values use the default log level. Document text is not logged. |
| `HOME` | Find the settings directory on macOS; fallback on Linux. |
| `XDG_CONFIG_HOME` | Find the settings directory on Linux when set. |
| `APPDATA` | Find the settings directory on Windows. |

Snapshot tests also read these variables. Set a snapshot variable to a writable PNG path before running its test.

| Variable | Snapshot |
| --- | --- |
| `NIA_SNAPSHOT` | Main editor |
| `NIA_LAYOUT_SNAPSHOT` | Split layout |
| `NIA_LAYOUT_SNAPSHOT_THEME=light` | Use a light theme for the layout snapshot |
| `NIA_GROUP_SNAPSHOT` | Tab groups |
| `NIA_READING_SNAPSHOT` | Reading mode |
| `NIA_IMAGE_SNAPSHOT` | Image in reading mode |
| `NIA_INLINE_IMAGE_SNAPSHOT` | Image while writing |
| `NIA_OPENED_SNAPSHOT` | Opened document |
| `NIA_SETTINGS_SNAPSHOT` | Settings |
| `NIA_THEME_SNAPSHOT` | Theme menu |
| `NIA_ZEN_SNAPSHOT` | Zen mode |

Before submitting changes, run:

```sh
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Builds and releases

The [build workflow](.github/workflows/build.yml) checks Linux, Windows, Intel macOS, and Apple Silicon macOS. Each push to `master`, including a squash merge, builds native archives and publishes a GitHub prerelease after all targets pass. A version tag matching the stable `X.Y.Z` version in `Cargo.toml` publishes a stable release. Manual workflow runs build downloadable archives without publishing a release. Update `Cargo.lock` with release version changes before tagging.

macOS archives include `nia.app` (macOS 11 or later), Windows archives include `nia.exe`, and Linux archives include `nia` (built on Ubuntu 24.04). Extract the archive before launching. The app is not signed or notarized.

See [PRIVACY.md](PRIVACY.md) for local data and logging details and [LICENSE](LICENSE) for the license.
