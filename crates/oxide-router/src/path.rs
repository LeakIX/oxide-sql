//! Path pattern matching.

use regex::Regex;
use std::collections::HashMap;

use crate::request::PathParams;

/// A segment in a path pattern.
#[derive(Debug, Clone)]
pub enum PathSegment {
    /// A literal string segment.
    Literal(String),
    /// A parameter segment (e.g., {id}).
    Param(String),
    /// A wildcard segment (matches remainder of path).
    Wildcard(String),
}

/// A compiled path pattern for matching URLs.
#[derive(Debug, Clone)]
pub struct PathPattern {
    /// The original pattern string.
    pattern: String,
    /// Parsed segments.
    segments: Vec<PathSegment>,
    /// Compiled regex for matching.
    regex: Regex,
    /// Parameter names in order.
    param_names: Vec<String>,
}

impl PathPattern {
    /// Parses a path pattern string.
    ///
    /// Pattern syntax:
    /// - `/users` - Literal path
    /// - `/users/{id}` - Path with parameter
    /// - `/files/{*path}` - Wildcard parameter (matches rest of path)
    ///
    /// # Example
    ///
    /// ```
    /// use oxide_router::PathPattern;
    ///
    /// let pattern = PathPattern::new("/posts/{id}/comments/{comment_id}");
    /// let params = pattern.match_path("/posts/123/comments/456").unwrap();
    /// assert_eq!(params.get("id"), Some("123"));
    /// assert_eq!(params.get("comment_id"), Some("456"));
    /// ```
    pub fn new(pattern: &str) -> Self {
        let mut segments = Vec::new();
        let mut param_names = Vec::new();
        let mut regex_str = String::from("^");

        for part in pattern.split('/').filter(|s| !s.is_empty()) {
            regex_str.push('/');

            if let Some(param) = part.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                if let Some(name) = param.strip_prefix('*') {
                    // Wildcard parameter
                    segments.push(PathSegment::Wildcard(name.to_string()));
                    param_names.push(name.to_string());
                    regex_str.push_str("(.+)");
                } else {
                    // Regular parameter
                    segments.push(PathSegment::Param(param.to_string()));
                    param_names.push(param.to_string());
                    regex_str.push_str("([^/]+)");
                }
            } else {
                // Literal segment
                segments.push(PathSegment::Literal(part.to_string()));
                regex_str.push_str(&regex::escape(part));
            }
        }

        regex_str.push_str("/?$");

        let regex = Regex::new(&regex_str).expect("Invalid path pattern regex");

        Self {
            pattern: pattern.to_string(),
            segments,
            regex,
            param_names,
        }
    }

    /// Attempts to match a path against this pattern.
    ///
    /// Returns extracted parameters if the path matches.
    pub fn match_path(&self, path: &str) -> Option<PathParams> {
        let caps = self.regex.captures(path)?;

        let mut params = PathParams::new();

        for (i, name) in self.param_names.iter().enumerate() {
            if let Some(value) = caps.get(i + 1) {
                params.insert(name.clone(), value.as_str().to_string());
            }
        }

        Some(params)
    }

    /// Returns the original pattern string.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Returns the parameter names.
    pub fn param_names(&self) -> &[String] {
        &self.param_names
    }

    /// Generates a path from parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use oxide_router::PathPattern;
    ///
    /// let pattern = PathPattern::new("/posts/{id}");
    /// let params: HashMap<String, String> =
    ///     [("id".to_string(), "123".to_string())]
    ///     .into_iter()
    ///     .collect();
    /// let path = pattern.reverse(&params).unwrap();
    /// assert_eq!(path, "/posts/123");
    /// ```
    pub fn reverse(&self, params: &HashMap<String, String>) -> Option<String> {
        let mut path = String::new();

        for segment in &self.segments {
            path.push('/');
            match segment {
                PathSegment::Literal(s) => path.push_str(s),
                PathSegment::Param(name) | PathSegment::Wildcard(name) => {
                    path.push_str(params.get(name)?);
                }
            }
        }

        if path.is_empty() {
            path.push('/');
        }

        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_path() {
        let pattern = PathPattern::new("/users");
        assert!(pattern.match_path("/users").is_some());
        assert!(pattern.match_path("/users/").is_some());
        assert!(pattern.match_path("/posts").is_none());
    }

    #[test]
    fn test_single_param() {
        let pattern = PathPattern::new("/users/{id}");
        let params = pattern.match_path("/users/123").unwrap();
        assert_eq!(params.get("id"), Some("123"));
    }

    #[test]
    fn test_multiple_params() {
        let pattern = PathPattern::new("/posts/{post_id}/comments/{comment_id}");
        let params = pattern.match_path("/posts/42/comments/7").unwrap();
        assert_eq!(params.get("post_id"), Some("42"));
        assert_eq!(params.get("comment_id"), Some("7"));
    }

    #[test]
    fn test_wildcard_param() {
        let pattern = PathPattern::new("/files/{*path}");
        let params = pattern.match_path("/files/docs/readme.md").unwrap();
        assert_eq!(params.get("path"), Some("docs/readme.md"));
    }

    #[test]
    fn test_reverse() {
        let pattern = PathPattern::new("/posts/{id}");
        let params: HashMap<String, String> = [("id".to_string(), "123".to_string())]
            .into_iter()
            .collect();
        assert_eq!(pattern.reverse(&params), Some("/posts/123".to_string()));
    }

    #[test]
    fn test_reverse_missing_param() {
        let pattern = PathPattern::new("/posts/{id}");
        let params: HashMap<String, String> = HashMap::new();
        assert!(pattern.reverse(&params).is_none());
    }
}
