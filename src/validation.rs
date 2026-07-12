// pattern: Functional Core

use semver::Version;
use thiserror::Error;

pub const MAX_NAME_LENGTH: usize = 64;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("name cannot be empty")]
    EmptyName,
    #[error("name is too long ({length} chars, max {MAX_NAME_LENGTH})")]
    NameTooLong { length: usize },
    #[error("name contains a character that is not allowed: {character:?}")]
    ForbiddenCharacter { character: char },
    #[error("name cannot start or end with a space")]
    EdgeWhitespace,
    #[error("invalid semver version: {0}")]
    InvalidVersion(#[from] semver::Error),
}

/// A project or channel name that is safe to embed in a filesystem path.
///
/// Parse, don't validate: the only way to construct one is through `parse`,
/// which rejects every character that could influence path resolution
/// (separators, dots, and anything outside a small allowlist), so a
/// `ResourceName` can never contain a `..` segment or escape a base
/// directory it is joined onto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceName(String);

impl ResourceName {
    pub fn parse(name: &str) -> Result<Self, ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyName);
        }
        let length = name.chars().count();
        if length > MAX_NAME_LENGTH {
            return Err(ValidationError::NameTooLong { length });
        }
        if name.starts_with(' ') || name.ends_with(' ') {
            return Err(ValidationError::EdgeWhitespace);
        }
        for character in name.chars() {
            let is_allowed = character.is_ascii_alphanumeric()
                || character == ' '
                || character == '-'
                || character == '_';
            if !is_allowed {
                return Err(ValidationError::ForbiddenCharacter { character });
            }
        }
        Ok(ResourceName(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Parses a version string as strict semver.
///
/// The canonical form (`version.to_string()`) is what gets embedded in
/// filenames: semver identifiers are limited to ASCII alphanumerics, `-`,
/// `.`, and `+`, and dot-separated identifiers cannot be empty, so the
/// rendered form can never contain a path separator or a `..` segment.
pub fn parse_version(version: &str) -> Result<Version, ValidationError> {
    Ok(Version::parse(version)?)
}

/// Constant-time token comparison so a caller cannot probe the API token
/// byte-by-byte through response timing. (Length mismatch returns early;
/// leaking the token's length is acceptable.)
pub fn token_matches(expected: &str, provided: &str) -> bool {
    if expected.len() != provided.len() {
        return false;
    }
    expected
        .bytes()
        .zip(provided.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_names() {
        for name in ["bb-bot", "BB bot", "wave_mech", "stable", "a"] {
            assert!(ResourceName::parse(name).is_ok(), "should accept {name:?}");
        }
    }

    #[test]
    fn rejects_path_traversal_shapes() {
        for name in ["..", "../up", "a/b", "a\\b", "/etc", ".hidden", "a.b"] {
            assert!(ResourceName::parse(name).is_err(), "should reject {name:?}");
        }
    }

    #[test]
    fn rejects_empty_and_edge_whitespace() {
        assert!(matches!(
            ResourceName::parse(""),
            Err(ValidationError::EmptyName)
        ));
        assert!(matches!(
            ResourceName::parse(" x"),
            Err(ValidationError::EdgeWhitespace)
        ));
        assert!(matches!(
            ResourceName::parse("x "),
            Err(ValidationError::EdgeWhitespace)
        ));
    }

    #[test]
    fn rejects_overlong_names() {
        let name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(matches!(
            ResourceName::parse(&name),
            Err(ValidationError::NameTooLong { length }) if length == MAX_NAME_LENGTH + 1
        ));
    }

    #[test]
    fn parses_valid_versions_and_rejects_junk() {
        assert!(parse_version("1.2.3").is_ok());
        assert!(parse_version("0.0.1-alpha.21").is_ok());
        assert!(parse_version("../../etc/passwd").is_err());
        assert!(parse_version("not-a-version").is_err());
        assert!(parse_version("1.2").is_err());
    }

    #[test]
    fn token_comparison_matches_only_exact_tokens() {
        assert!(token_matches("secret", "secret"));
        assert!(!token_matches("secret", "secreT"));
        assert!(!token_matches("secret", "secre"));
        assert!(!token_matches("secret", ""));
    }
}
