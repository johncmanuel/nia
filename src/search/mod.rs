use std::ops::Range;

pub(crate) fn matches(line: &str, query: &str, match_case: bool) -> Vec<Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }
    if match_case {
        return line
            .match_indices(query)
            .map(|(start, _)| start..start + query.len())
            .collect();
    }

    // scan Unicode scalar values. index matches if profiling shows long queries cost time.
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < line.len() {
        let rest = &line[offset..];
        let mut characters = rest.chars();
        let mut length = 0;
        let found = query.chars().all(|expected| {
            if let Some(actual) = characters.next() {
                length += actual.len_utf8();
                actual.to_lowercase().eq(expected.to_lowercase())
            } else {
                false
            }
        });
        if found {
            result.push(offset..offset + length);
            offset += length;
        } else {
            offset += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
    result
}

#[cfg(test)]
mod tests {
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
}
