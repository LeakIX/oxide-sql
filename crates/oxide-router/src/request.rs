//! HTTP request type.

use std::collections::HashMap;

/// HTTP request methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    /// GET method
    Get,
    /// POST method
    Post,
    /// PUT method
    Put,
    /// PATCH method
    Patch,
    /// DELETE method
    Delete,
    /// HEAD method
    Head,
    /// OPTIONS method
    Options,
}

impl Method {
    /// Parses a method from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "PATCH" => Some(Self::Patch),
            "DELETE" => Some(Self::Delete),
            "HEAD" => Some(Self::Head),
            "OPTIONS" => Some(Self::Options),
            _ => None,
        }
    }

    /// Returns the method as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Path parameters extracted from the URL.
#[derive(Debug, Clone, Default)]
pub struct PathParams {
    params: HashMap<String, String>,
}

impl PathParams {
    /// Creates new empty path params.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a parameter.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.params.insert(key.into(), value.into());
    }

    /// Gets a parameter value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }

    /// Gets a parameter value or returns an error.
    pub fn require(&self, key: &str) -> Result<&str, String> {
        self.get(key)
            .ok_or_else(|| format!("Missing path parameter: {key}"))
    }

    /// Parses a parameter as a specific type.
    pub fn parse<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    /// Returns an iterator over the parameters.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.params.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// An HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method.
    pub method: Method,
    /// Request path.
    pub path: String,
    /// Path parameters extracted from URL patterns.
    pub params: PathParams,
    /// Query string parameters.
    pub query: HashMap<String, String>,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// Request body.
    pub body: Vec<u8>,
}

impl Request {
    /// Creates a new request.
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            params: PathParams::new(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Creates a GET request.
    pub fn get(path: impl Into<String>) -> Self {
        Self::new(Method::Get, path)
    }

    /// Creates a POST request.
    pub fn post(path: impl Into<String>) -> Self {
        Self::new(Method::Post, path)
    }

    /// Sets a header.
    #[must_use]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the body.
    #[must_use]
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Sets a query parameter.
    #[must_use]
    pub fn query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    /// Gets a header value.
    pub fn get_header(&self, key: &str) -> Option<&str> {
        // Case-insensitive header lookup
        let key_lower = key.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Gets a query parameter.
    pub fn get_query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(String::as_str)
    }

    /// Returns the body as a string.
    pub fn body_string(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }

    /// Parses the body as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    /// Parses query parameters from a query string.
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        query
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next().unwrap_or("");
                Some((urlencoding_decode(key), urlencoding_decode(value)))
            })
            .collect()
    }
}

/// Simple URL decoding.
fn urlencoding_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('%');
            result.push_str(&hex);
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_parsing() {
        assert_eq!(Method::from_str("GET"), Some(Method::Get));
        assert_eq!(Method::from_str("post"), Some(Method::Post));
        assert_eq!(Method::from_str("INVALID"), None);
    }

    #[test]
    fn test_path_params() {
        let mut params = PathParams::new();
        params.insert("id", "123");
        params.insert("name", "test");

        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.parse::<i64>("id"), Some(123));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_request_builder() {
        let req = Request::get("/users")
            .header("Content-Type", "application/json")
            .query_param("page", "1");

        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/users");
        assert_eq!(req.get_header("content-type"), Some("application/json"));
        assert_eq!(req.get_query("page"), Some("1"));
    }

    #[test]
    fn test_query_string_parsing() {
        let query = Request::parse_query_string("name=John+Doe&age=30&city=New%20York");
        assert_eq!(query.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(query.get("age"), Some(&"30".to_string()));
        assert_eq!(query.get("city"), Some(&"New York".to_string()));
    }
}
