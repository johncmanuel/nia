use super::*;

#[test]
fn case_toggle_and_unicode_preserve_original_offsets() {
    assert_eq!(
        matches("🌍 Alpha alpha", "alpha", false),
        vec![5..10, 11..16]
    );
    assert_eq!(matches("🌍 Alpha alpha", "alpha", true), vec![11..16]);
    assert_eq!(matches("Straße STRAẞE", "straße", false), vec![0..7, 8..16]);
}
