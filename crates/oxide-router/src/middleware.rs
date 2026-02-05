//! Middleware support for request/response processing.

use std::future::Future;
use std::pin::Pin;

use crate::request::Request;
use crate::response::Response;

/// A boxed future for async middleware operations.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Result of middleware processing.
pub enum MiddlewareResult {
    /// Continue to the next middleware/handler.
    Continue(Request),
    /// Stop processing and return this response.
    Response(Response),
}

/// Trait for middleware that processes requests and responses.
///
/// Middleware can:
/// - Modify the request before it reaches the handler
/// - Short-circuit processing and return a response
/// - Modify the response after the handler runs
///
/// # Example
///
/// ```ignore
/// struct LoggingMiddleware;
///
/// impl Middleware for LoggingMiddleware {
///     fn before<'a>(&'a self, req: &'a Request) -> BoxFuture<'a, MiddlewareResult> {
///         Box::pin(async move {
///             println!("{} {}", req.method, req.path);
///             MiddlewareResult::Continue(req.clone())
///         })
///     }
///
///     fn after<'a>(&'a self, res: Response) -> BoxFuture<'a, Response> {
///         Box::pin(async move {
///             println!("Response: {}", res.status);
///             res
///         })
///     }
/// }
/// ```
pub trait Middleware: Send + Sync {
    /// Called before the request handler.
    ///
    /// Can modify the request or short-circuit with a response.
    fn before<'a>(&'a self, req: &'a Request) -> BoxFuture<'a, MiddlewareResult>;

    /// Called after the request handler.
    ///
    /// Can modify the response.
    fn after<'a>(&'a self, res: Response) -> BoxFuture<'a, Response>;
}

/// Middleware that requires authentication.
pub struct AuthMiddleware {
    /// Paths to exclude from authentication.
    pub exclude: Vec<String>,
    /// The login redirect URL.
    pub login_url: String,
}

impl AuthMiddleware {
    /// Creates new auth middleware.
    pub fn new(login_url: impl Into<String>) -> Self {
        Self {
            exclude: Vec::new(),
            login_url: login_url.into(),
        }
    }

    /// Adds paths to exclude from authentication.
    #[must_use]
    pub fn exclude(mut self, paths: &[&str]) -> Self {
        self.exclude = paths.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Checks if a path should be excluded.
    fn is_excluded(&self, path: &str) -> bool {
        self.exclude.iter().any(|p| path.starts_with(p))
    }
}

impl Middleware for AuthMiddleware {
    fn before<'a>(&'a self, req: &'a Request) -> BoxFuture<'a, MiddlewareResult> {
        Box::pin(async move {
            if self.is_excluded(&req.path) {
                return MiddlewareResult::Continue(req.clone());
            }

            // Check for session cookie or auth header
            // This is a simplified check - real implementation would verify the session
            if req.get_header("Authorization").is_some()
                || req
                    .get_header("Cookie")
                    .is_some_and(|c| c.contains("session="))
            {
                MiddlewareResult::Continue(req.clone())
            } else {
                MiddlewareResult::Response(Response::redirect(&self.login_url))
            }
        })
    }

    fn after<'a>(&'a self, res: Response) -> BoxFuture<'a, Response> {
        Box::pin(async move { res })
    }
}

/// Middleware that logs requests.
pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn before<'a>(&'a self, req: &'a Request) -> BoxFuture<'a, MiddlewareResult> {
        Box::pin(async move {
            // In a real implementation, this would use a proper logger
            eprintln!("--> {} {}", req.method, req.path);
            MiddlewareResult::Continue(req.clone())
        })
    }

    fn after<'a>(&'a self, res: Response) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            eprintln!("<-- {}", res.status);
            res
        })
    }
}

/// Middleware that adds CORS headers.
pub struct CorsMiddleware {
    /// Allowed origins.
    pub allowed_origins: Vec<String>,
    /// Allowed methods.
    pub allowed_methods: Vec<String>,
    /// Allowed headers.
    pub allowed_headers: Vec<String>,
}

impl CorsMiddleware {
    /// Creates CORS middleware that allows all origins.
    pub fn permissive() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
        }
    }

    /// Creates CORS middleware with specific origins.
    pub fn new(origins: &[&str]) -> Self {
        Self {
            allowed_origins: origins.iter().map(|s| (*s).to_string()).collect(),
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
            ],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
        }
    }
}

impl Middleware for CorsMiddleware {
    fn before<'a>(&'a self, req: &'a Request) -> BoxFuture<'a, MiddlewareResult> {
        Box::pin(async move {
            // Handle preflight requests
            if req.method == crate::request::Method::Options {
                let res = Response::ok()
                    .header(
                        "Access-Control-Allow-Origin",
                        self.allowed_origins.join(", "),
                    )
                    .header(
                        "Access-Control-Allow-Methods",
                        self.allowed_methods.join(", "),
                    )
                    .header(
                        "Access-Control-Allow-Headers",
                        self.allowed_headers.join(", "),
                    )
                    .header("Access-Control-Max-Age", "86400");
                return MiddlewareResult::Response(res);
            }
            MiddlewareResult::Continue(req.clone())
        })
    }

    fn after<'a>(&'a self, res: Response) -> BoxFuture<'a, Response> {
        let origins = self.allowed_origins.join(", ");
        Box::pin(async move { res.header("Access-Control-Allow-Origin", origins) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_middleware_exclude() {
        let mw = AuthMiddleware::new("/login").exclude(&["/public", "/api/health"]);
        assert!(mw.is_excluded("/public/file.txt"));
        assert!(mw.is_excluded("/api/health"));
        assert!(!mw.is_excluded("/admin"));
    }
}
