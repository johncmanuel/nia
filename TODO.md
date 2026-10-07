# TODO

- [ ] Verify the first GitHub Actions build and tagged release on Linux, Windows, and both macOS architectures after pushing the workflow.
- [ ] Render Markdown images inline while writing. Read mode already displays local images.
- [ ] Hide Markdown syntax while writing and render formatted text inline, including heading markers (for example, hide `#` and enlarge the heading text). Read mode already renders formatted text.
- [ ] Make typewriter caret positioning exact for wrapped lines and document edges. Iced's text editor exposes relative line scrolling but not direct caret viewport positioning.

Read mode uses Iced's Markdown renderer and a custom image viewer. Inline formatting while typing still needs an editable rich surface with cursor mapping and Markdown source preservation.

Synchronization from the design document remains deferred until requested.

## Post project tasks

- [ ] After the project is finalized, build a static HTML, CSS, and JavaScript brochure website that demonstrates it. Add project files under site/
- [ ] Verify pane dragging, pane resizing, typing responsiveness, all-tabs search, installed font rendering, wallpaper transparency, and the optional Iced debug overlay on an interactive macOS desktop. This executor cannot connect to the window service; headless `iced_test` interactions pass.
