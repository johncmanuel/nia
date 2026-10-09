# TODO

## Before release

- [ ] Merge the final release changes into `master`, confirm its CI checks pass, then tag that commit `v0.1.0` and verify the stable release is marked Latest.
- [ ] Verify the first GitHub Actions build and tagged release on Linux, Windows, and both macOS architectures after pushing the workflow.
- [ ] Verify that the first merge into `master` after the workflow update publishes a GitHub prerelease with four native archives and checksums.
- [ ] On macOS, verify tab group switching and Primary+Shift+W group closing with clean and unsaved documents in multiple splits. Check save, discard, and cancel paths.
- [ ] On macOS, verify pane dragging and resizing, restored groups and layouts after relaunch, the last selected group, each tab's Read or Write mode, long tab titles, and long outline headings. Check outline toggling by mouse and shortcut.
- [ ] On macOS, reopen one file in two splits. Verify typing, cursor and scroll positions, undo, save, close prompts, and relaunch with a large document.
- [ ] On macOS, verify typing responsiveness, all-tabs search, installed fonts, wallpaper transparency and opacity adjustment, and the F12 Iced debug overlay. Headless `iced_test` interactions pass.

## Editor improvements

- [ ] Render extended Markdown forms that Iced's current Markdown viewer leaves as source text: footnotes, definition lists, emoji codes, highlights, subscripts, and superscripts.
- [ ] Evaluate a faster text storage strategy using the plan below. Do not replace the editor's text representation before measurements identify a bottleneck.
- [ ] Evaluate cross-process locking or an atomic conflict check for saving an open file. A file can change between the current comparison and rename.

Writing mode keeps Markdown syntax visible. Read mode uses Iced's Markdown renderer and a custom image viewer.

Synchronization from the design document remains deferred until requested.

## Text storage evaluation plan

**Current choice:** Keep Iced's `text_editor::Content` as the live editing buffer. Its underlying Cosmic Text buffer already stores a vector of lines. Iced does not accept a rope or piece table as a replacement for `Content`, so adding one now would duplicate the document and require full-text conversions. If a custom editing surface becomes necessary, prototype a **rope** first: it supports edits and line lookup together, which suits cursor movement, Markdown, and the outline. A piece table is the fallback to compare if undo history or edit memory becomes the measured bottleneck; fast line lookup needs an additional index or tree.

1. Establish a baseline in a release build and with Iced's F12 debug view. For 1 MB, 10 MB, and 100 MB documents, measure p50/p95/p99 insertion, deletion, cursor navigation, full app update, and frame layout at the start, middle, and end. Include Unicode, very long lines, many short lines, and documents with image links. Record peak memory and allocations per edit. Treat typing latency as the first priority when evaluating tradeoffs.
2. Attribute the time before changing storage. In particular, measure `Content::text()` and its per-line copies, `Document.contents` and `saved_contents`, `sync_document`, outline generation while its panel is open, and all-tabs search. Measure save and recovery separately from typing.
3. Remove only measured work on the hot path first, such as repeated full-text extraction or rebuilding derived views on every frame. Rerun the same workload and confirm cursor behavior, Markdown source, search, undo/redo, autosave, recovery, and memory use.
4. If typing still misses the chosen frame budget, prototype a rope-backed editor behind a small benchmark, including byte/Unicode cursor mapping and line access. Compare it with the current Iced editor under identical workloads. Prototype a piece table only if the measurements indicate a meaningful advantage for this editor. Do not keep a second authoritative text buffer merely to introduce a new data structure.
5. Migrate only when the end-to-end typing improvement outweighs the cost of replacing Iced's editor behavior. Record benchmark results and the decision here before implementation.

## After the project is finalized

- [ ] Build a static HTML, CSS, and JavaScript brochure website that demonstrates nia. Add project files under site/.
