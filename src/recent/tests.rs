use super::*;

#[test]
fn recent_files_round_trip_and_deduplicate() {
    let mut paths = vec![];
    remember(&mut paths, Path::new("/tmp/one.md"));
    remember(&mut paths, Path::new("/tmp/two\nlines.md"));
    remember(&mut paths, Path::new("/tmp/one.md"));
    assert_eq!(paths.len(), 2);
    assert_eq!(decode(&encode(&paths)), paths);
}
