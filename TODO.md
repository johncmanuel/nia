# TODO

- [ ] Verify the first GitHub Actions build and tagged release on Linux, Windows, and both macOS architectures after pushing the workflow.
- [ ] Verify that the first merge into `master` after the workflow update publishes a GitHub prerelease with four native archives and checksums.
- [ ] Render Markdown images inline while writing. Read mode already displays local images.
- [ ] Hide Markdown syntax while writing and render formatted text inline, including heading markers (for example, hide `#` and enlarge the heading text). Read mode already renders formatted text.

Read mode uses Iced's Markdown renderer and a custom image viewer. Inline formatting while typing still needs an editable rich surface with cursor mapping and Markdown source preservation.

Synchronization from the design document remains deferred until requested.

## Post project tasks

- [ ] After the project is finalized, build a static HTML, CSS, and JavaScript brochure website that demonstrates it. Add project files under site/
- [ ] Verify tab group switching, pane dragging, pane resizing, restored group layouts after relaunch, typing responsiveness, all-tabs search, installed font rendering, wallpaper transparency, and the optional Iced debug overlay on an interactive macOS desktop. This executor cannot connect to the window service; headless `iced_test` interactions pass.
