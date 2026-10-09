use super::{LineKind, Style, project};

fn visible(source: &str) -> (LineKind, String) {
    let lines = project(&format!("{source}\nother"), 1);
    (
        lines[0].kind,
        lines[0].glyphs.iter().map(|glyph| glyph.ch).collect(),
    )
}

#[test]
fn hides_markers_without_changing_source_offsets() {
    let source = "# Café **bold** and *soft*\nplain";
    let lines = project(source, 1);
    let visible: String = lines[0].glyphs.iter().map(|glyph| glyph.ch).collect();
    assert_eq!(visible, "Café bold and soft");
    assert_eq!(lines[0].glyphs[0].start, 2);
    assert_eq!(lines[0].glyphs[3].start, 5);
    assert!(
        lines[0]
            .glyphs
            .iter()
            .any(|glyph| glyph.style == Style::Strong)
    );
    assert!(
        lines[0]
            .glyphs
            .iter()
            .any(|glyph| glyph.style == Style::Emphasis)
    );
    assert_eq!(lines[1].start, source.find("plain").unwrap());
}

#[test]
fn reveals_markers_on_active_line() {
    let lines = project("# Hello *world*", 0);
    let visible: String = lines[0].glyphs.iter().map(|glyph| glyph.ch).collect();
    assert_eq!(visible, "# Hello *world*");
    assert_eq!(lines[0].glyphs[0].style, Style::Marker);
}

#[test]
fn unmatched_marker_stays_visible() {
    let lines = project("start *unfinished", 1);
    let visible: String = lines[0].glyphs.iter().map(|glyph| glyph.ch).collect();
    assert_eq!(visible, "start *unfinished");
}

#[test]
fn inline_forms_hide_source_markers_and_keep_content() {
    let (_, rendered) =
        visible("`code` [link](https://x.test) ~~old~~ ==new== H~2~O X^2^ [^1] :joy:");
    assert_eq!(rendered, "code link old new H2O X2 1 😂");
    assert_eq!(visible("![alt](photo.png)").1, "▣alt");
    assert_eq!(visible("\\*literal*").1, "\\*literal*");
}

#[test]
fn block_forms_have_readable_visuals() {
    assert_eq!(visible("> quote"), (LineKind::Quote, "quote".into()));
    assert_eq!(visible("- item"), (LineKind::List, "• item".into()));
    assert_eq!(visible("1. item"), (LineKind::List, "1. item".into()));
    assert_eq!(visible("- [x] done"), (LineKind::Task, "☑ done".into()));
    assert_eq!(visible("- [ ] todo"), (LineKind::Task, "☐ todo".into()));
    assert_eq!(visible("---"), (LineKind::Rule, String::new()));
    assert_eq!(
        visible(": meaning"),
        (LineKind::Definition, "→ meaning".into())
    );
    assert_eq!(
        visible("[^1]: note"),
        (LineKind::Footnote, "1: note".into())
    );
    assert_eq!(
        visible("[^café]: note"),
        (LineKind::Footnote, "café: note".into())
    );
}

#[test]
fn fenced_code_and_tables_do_not_parse_inline_markers() {
    let lines = project(
        "```rust\n# literal *code*\n```\n| Name | Value |\n| --- | --- |",
        8,
    );
    assert_eq!(lines[0].kind, LineKind::Fence);
    assert_eq!(
        lines[0]
            .glyphs
            .iter()
            .map(|glyph| glyph.ch)
            .collect::<String>(),
        "rust"
    );
    assert_eq!(lines[1].kind, LineKind::Code);
    assert_eq!(
        lines[1]
            .glyphs
            .iter()
            .map(|glyph| glyph.ch)
            .collect::<String>(),
        "# literal *code*"
    );
    assert_eq!(lines[3].kind, LineKind::Table);
    assert_eq!(
        lines[3]
            .glyphs
            .iter()
            .map(|glyph| glyph.ch)
            .collect::<String>(),
        "│ Name │ Value │"
    );
    assert_eq!(lines[4].kind, LineKind::TableRule);
    assert!(lines[4].glyphs.is_empty());
}

#[test]
fn projected_glyph_ranges_are_valid_utf8_boundaries() {
    let source = "- [x] Café :joy:\n[^café]: [link](https://x.test) H~2~O";
    for line in project(source, 9) {
        for glyph in line.glyphs {
            assert!(glyph.start < glyph.end);
            assert!(source.is_char_boundary(glyph.start));
            assert!(source.is_char_boundary(glyph.end));
        }
    }
}
