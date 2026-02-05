//! # oxide-router
//!
//! A lightweight URL routing library with middleware support.
//!
//! This crate provides:
//! - Path pattern matching with parameters
//! - HTTP method-based routing
//! - Middleware support (before/after hooks)
//! - Route groups with prefixes
//! - Named routes for reverse URL lookup
//!
//! ## Quick Start
//!
//! ```ignore
//! use oxide_router::{Router, Request, Response};
//!
//! async fn hello_handler(_req: Request) -> Response {
//!     Response::text("Hello, World!")
//! }
//!
//! async fn user_handler(req: Request) -> Response {
//!     let id = req.params.get("id").unwrap_or("unknown");
//!     Response::json(&serde_json::json!({"id": id}))
//! }
//!
//! let router = Router::new()
//!     .get("/", hello_handler)
//!     .get("/users/{id}", user_handler)
//!     .post("/users", create_user_handler);
//!
//! // Handle a request
//! let request = Request::get("/users/123");
//! let response = router.handle(request).await;
//! ```
//!
//! ## Path Parameters
//!
//! Routes can include path parameters using `{name}` syntax:
//!
//! ```ignore
//! router.get("/posts/{post_id}/comments/{comment_id}", handler)
//! ```
//!
//! Parameters are available in `request.params`:
//!
//! ```ignore
//! async fn handler(req: Request) -> Response {
//!     let post_id = req.params.get("post_id").unwrap();
//!     let comment_id = req.params.get("comment_id").unwrap();
//!     // ...
//! }
//! ```
//!
//! ## Middleware
//!
//! ```ignore
//! use oxide_router::{Router, Middleware, LoggingMiddleware, AuthMiddleware};
//!
//! let router = Router::new()
//!     .middleware(LoggingMiddleware)
//!     .middleware(AuthMiddleware::new("/login").exclude(&["/public"]))
//!     .get("/", handler);
//! ```
//!
//! ## Route Groups
//!
//! ```ignore
//! use oxide_router::{Router, RouteGroup};
//!
//! let api = RouteGroup::new("/api/v1")
//!     .get("/users", list_users)
//!     .get("/users/{id}", get_user)
//!     .post("/users", create_user);
//!
//! let router = Router::new()
//!     .group(api);
//! ```
//!
//! ## Named Routes
//!
//! ```ignore
//! let router = Router::new()
//!     .named_route("user_detail", Method::Get, "/users/{id}", handler);
//!
//! // Generate URL
//! let url = router.url_for("user_detail", &[("id", "123")].into());
//! assert_eq!(url, Some("/users/123".to_string()));
//! ```

mod error;
mod middleware;
mod path;
mod request;
mod response;
mod router;

pub use error::{Result, RouterError};
pub use middleware::{
    AuthMiddleware, BoxFuture, CorsMiddleware, LoggingMiddleware, Middleware, MiddlewareResult,
};
pub use path::PathPattern;
pub use request::{Method, PathParams, Request};
pub use response::Response;
pub use router::{Handler, Route, RouteGroup, Router};
