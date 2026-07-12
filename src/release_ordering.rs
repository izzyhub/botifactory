// pattern: Functional Core

use semver::Version;

/// The fields of a release row that matter for picking latest/previous.
/// Ordering is semver first, then row recency, so re-uploads of the same
/// version resolve to the newest row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCandidate {
    pub id: i64,
    pub version: Version,
    pub created_at: i64,
}

fn sort_key(candidate: &ReleaseCandidate) -> (&Version, i64, i64) {
    (&candidate.version, candidate.created_at, candidate.id)
}

/// The release with the highest version (not the most recently uploaded one:
/// uploading an old version must not make devices "upgrade" backwards).
pub fn latest_release(candidates: &[ReleaseCandidate]) -> Option<&ReleaseCandidate> {
    candidates.iter().max_by_key(|candidate| sort_key(candidate))
}

/// The newest release with a version strictly lower than the latest — the
/// release a device would roll back to. Returns None when no strictly lower
/// version exists (a single release, or every release at the same version).
pub fn previous_release(candidates: &[ReleaseCandidate]) -> Option<&ReleaseCandidate> {
    let latest = latest_release(candidates)?;
    candidates
        .iter()
        .filter(|candidate| candidate.version < latest.version)
        .max_by_key(|candidate| sort_key(candidate))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: i64, version: &str, created_at: i64) -> ReleaseCandidate {
        ReleaseCandidate {
            id,
            version: Version::parse(version).expect("test version should parse"),
            created_at,
        }
    }

    #[test]
    fn latest_is_highest_semver_not_newest_upload() {
        // 1.0.1 was uploaded after 1.1.0 (e.g. a backport); latest must stay 1.1.0.
        let releases = [
            candidate(1, "1.0.0", 100),
            candidate(2, "1.1.0", 200),
            candidate(3, "1.0.1", 300),
        ];
        assert_eq!(latest_release(&releases).map(|c| c.id), Some(2));
    }

    #[test]
    fn latest_prefers_newest_row_for_same_version() {
        let releases = [candidate(1, "1.0.0", 100), candidate(2, "1.0.0", 200)];
        assert_eq!(latest_release(&releases).map(|c| c.id), Some(2));
    }

    #[test]
    fn latest_orders_prerelease_identifiers_numerically() {
        let releases = [
            candidate(1, "0.0.1-alpha.6", 100),
            candidate(2, "0.0.1-alpha.21", 50),
        ];
        assert_eq!(latest_release(&releases).map(|c| c.id), Some(2));
    }

    #[test]
    fn previous_is_highest_strictly_lower_version() {
        let releases = [
            candidate(1, "1.0.0", 100),
            candidate(2, "1.1.0", 200),
            candidate(3, "1.0.1", 300),
        ];
        assert_eq!(previous_release(&releases).map(|c| c.id), Some(3));
    }

    #[test]
    fn previous_is_none_with_single_release() {
        let releases = [candidate(1, "1.0.0", 100)];
        assert_eq!(previous_release(&releases), None);
    }

    #[test]
    fn previous_is_none_when_all_versions_equal() {
        let releases = [candidate(1, "1.0.0", 100), candidate(2, "1.0.0", 200)];
        assert_eq!(previous_release(&releases), None);
    }

    #[test]
    fn empty_input_yields_none() {
        assert_eq!(latest_release(&[]), None);
        assert_eq!(previous_release(&[]), None);
    }
}
