use super::*;

#[test]
fn notices_match_error_kinds_without_discarding_edit_guidance() {
    let conflict = FileFailure::from(io::Error::from(ErrorKind::AlreadyExists));
    assert!(conflict.save_notice().contains("Save As"));
    let full = FileFailure::from(io::Error::from(ErrorKind::StorageFull));
    assert!(full.save_notice().contains("Storage is full"));
    assert!(full.recovery_notice().contains("not backed up"));
    let missing = FileFailure::from(io::Error::from(ErrorKind::NotFound));
    assert!(missing.open_notice().contains("not found"));
    assert!(missing.external_notice().contains("moved or deleted"));
}
