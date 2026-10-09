use super::{Style, project};

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
