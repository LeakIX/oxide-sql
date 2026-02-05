//! Main router implementation.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::error::{Result, RouterError};
use crate::middleware::{BoxFuture, Middleware, MiddlewareResult};
use crate::path::PathPattern;
use crate::request::{Method, Request};
use crate::response::Response;

/// A boxed async handler function.
pub type Handler = Arc<dyn Fn(Request) -> BoxFuture<'static, Response> + Send + Sync>;

/// A single route definition.
#[derive(Clone)]
pub struct Route {
    /// Optional route name for reverse URL lookup.
    pub name: Option<String>,
    /// HTTP method.
    pub method: Method,
    /// Path pattern.
    pub pattern: PathPattern,
    /// Request handler.
    pub handler: Handler,
}

impl Route {
    /// Creates a new route.
    pub fn new<F, Fut>(method: Method, pattern: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        Self {
            name: None,
            method,
            pattern: PathPattern::new(pattern),
            handler: Arc::new(move |req| Box::pin(handler(req))),
        }
    }

    /// Sets the route name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// A group of routes with a common prefix.
pub struct RouteGroup {
    /// URL prefix for all routes in this group.
    prefix: String,
    /// Routes in this group.
    routes: Vec<Route>,
    /// Middleware for this group.
    middleware: Vec<Arc<dyn Middleware>>,
}

impl RouteGroup {
    /// Creates a new route group with the given prefix.
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    /// Adds a GET route.
    #[must_use]
    pub fn get<F, Fut>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let full_path = format!("{}{}", self.prefix, path);
        self.routes
            .push(Route::new(Method::Get, &full_path, handler));
        self
    }

    /// Adds a POST route.
    #[must_use]
    pub fn post<F, Fut>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let full_path = format!("{}{}", self.prefix, path);
        self.routes
            .push(Route::new(Method::Post, &full_path, handler));
        self
    }

    /// Adds a PUT route.
    #[must_use]
    pub fn put<F, Fut>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let full_path = format!("{}{}", self.prefix, path);
        self.routes
            .push(Route::new(Method::Put, &full_path, handler));
        self
    }

    /// Adds a DELETE route.
    #[must_use]
    pub fn delete<F, Fut>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let full_path = format!("{}{}", self.prefix, path);
        self.routes
            .push(Route::new(Method::Delete, &full_path, handler));
        self
    }

    /// Adds middleware to this group.
    #[must_use]
    pub fn middleware(mut self, mw: impl Middleware + 'static) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    /// Returns the routes in this group.
    pub fn into_routes(self) -> Vec<Route> {
        self.routes
    }
}

