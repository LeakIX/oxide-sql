//! HTTP response type.

use std::collections::HashMap;

/// An HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response body.
    pub body: Vec<u8>,
}

impl Response {
    /// Creates a new response with the given status.
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Creates a 200 OK response.
    pub fn ok() -> Self {
        Self::new(200)
    }

    /// Creates a response with HTML content.
    pub fn html(body: impl Into<String>) -> Self {
        let body_str = body.into();
        Self {
            status: 200,
            headers: [(
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string(),
            )]
            .into_iter()
            .collect(),
            body: body_str.into_bytes(),
        }
    }

    /// Creates a response with JSON content.
    pub fn json<T: serde::Serialize>(data: &T) -> Self {
        match serde_json::to_vec(data) {
            Ok(body) => Self {
                status: 200,
                headers: [("Content-Type".to_string(), "application/json".to_string())]
                    .into_iter()
                    .collect(),
                body,
            },
            Err(_) => Self::internal_server_error(),
        }
    }

    /// Creates a response with plain text content.
    pub fn text(body: impl Into<String>) -> Self {
        let body_str = body.into();
        Self {
            status: 200,
            headers: [(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]
            .into_iter()
            .collect(),
            body: body_str.into_bytes(),
        }
    }

    /// Creates a redirect response.
    pub fn redirect(url: impl Into<String>) -> Self {
        Self {
            status: 302,
            headers: [("Location".to_string(), url.into())].into_iter().collect(),
            body: Vec::new(),
        }
    }

    /// Creates a permanent redirect response.
    pub fn redirect_permanent(url: impl Into<String>) -> Self {
        Self {
            status: 301,
            headers: [("Location".to_string(), url.into())].into_iter().collect(),
            body: Vec::new(),
        }
    }

    /// Creates a 400 Bad Request response.
    pub fn bad_request() -> Self {
        Self {
            status: 400,
            headers: HashMap::new(),
            body: b"Bad Request".to_vec(),
        }
    }

    /// Creates a 401 Unauthorized response.
    pub fn unauthorized() -> Self {
        Self {
            status: 401,
            headers: HashMap::new(),
            body: b"Unauthorized".to_vec(),
        }
    }

    /// Creates a 403 Forbidden response.
    pub fn forbidden() -> Self {
        Self {
            status: 403,
            headers: HashMap::new(),
            body: b"Forbidden".to_vec(),
        }
    }

    /// Creates a 404 Not Found response.
    pub fn not_found() -> Self {
        Self {
            status: 404,
            headers: HashMap::new(),
            body: b"Not Found".to_vec(),
        }
    }

    /// Creates a 405 Method Not Allowed response.
    pub fn method_not_allowed() -> Self {
        Self {
            status: 405,
            headers: HashMap::new(),
            body: b"Method Not Allowed".to_vec(),
        }
    }

    /// Creates a 500 Internal Server Error response.
    pub fn internal_server_error() -> Self {
        Self {
            status: 500,
            headers: HashMap::new(),
            body: b"Internal Server Error".to_vec(),
        }
    }

    /// Sets a header.
    #[must_use]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the status code.
    #[must_use]
    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Sets the body.
    #[must_use]
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Returns the body as a string.
    pub fn body_string(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }

    /// Returns the status text for the current status code.
    pub fn status_text(&self) -> &'static str {
        match self.status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            422 => "Unprocessable Entity",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            _ => "Unknown",
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_html() {
        let res = Response::html("<h1>Hello</h1>");
        assert_eq!(res.status, 200);
        assert_eq!(
            res.headers.get("Content-Type"),
            Some(&"text/html; charset=utf-8".to_string())
        );
        assert_eq!(res.body_string(), Some("<h1>Hello</h1>".to_string()));
    }

    #[test]
    fn test_response_json() {
        let data = serde_json::json!({"name": "test"});
        let res = Response::json(&data);
        assert_eq!(res.status, 200);
        assert_eq!(
            res.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_response_redirect() {
        let res = Response::redirect("/login");
        assert_eq!(res.status, 302);
        assert_eq!(res.headers.get("Location"), Some(&"/login".to_string()));
    }

    #[test]
    fn test_response_builder() {
        let res = Response::ok().header("X-Custom", "value").body("Hello");

        assert_eq!(res.status, 200);
        assert_eq!(res.headers.get("X-Custom"), Some(&"value".to_string()));
        assert_eq!(res.body_string(), Some("Hello".to_string()));
    }
}
