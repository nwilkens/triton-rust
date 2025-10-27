//! Convenience builder for HTTP query parameters.
//!
//! This module provides a lightweight helper for constructing URL query pairs
//! from optional values, reducing boilerplate in client crates.

use std::fmt::Display;

/// Builder for assembling query parameter pairs.
#[derive(Debug, Default, Clone)]
pub struct QueryParams {
    pairs: Vec<(&'static str, String)>,
}

impl QueryParams {
    /// Create a new, empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    /// Append a key/value pair when the value is present.
    pub fn push_opt<T>(&mut self, key: &'static str, value: Option<T>)
    where
        T: ToString,
    {
        if let Some(value) = value {
            self.pairs.push((key, value.to_string()));
        }
    }

    /// Append using a mapping function when the value is present.
    pub fn push_opt_with<T, F>(&mut self, key: &'static str, value: Option<T>, mut map: F)
    where
        F: FnMut(T) -> String,
    {
        if let Some(value) = value {
            self.pairs.push((key, map(value)));
        }
    }

    /// Append a required key/value pair.
    pub fn push<T>(&mut self, key: &'static str, value: T)
    where
        T: Display,
    {
        self.pairs.push((key, value.to_string()));
    }

    /// Return the collected key/value pairs.
    #[must_use]
    pub fn into_pairs(self) -> Vec<(&'static str, String)> {
        self.pairs
    }

    /// Returns true if no parameters have been added.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::QueryParams;

    #[test]
    fn push_opt_skips_none() {
        let mut params = QueryParams::new();
        params.push_opt("name", Option::<String>::None);
        assert!(params.is_empty());
    }

    #[test]
    fn push_opt_with_applies_mapper() {
        let mut params = QueryParams::new();
        params.push_opt_with("limit", Some(5u32), |v| format!("{v:02}"));
        assert_eq!(params.into_pairs(), vec![("limit", "05".to_string())]);
    }
}
