# nia

A small native Markdown editor built with Rust and Iced. It supports tab groups with movable panes, search, a document outline, reading mode, focus controls, themes, and installed fonts.

Run `cargo run` for a blank document or `cargo run -- path/to/article.md` to open a Markdown file. The [Handbook](HANDBOOK.md) explains how to use the editor. It is also available offline through the **Handbook** button in the app.

## Development

`cargo test` runs headless `iced_test` interaction and rendering checks. Run `cargo run --features debug-tools` and press F12 to view Iced's debug metrics; a regular build leaves the debugger detached.

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
