//! Version comparison, independent of where a version came from.
//!
//! Compares as semver when possible (stripping a leading `v`), and falls back
//! to a plain string difference for tags that are not valid semver.

/// Compare a discovered version against the already-known one.
/// Returns true if `fetched` is newer than `known`.
///
/// If `known` is None, any discovered version counts as new. We first try to
/// compare as semver (stripping a leading 'v'); if either side fails to parse,
/// we compare as strings (and treat it as new only on an actual difference).
pub fn is_newer(fetched: &str, known: Option<&str>) -> bool {
    let known = match known {
        None => return true,
        Some(k) => k,
    };
    if fetched == known {
        return false;
    }

    match (parse_semver(fetched), parse_semver(known)) {
        (Some(f), Some(k)) => f > k,
        // Not semver — since the strings differ, treat it as new.
        _ => true,
    }
}

/// Parse a tag as semver, stripping an optional leading 'v'.
fn parse_semver(tag: &str) -> Option<semver::Version> {
    let trimmed = tag.strip_prefix('v').unwrap_or(tag);
    semver::Version::parse(trimmed).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_when_unknown() {
        assert!(is_newer("v1.0.0", None));
    }

    #[test]
    fn equal_is_not_newer() {
        assert!(!is_newer("v1.2.3", Some("v1.2.3")));
        assert!(!is_newer("1.2.3", Some("1.2.3")));
    }

    #[test]
    fn semver_comparison() {
        assert!(is_newer("v1.2.4", Some("v1.2.3")));
        assert!(is_newer("v2.0.0", Some("v1.9.9")));
        assert!(!is_newer("v1.2.3", Some("v1.2.4")));
        // with and without 'v' is equivalent
        assert!(is_newer("1.2.4", Some("v1.2.3")));
    }

    #[test]
    fn non_semver_falls_back_to_string_diff() {
        // dates / non-standard tags: any difference counts as new
        assert!(is_newer("2024-05-01", Some("2024-04-01")));
        assert!(!is_newer("nightly", Some("nightly")));
        assert!(is_newer("release-42", Some("release-41")));
    }
}
