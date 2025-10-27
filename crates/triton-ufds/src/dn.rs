//! Distinguished Name utilities for working with UFDS entries.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

use triton_core::error::Error as CoreError;

/// Errors that can occur when parsing or manipulating distinguished names.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DistinguishedNameError {
    /// The distinguished name was empty.
    #[error("distinguished name cannot be empty")]
    Empty,
    /// A component in the distinguished name was invalid.
    #[error("invalid distinguished name component: {0}")]
    InvalidComponent(String),
    /// A component was missing the attribute name to the left of the `=`.
    #[error("distinguished name component missing attribute: {0}")]
    MissingAttribute(String),
    /// A component was missing the value to the right of the `=`.
    #[error("distinguished name component missing value for attribute {0}")]
    MissingValue(String),
    /// The distinguished name ended with an escape character.
    #[error("distinguished name contains an unterminated escape sequence")]
    UnterminatedEscape,
}

impl From<DistinguishedNameError> for CoreError {
    fn from(err: DistinguishedNameError) -> Self {
        CoreError::InvalidRequest(err.to_string())
    }
}

/// Relative distinguished name (single attribute/value pair).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelativeDistinguishedName {
    attribute: String,
    value: String,
}

impl RelativeDistinguishedName {
    /// Create a new relative distinguished name.
    #[must_use]
    pub fn new(attribute: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            attribute: attribute.into(),
            value: value.into(),
        }
    }

    /// Attribute portion of the RDN (e.g. `cn`).
    #[must_use]
    pub fn attribute(&self) -> &str {
        &self.attribute
    }

    /// Attribute value portion of the RDN.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns true if this RDN matches the provided attribute name (case-insensitive).
    #[must_use]
    pub fn matches_attribute(&self, attribute: &str) -> bool {
        self.attribute.eq_ignore_ascii_case(attribute)
    }
}

/// Strongly-typed distinguished name wrapper.
///
/// The structure keeps a canonical string representation while providing convenient access to the
/// individual relative distinguished names. Parsing is intentionally strict to surface malformed
/// DNs early.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistinguishedName {
    raw: String,
    rdns: Vec<Vec<RelativeDistinguishedName>>,
}

impl DistinguishedName {
    /// Parses a distinguished name from a string.
    ///
    /// # Errors
    ///
    /// Returns [`DistinguishedNameError`] if the distinguished name is empty or contains invalid
    /// syntax.
    pub fn parse(input: impl AsRef<str>) -> std::result::Result<Self, DistinguishedNameError> {
        let raw = input.as_ref().trim();
        if raw.is_empty() {
            return Err(DistinguishedNameError::Empty);
        }

        let mut rdns = Vec::new();
        for component in split_escaped(raw, ',')? {
            if component.is_empty() {
                return Err(DistinguishedNameError::InvalidComponent(raw.to_string()));
            }

            let mut rdn_components = Vec::new();
            for part in split_escaped(&component, '+')? {
                if part.is_empty() {
                    return Err(DistinguishedNameError::InvalidComponent(component.clone()));
                }

                let (attribute, value) = split_attribute_value(&part)?;
                rdn_components.push(RelativeDistinguishedName::new(attribute, value));
            }

            if rdn_components.is_empty() {
                return Err(DistinguishedNameError::InvalidComponent(component));
            }

            rdns.push(rdn_components);
        }

        Ok(Self {
            raw: rdns_to_string(&rdns),
            rdns,
        })
    }

    /// Borrows the canonical distinguished name string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Returns the RDN sets in order (each set represents a `+`-joined group).
    #[must_use]
    pub fn rdns(&self) -> &[Vec<RelativeDistinguishedName>] {
        &self.rdns
    }

    /// Returns an iterator over all relative distinguished names in order.
    #[must_use]
    pub fn components(&self) -> impl Iterator<Item = &RelativeDistinguishedName> + '_ {
        self.rdns.iter().flat_map(|rdn| rdn.iter())
    }

    /// Looks up the value for the first attribute that matches `attribute` (case-insensitive).
    #[must_use]
    pub fn get(&self, attribute: &str) -> Option<&str> {
        self.components()
            .find(|rdn| rdn.matches_attribute(attribute))
            .map(RelativeDistinguishedName::value)
    }

    /// Returns true if the distinguished name contains a matching attribute/value pair.
    #[must_use]
    pub fn contains(&self, attribute: &str, value: &str) -> bool {
        self.components()
            .any(|rdn| rdn.matches_attribute(attribute) && rdn.value.eq_ignore_ascii_case(value))
    }

    /// Creates a new distinguished name by prefixing the provided RDN.
    #[must_use]
    pub fn with_prefix(mut self, rdn: RelativeDistinguishedName) -> Self {
        self.rdns.insert(0, vec![rdn]);
        self.raw = rdns_to_string(&self.rdns);
        self
    }

    /// Creates a new distinguished name by appending another distinguished name.
    ///
    /// This is useful when combining an entry-specific RDN with a base DN.
    #[must_use]
    pub fn join(mut self, suffix: &DistinguishedName) -> Self {
        self.rdns.extend(suffix.rdns.iter().cloned());
        self.raw = rdns_to_string(&self.rdns);
        self
    }
}