/// The main router for handling HTTP requests.
pub struct Router {
    /// Registered routes.
    routes: Vec<Route>,
    /// Global middleware.
    middleware: Vec<Arc<dyn Middleware>>,
    /// Named routes for reverse URL lookup.
    named_routes: HashMap<String, PathPattern>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    /// Creates a new empty router.
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
            named_routes: HashMap::new(),
        }
    }

    /// Adds a GET route.
    #[must_use]
    pub fn get<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Get, path, handler)
    }

    /// Adds a POST route.
    #[must_use]
    pub fn post<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Post, path, handler)
    }

    /// Adds a PUT route.
    #[must_use]
    pub fn put<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Put, path, handler)
    }

    /// Adds a PATCH route.
    #[must_use]
    pub fn patch<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Patch, path, handler)
    }

    /// Adds a DELETE route.
    #[must_use]
    pub fn delete<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Delete, path, handler)
    }

    /// Adds a route with any method.
    #[must_use]
    pub fn route<F, Fut>(mut self, method: Method, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.routes.push(Route::new(method, path, handler));
        self
    }

    /// Adds a named route.
    #[must_use]
    pub fn named_route<F, Fut>(mut self, name: &str, method: Method, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let route = Route::new(method, path, handler).name(name);
        self.named_routes
            .insert(name.to_string(), route.pattern.clone());
        self.routes.push(route);
        self
    }

    /// Adds global middleware.
    #[must_use]
    pub fn middleware(mut self, mw: impl Middleware + 'static) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    /// Adds a route group.
    #[must_use]
    pub fn group(mut self, group: RouteGroup) -> Self {
        self.routes.extend(group.into_routes());
        self
    }

    /// Generates a URL for a named route.
    pub fn url_for(&self, name: &str, params: &HashMap<String, String>) -> Option<String> {
        self.named_routes.get(name).and_then(|p| p.reverse(params))
    }

    /// Handles an incoming request.
    pub fn handle<'a>(
        &'a self,
        mut request: Request,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + 'a>> {
        Box::pin(async move {
            // Run before middleware
            for mw in &self.middleware {
                match mw.before(&request).await {
                    MiddlewareResult::Continue(req) => request = req,
                    MiddlewareResult::Response(res) => {
                        // Run after middleware even on early return
                        let mut response = res;
                        for mw in self.middleware.iter().rev() {
                            response = mw.after(response).await;
                        }
                        return response;
                    }
                }
            }

            // Find matching route
            let mut response = match self.find_route(&request) {
                Ok((route, params)) => {
                    let mut req = request.clone();
                    req.params = params;
                    (route.handler)(req).await
                }
                Err(RouterError::NotFound { .. }) => Response::not_found(),
                Err(RouterError::MethodNotAllowed { .. }) => Response::method_not_allowed(),
                Err(_) => Response::internal_server_error(),
            };

            // Run after middleware
            for mw in self.middleware.iter().rev() {
                response = mw.after(response).await;
            }

            response
        })
    }

    /// Finds a matching route for the request.
    fn find_route(&self, request: &Request) -> Result<(&Route, crate::request::PathParams)> {
        let mut method_matched = false;

        for route in &self.routes {
            if let Some(params) = route.pattern.match_path(&request.path) {
                method_matched = true;
                if route.method == request.method {
                    return Ok((route, params));
                }
            }
        }

        if method_matched {
            Err(RouterError::MethodNotAllowed {
                method: request.method.to_string(),
                path: request.path.clone(),
            })
        } else {
            Err(RouterError::NotFound {
                method: request.method.to_string(),
                path: request.path.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn hello_handler(_req: Request) -> Response {
        Response::text("Hello, World!")
    }

    async fn user_handler(req: Request) -> Response {
        let id = req.params.get("id").unwrap_or("unknown");
        Response::text(format!("User: {id}"))
    }

    #[tokio::test]
    async fn test_basic_routing() {
        let router = Router::new()
            .get("/", hello_handler)
            .get("/users/{id}", user_handler);

        let req = Request::get("/");
        let res = router.handle(req).await;
        assert_eq!(res.status, 200);
        assert_eq!(res.body_string(), Some("Hello, World!".to_string()));
    }

    #[tokio::test]
    async fn test_path_params() {
        let router = Router::new().get("/users/{id}", user_handler);

        let req = Request::get("/users/123");
        let res = router.handle(req).await;
        assert_eq!(res.status, 200);
        assert_eq!(res.body_string(), Some("User: 123".to_string()));
    }

    #[tokio::test]
    async fn test_not_found() {
        let router = Router::new().get("/", hello_handler);

        let req = Request::get("/nonexistent");
        let res = router.handle(req).await;
        assert_eq!(res.status, 404);
    }

    #[tokio::test]
    async fn test_method_not_allowed() {
        let router = Router::new().get("/", hello_handler);

        let req = Request::post("/");
        let res = router.handle(req).await;
        assert_eq!(res.status, 405);
    }

    #[tokio::test]
    async fn test_named_route() {
        let router =
            Router::new().named_route("user_detail", Method::Get, "/users/{id}", user_handler);

        let params: HashMap<String, String> =
            [("id".to_string(), "42".to_string())].into_iter().collect();
        let url = router.url_for("user_detail", &params);
        assert_eq!(url, Some("/users/42".to_string()));
    }

    #[tokio::test]
    async fn test_route_group() {
        let api_group = RouteGroup::new("/api/v1")
            .get("/users", hello_handler)
            .get("/users/{id}", user_handler);

        let router = Router::new().group(api_group);

        let req = Request::get("/api/v1/users/123");
        let res = router.handle(req).await;
        assert_eq!(res.status, 200);
    }
}
