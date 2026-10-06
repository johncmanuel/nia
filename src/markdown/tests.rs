use super::*;

#[test]
fn headings_ignore_fences_and_recognize_setext() {
    let parsed = headings(
        "# One\n```md\n# Not a heading\n```\nUnderlined\n---\n  ### Three ###\n####### No\n# Literal#",
    );
    assert_eq!(
        parsed,
        vec![
            Heading {
                level: 1,
                line: 0,
                title: "One".into()
            },
            Heading {
                level: 2,
                line: 4,
                title: "Underlined".into()
            },
            Heading {
                level: 3,
                line: 6,
                title: "Three".into()
            },
            Heading {
                level: 1,
                line: 8,
                title: "Literal#".into()
            },
        ]
    );
}