impl fmt::Display for DistinguishedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl FromStr for DistinguishedName {
    type Err = DistinguishedNameError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<DistinguishedName> for String {
    fn from(value: DistinguishedName) -> Self {
        value.raw
    }
}

impl TryFrom<&str> for DistinguishedName {
    type Error = DistinguishedNameError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Self::parse(value)
    }
}

fn split_escaped(
    input: &str,
    delimiter: char,
) -> std::result::Result<Vec<String>, DistinguishedNameError> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' {
            escape = true;
            continue;
        }

        if ch == delimiter {
            parts.push(current.trim().to_string());
            current.clear();
            continue;
        }

        current.push(ch);
    }

    if escape {
        return Err(DistinguishedNameError::UnterminatedEscape);
    }

    parts.push(current.trim().to_string());
    if parts.iter().any(|part| part.is_empty()) {
        return Err(DistinguishedNameError::InvalidComponent(input.to_string()));
    }
    Ok(parts)
}

fn split_attribute_value(
    component: &str,
) -> std::result::Result<(String, String), DistinguishedNameError> {
    let mut escape = false;
    let mut index = None;

    for (i, ch) in component.char_indices() {
        if escape {
            escape = false;
            continue;
        }

        if ch == '\\' {
            escape = true;
            continue;
        }

        if ch == '=' {
            index = Some(i);
            break;
        }
    }

    let idx =
        index.ok_or_else(|| DistinguishedNameError::InvalidComponent(component.to_string()))?;
    let attribute = component[..idx].trim();
    let value_part = component[idx + 1..].trim_start();

    if attribute.is_empty() {
        return Err(DistinguishedNameError::MissingAttribute(
            component.to_string(),
        ));
    }

    if value_part.is_empty() {
        return Err(DistinguishedNameError::MissingValue(attribute.to_string()));
    }

    Ok((attribute.to_string(), unescape(value_part)?))
}

fn unescape(value: &str) -> std::result::Result<String, DistinguishedNameError> {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let next = chars
                .next()
                .ok_or(DistinguishedNameError::UnterminatedEscape)?;
            result.push(next);
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

fn escape(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = value.chars().collect();
    let mut escaped = String::with_capacity(value.len());

    for (idx, ch) in chars.iter().enumerate() {
        let is_first = idx == 0;
        let is_last = idx == chars.len() - 1;
        let needs_escape = matches!(ch, ',' | '+' | '"' | '\\' | '<' | '>' | ';' | '=')
            || (is_first && (*ch == ' ' || *ch == '#'))
            || (is_last && *ch == ' ');

        if needs_escape {
            escaped.push('\\');
        }
        escaped.push(*ch);
    }

    escaped
}

fn rdns_to_string(rdns: &[Vec<RelativeDistinguishedName>]) -> String {
    rdns.iter()
        .map(|rdn| {
            rdn.iter()
                .map(|component| format!("{}={}", component.attribute(), escape(component.value())))
                .collect::<Vec<_>>()
                .join("+")
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_dn() {
        let dn = DistinguishedName::parse("cn=John Doe,ou=People,dc=example,dc=com").unwrap();
        assert_eq!(dn.get("cn"), Some("John Doe"));
        assert_eq!(dn.get("ou"), Some("People"));
        assert!(dn.contains("dc", "example"));
        assert_eq!(dn.to_string(), "cn=John Doe,ou=People,dc=example,dc=com");
    }

    #[test]
    fn parse_dn_with_escape() {
        let dn = DistinguishedName::parse("cn=Smith\\, John,ou=People,dc=example,dc=com").unwrap();
        assert_eq!(dn.get("cn"), Some("Smith, John"));
        assert!(dn.to_string().starts_with("cn=Smith\\, John,ou=People"));
    }

    #[test]
    fn parse_multi_valued_rdn() {
        let dn = DistinguishedName::parse("cn=John+uid=1234,ou=People,dc=example,dc=com").unwrap();
        assert!(dn.contains("cn", "John"));
        assert!(dn.contains("uid", "1234"));
        assert_eq!(
            dn.to_string(),
            "cn=John+uid=1234,ou=People,dc=example,dc=com"
        );
    }

    #[test]
    fn invalid_trailing_delimiter() {
        let err = DistinguishedName::parse("cn=John,").unwrap_err();
        assert!(matches!(err, DistinguishedNameError::InvalidComponent(_)));
    }

    #[test]
    fn with_prefix_and_join() {
        let base = DistinguishedName::parse("ou=People,dc=example,dc=com").unwrap();
        let user_rdn = RelativeDistinguishedName::new("cn", "Jane Doe");
        let user_dn = base.clone().with_prefix(user_rdn);
        assert_eq!(
            user_dn.to_string(),
            "cn=Jane Doe,ou=People,dc=example,dc=com"
        );

        let full = DistinguishedName::parse("uid=1234").unwrap().join(&base);
        assert_eq!(full.to_string(), "uid=1234,ou=People,dc=example,dc=com");
    }
}
