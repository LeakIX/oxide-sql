//! Blog Admin Example
//!
//! This example demonstrates the oxide-admin interface with a blog application.
//! Run with: cargo run --example blog_admin
//! Then visit: http://localhost:3000/admin/
//!
//! ## Type-Safe Queries
//!
//! This example uses `#[derive(Table)]` to generate type-safe query builders.
//! Trying to use a non-existent column will cause a compile-time error:
//!
//! ```compile_fail
//! // This won't compile - `invalid_column` doesn't exist on Post
//! let (sql, _) = Select::<PostTable, _, _>::new()
//!     .select::<(PostColumns::InvalidColumn,)>()  // Error!
//!     .from_table()
//!     .build();
//! ```

use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request as HyperRequest, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use oxide_admin::{AdminSite, Fieldset, ModelAdmin};
use oxide_router::{Method, Request, Response, Router};
use oxide_sql_core::builder::{col, Delete, Insert, Select, Update};
use oxide_sql_derive::Table;

use ironhtml::html;
use ironhtml::typed::{Document, Element};
use ironhtml_elements::{
    Body, Div, Form, Head, Html, Li, Main, Meta, Nav, Option_, Script, Select as SelectEl, Td, Th,
    Title, Tr, Ul,
};

// ============================================================================
// Models - Using #[derive(Table)] for type-safe queries
// ============================================================================

/// Blog post model.
///
/// The `#[derive(Table)]` macro generates:
/// - `PostTable` - implements the `Table` trait
/// - `PostColumns` - module with typed column accessors (Id, Title, Slug, etc.)
/// - Column accessor methods: `Post::id()`, `Post::title()`, etc.
#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "posts")]
pub struct Post {
    #[column(primary_key)]
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
    pub author_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl oxide_orm::Model for Post {
    type Table = PostTable;
    type PrimaryKey = i64;

    fn pk_column() -> &'static str {
        "id"
    }

    fn pk(&self) -> Self::PrimaryKey {
        self.id
    }

    fn is_saved(&self) -> bool {
        self.id > 0
    }
}

/// Comment model with type-safe columns.
#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "comments")]
pub struct Comment {
    #[column(primary_key)]
    pub id: i64,
    pub post_id: i64,
    pub author: String,
    pub content: String,
    pub created_at: String,
}

impl oxide_orm::Model for Comment {
    type Table = CommentTable;
    type PrimaryKey = i64;

    fn pk_column() -> &'static str {
        "id"
    }

    fn pk(&self) -> Self::PrimaryKey {
        self.id
    }

    fn is_saved(&self) -> bool {
        self.id > 0
    }
}

/// Tag model with type-safe columns.
#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "tags")]
pub struct Tag {
    #[column(primary_key)]
    pub id: i64,
    pub name: String,
    pub slug: String,
}

impl oxide_orm::Model for Tag {
    type Table = TagTable;
    type PrimaryKey = i64;

    fn pk_column() -> &'static str {
        "id"
    }

    fn pk(&self) -> Self::PrimaryKey {
        self.id
    }

    fn is_saved(&self) -> bool {
        self.id > 0
    }
}

// ============================================================================
// Type-Safe Query Examples
// ============================================================================
//
// These examples demonstrate compile-time validated SQL queries.
// Attempting to use non-existent columns results in a compile error.

/// Demonstrates type-safe SELECT queries.
///
/// The column types are validated at compile time - using an invalid column
/// name will cause a compilation error, not a runtime error.
#[allow(dead_code)]
fn example_select_queries() {
    // Select specific columns with type safety
    let (sql, _params) = Select::<PostTable, _, _>::new()
        .select::<(
            PostColumns::Id,
            PostColumns::Title,
            PostColumns::Status,
            PostColumns::CreatedAt,
        )>()
        .from_table()
        .where_clause(
            col(Post::status())
                .eq("published")
                .and(col(Post::author_id()).eq(1_i64)),
        )
        .order_by(Post::created_at(), false) // descending
        .limit(10)
        .build();

    println!("SELECT query: {}", sql);
    // Output: SELECT id, title, status, created_at FROM posts
    //         WHERE status = ? AND author_id = ? ORDER BY created_at DESC LIMIT 10

    // Select all columns
    let (sql, _) = Select::<PostTable, _, _>::new()
        .select_all()
        .from_table()
        .where_clause(col(Post::id()).eq(1_i64))
        .build();

    println!("SELECT * query: {}", sql);

    // Using IN clause
    let (sql, _) = Select::<PostTable, _, _>::new()
        .select::<(PostColumns::Id, PostColumns::Title)>()
        .from_table()
        .where_clause(col(Post::status()).in_list(vec!["published", "draft"]))
        .build();

    println!("SELECT with IN: {}", sql);
}

/// Demonstrates type-safe INSERT queries.
///
/// Each `.set()` call validates that the column belongs to the table.
#[allow(dead_code)]
fn example_insert_queries() {
    let (sql, _params) = Insert::<PostTable, _>::new()
        .set(Post::title(), "Hello World")
        .set(Post::slug(), "hello-world")
        .set(Post::content(), "This is my first post.")
        .set(Post::status(), "draft")
        .set(Post::author_id(), 1_i64)
        .set(Post::created_at(), "2024-01-15 10:00:00")
        .set(Post::updated_at(), "2024-01-15 10:00:00")
        .build();

    println!("INSERT query: {}", sql);
    // Output: INSERT INTO posts (title, slug, content, status, author_id,
    //         created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)
}

/// Demonstrates type-safe UPDATE queries.
///
/// Both the SET clause and WHERE clause columns are validated at compile time.
#[allow(dead_code)]
fn example_update_queries() {
    let (sql, _params) = Update::<PostTable, _>::new()
        .set(Post::title(), "Updated Title")
        .set(Post::status(), "published")
        .set(Post::updated_at(), "2024-01-16 12:00:00")
        .where_clause(col(Post::id()).eq(1_i64))
        .build();

    println!("UPDATE query: {}", sql);
    // Output: UPDATE posts SET title = ?, status = ?, updated_at = ?
    //         WHERE id = ?
}

/// Demonstrates type-safe DELETE queries.
///
/// The WHERE clause column is validated at compile time.
#[allow(dead_code)]
fn example_delete_queries() {
    let (sql, _params) = Delete::<PostTable>::new()
        .where_clause(col(Post::id()).eq(1_i64))
        .build();

    println!("DELETE query: {}", sql);
    // Output: DELETE FROM posts WHERE id = ?

    // Delete with complex condition
    let (sql, _) = Delete::<PostTable>::new()
        .where_clause(
            col(Post::status())
                .eq("draft")
                .and(col(Post::created_at()).lt("2024-01-01")),
        )
        .build();

    println!("DELETE with complex WHERE: {}", sql);
}

/// Print all example queries to demonstrate type safety.
fn print_type_safe_query_examples() {
    println!("\n-- =====================================================");
    println!("-- TYPE-SAFE QUERY EXAMPLES");
    println!("-- These queries have compile-time validated columns!");
    println!("-- =====================================================\n");

    example_select_queries();
    println!();
    example_insert_queries();
    println!();
    example_update_queries();
    println!();
    example_delete_queries();
    println!();
}

// ============================================================================
// In-memory data store
// ============================================================================

#[derive(Debug, Default)]
struct DataStore {
    posts: Vec<Post>,
    comments: Vec<Comment>,
    tags: Vec<Tag>,
    sessions: HashMap<String, String>, // session_id -> username
}

type AppState = Arc<RwLock<DataStore>>;

fn init_sample_data() -> DataStore {
    DataStore {
        posts: vec![
            Post {
                id: 1,
                title: "Welcome to Oxide Blog".to_string(),
                slug: "welcome-to-oxide-blog".to_string(),
                content: "This is the first post on our new blog platform.".to_string(),
                status: "published".to_string(),
                author_id: 1,
                created_at: "2024-01-15 10:00:00".to_string(),
                updated_at: "2024-01-15 10:00:00".to_string(),
            },
            Post {
                id: 2,
                title: "Getting Started with Rust".to_string(),
                slug: "getting-started-with-rust".to_string(),
                content: "Rust is a systems programming language focused on safety.".to_string(),
                status: "published".to_string(),
                author_id: 1,
                created_at: "2024-01-16 14:30:00".to_string(),
                updated_at: "2024-01-16 14:30:00".to_string(),
            },
            Post {
                id: 3,
                title: "Draft Post".to_string(),
                slug: "draft-post".to_string(),
                content: "This post is still being written.".to_string(),
                status: "draft".to_string(),
                author_id: 1,
                created_at: "2024-01-17 09:00:00".to_string(),
                updated_at: "2024-01-17 09:00:00".to_string(),
            },
        ],
        comments: vec![
            Comment {
                id: 1,
                post_id: 1,
                author: "Reader".to_string(),
                content: "Great first post!".to_string(),
                created_at: "2024-01-15 12:00:00".to_string(),
            },
            Comment {
                id: 2,
                post_id: 2,
                author: "Rustacean".to_string(),
                content: "Love Rust!".to_string(),
                created_at: "2024-01-16 15:00:00".to_string(),
            },
        ],
        tags: vec![
            Tag {
                id: 1,
                name: "Rust".to_string(),
                slug: "rust".to_string(),
            },
            Tag {
                id: 2,
                name: "Programming".to_string(),
                slug: "programming".to_string(),
            },
            Tag {
                id: 3,
                name: "Tutorial".to_string(),
                slug: "tutorial".to_string(),
            },
        ],
        sessions: HashMap::new(),
    }
}

// ============================================================================
// View Handlers
// ============================================================================

fn create_admin_site() -> AdminSite {
    AdminSite::new("Blog Admin")
        .register::<Post>(
            ModelAdmin::new()
                .list_display(&["id", "title", "status", "created_at"])
                .list_display_links(&["title"])
                .list_filter(&["status"])
                .search_fields(&["title", "content"])
                .ordering(&["-created_at"])
                .fieldset(Fieldset::named("Content", &["title", "slug", "content"]))
                .fieldset(Fieldset::named("Publishing", &["status", "author_id"]).collapse()),
        )
        .register::<Comment>(
            ModelAdmin::new()
                .list_display(&["id", "post_id", "author", "created_at"])
                .list_display_links(&["id"])
                .search_fields(&["author", "content"]),
        )
        .register::<Tag>(
            ModelAdmin::new()
                .list_display(&["id", "name", "slug"])
                .list_display_links(&["name"])
                .search_fields(&["name"]),
        )
}

async fn login_handler(req: Request, state: AppState) -> Response {
    if req.method == Method::Post {
        // Parse form data from body
        let body_str = req.body_string().unwrap_or_default();
        let form_data = Request::parse_query_string(&body_str);

        let username = form_data.get("username").map(|s| s.as_str()).unwrap_or("");
        let password = form_data.get("password").map(|s| s.as_str()).unwrap_or("");

        // Simple auth check
        if username == "admin" && password == "admin123" {
            let session_id = format!("session_{}", rand::random::<u64>());
            {
                let mut store = state.write().await;
                store
                    .sessions
                    .insert(session_id.clone(), username.to_string());
            }

            let next = req.get_query("next").unwrap_or("/admin/");
            return Response::redirect(next)
                .header("Set-Cookie", format!("session_id={}; Path=/", session_id));
        }

        // Invalid credentials
        return Response::html(render_login_page(Some("Invalid username or password")));
    }

    Response::html(render_login_page(None))
}

async fn logout_handler(_req: Request, _state: AppState) -> Response {
    // In a real app, we'd parse the cookie and remove the session
    // For simplicity, we'll just redirect
    Response::redirect("/admin/login").header("Set-Cookie", "session_id=; Path=/; Max-Age=0")
}

async fn dashboard_handler(req: Request, state: AppState) -> Response {
    // Check authentication
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login?next=/admin/");
    }

    let store = state.read().await;
    let html = render_dashboard(&store);
    Response::html(html)
}

async fn list_posts_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login?next=/admin/posts/");
    }

    let store = state.read().await;
    let admin = create_admin_site();

    // Parse query params
    let page: usize = req
        .get_query("page")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let search = req.get_query("q").unwrap_or("");
    let status_filter = req.get_query("status");

    // Filter posts
    let mut posts: Vec<&Post> = store.posts.iter().collect();

    if !search.is_empty() {
        posts.retain(|p| {
            p.title.to_lowercase().contains(&search.to_lowercase())
                || p.content.to_lowercase().contains(&search.to_lowercase())
        });
    }

    if let Some(status) = status_filter {
        posts.retain(|p| p.status == status);
    }

    let html = render_post_list(&posts, page, search, status_filter, &admin);
    Response::html(html)
}

async fn add_post_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login?next=/admin/posts/add/");
    }

    if req.method == Method::Post {
        let body_str = req.body_string().unwrap_or_default();
        let form_data = Request::parse_query_string(&body_str);

        let title = form_data.get("title").cloned().unwrap_or_default();
        let slug = form_data.get("slug").cloned().unwrap_or_default();
        let content = form_data.get("content").cloned().unwrap_or_default();
        let status = form_data
            .get("status")
            .cloned()
            .unwrap_or("draft".to_string());

        if title.is_empty() {
            return Response::html(render_post_form(None, Some("Title is required")));
        }

        let mut store = state.write().await;
        let new_id = store.posts.iter().map(|p| p.id).max().unwrap_or(0) + 1;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        store.posts.push(Post {
            id: new_id,
            title,
            slug,
            content,
            status,
            author_id: 1,
            created_at: now.clone(),
            updated_at: now,
        });

        return Response::redirect("/admin/posts/")
            .header("X-Message", "Post created successfully".to_string());
    }

    Response::html(render_post_form(None, None))
}

async fn change_post_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    let pk: i64 = req.params.parse("pk").unwrap_or(0);

    if req.method == Method::Post {
        let body_str = req.body_string().unwrap_or_default();
        let form_data = Request::parse_query_string(&body_str);

        let title = form_data.get("title").cloned().unwrap_or_default();
        let slug = form_data.get("slug").cloned().unwrap_or_default();
        let content = form_data.get("content").cloned().unwrap_or_default();
        let status = form_data
            .get("status")
            .cloned()
            .unwrap_or("draft".to_string());

        if title.is_empty() {
            return Response::html(render_post_form(None, Some("Title is required")));
        }

        let mut store = state.write().await;
        if let Some(post) = store.posts.iter_mut().find(|p| p.id == pk) {
            post.title = title;
            post.slug = slug;
            post.content = content;
            post.status = status;
            post.updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        }

        return Response::redirect("/admin/posts/");
    }

    let store = state.read().await;
    let post = store.posts.iter().find(|p| p.id == pk);

    match post {
        Some(p) => Response::html(render_post_form(Some(p), None)),
        None => Response::not_found(),
    }
}

async fn delete_post_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    let pk: i64 = req.params.parse("pk").unwrap_or(0);

    if req.method == Method::Post {
        let mut store = state.write().await;
        store.posts.retain(|p| p.id != pk);
        return Response::redirect("/admin/posts/");
    }

    let store = state.read().await;
    let post = store.posts.iter().find(|p| p.id == pk);

    match post {
        Some(p) => Response::html(render_delete_confirmation(p)),
        None => Response::not_found(),
    }
}

async fn action_posts_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    let body_str = req.body_string().unwrap_or_default();
    let form_data = Request::parse_query_string(&body_str);

    let action = form_data.get("action").map(|s| s.as_str()).unwrap_or("");

    // Collect selected IDs from "selected=1&selected=2&..." format
    let selected: Vec<i64> = body_str
        .split('&')
        .filter_map(|pair| {
            let (key, val) = pair.split_once('=')?;
            if key == "selected" {
                val.parse().ok()
            } else {
                None
            }
        })
        .collect();

    if action == "delete_selected" && !selected.is_empty() {
        let mut store = state.write().await;
        store.posts.retain(|p| !selected.contains(&p.id));
    }

    Response::redirect("/admin/posts/")
}

// ============================================================================
// Comments handlers
// ============================================================================

async fn list_comments_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login?next=/admin/comments/");
    }

    let store = state.read().await;
    let search = req.get_query("q").unwrap_or("");

    let mut comments: Vec<&Comment> = store.comments.iter().collect();
    if !search.is_empty() {
        comments.retain(|c| {
            c.author.to_lowercase().contains(&search.to_lowercase())
                || c.content.to_lowercase().contains(&search.to_lowercase())
        });
    }

    let html = render_comment_list(&comments, search);
    Response::html(html)
}

async fn add_comment_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    if req.method == Method::Post {
        let body_str = req.body_string().unwrap_or_default();
        let form_data = Request::parse_query_string(&body_str);

        let post_id: i64 = form_data
            .get("post_id")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let author = form_data.get("author").cloned().unwrap_or_default();
        let content = form_data.get("content").cloned().unwrap_or_default();

        if author.is_empty() {
            return Response::html(render_comment_form(None, Some("Author is required")));
        }

        let mut store = state.write().await;
        let new_id = store.comments.iter().map(|c| c.id).max().unwrap_or(0) + 1;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        store.comments.push(Comment {
            id: new_id,
            post_id,
            author,
            content,
            created_at: now,
        });

        return Response::redirect("/admin/comments/");
    }

    Response::html(render_comment_form(None, None))
}

async fn change_comment_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    let pk: i64 = req.params.parse("pk").unwrap_or(0);

    if req.method == Method::Post {
        let body_str = req.body_string().unwrap_or_default();
        let form_data = Request::parse_query_string(&body_str);

        let post_id: i64 = form_data
            .get("post_id")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let author = form_data.get("author").cloned().unwrap_or_default();
        let content = form_data.get("content").cloned().unwrap_or_default();

        let mut store = state.write().await;
        if let Some(comment) = store.comments.iter_mut().find(|c| c.id == pk) {
            comment.post_id = post_id;
            comment.author = author;
            comment.content = content;
        }

        return Response::redirect("/admin/comments/");
    }

    let store = state.read().await;
    match store.comments.iter().find(|c| c.id == pk) {
        Some(c) => Response::html(render_comment_form(Some(c), None)),
        None => Response::not_found(),
    }
}

async fn delete_comment_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    let pk: i64 = req.params.parse("pk").unwrap_or(0);

    if req.method == Method::Post {
        let mut store = state.write().await;
        store.comments.retain(|c| c.id != pk);
        return Response::redirect("/admin/comments/");
    }

    let store = state.read().await;
    match store.comments.iter().find(|c| c.id == pk) {
        Some(c) => {
            let msg = format!("You are about to delete comment #{} by ", c.id);
            let author_ref = &c.author;
            let heading = html! {
                h1.class("text-2xl font-semibold text-gray-900 mb-6") {
                    "Delete Comment"
                }
            };
            let warning = html! {
                div.class("bg-amber-50 border border-amber-200 rounded-lg p-6 mb-6") {
                    h4.class("text-lg font-semibold text-amber-800 mb-2") {
                        "Are you sure?"
                    }
                    p.class("text-amber-700 mb-2") {
                        #msg
                        strong { #author_ref }
                    }
                    p.class("text-amber-700") {
                        "This action cannot be undone."
                    }
                }
            };
            let confirm_btn = html! {
                button.type_("submit").class("px-6 py-2 bg-red-600 text-white font-medium rounded-lg hover:bg-red-700 transition-colors duration-200") { "Confirm Delete" }
            };
            let cancel_link = html! {
                a.href("/admin/comments/").class("px-6 py-2 border border-gray-300 text-gray-700 font-medium rounded-lg hover:bg-gray-50 transition-colors duration-200") { "Cancel" }
            };
            let confirm_btn_r = confirm_btn.render();
            let cancel_link_r = cancel_link.render();
            let content = Element::<Div>::new()
                .raw(heading.render())
                .child::<Div, _>(|d| {
                    d.class("delete-confirmation max-w-2xl")
                        .raw(warning.render())
                        .child::<Form, _>(|f| {
                            f.attr("method", "POST")
                                .class("flex gap-4")
                                .child::<Div, _>(|d| d.raw(&confirm_btn_r).raw(&cancel_link_r))
                        })
                })
                .render();
            Response::html(render_base("Delete Comment", &content, true))
        }
        None => Response::not_found(),
    }
}

async fn action_comments_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    let body_str = req.body_string().unwrap_or_default();
    let form_data = Request::parse_query_string(&body_str);
    let action = form_data.get("action").map(|s| s.as_str()).unwrap_or("");

    let selected: Vec<i64> = body_str
        .split('&')
        .filter_map(|pair| {
            let (key, val) = pair.split_once('=')?;
            if key == "selected" {
                val.parse().ok()
            } else {
                None
            }
        })
        .collect();

    if action == "delete_selected" && !selected.is_empty() {
        let mut store = state.write().await;
        store.comments.retain(|c| !selected.contains(&c.id));
    }

    Response::redirect("/admin/comments/")
}

// ============================================================================
// Tags handlers
// ============================================================================

async fn list_tags_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login?next=/admin/tags/");
    }

    let store = state.read().await;
    let search = req.get_query("q").unwrap_or("");

    let mut tags: Vec<&Tag> = store.tags.iter().collect();
    if !search.is_empty() {
        tags.retain(|t| t.name.to_lowercase().contains(&search.to_lowercase()));
    }

    let html = render_tag_list(&tags, search);
    Response::html(html)
}

async fn add_tag_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    if req.method == Method::Post {
        let body_str = req.body_string().unwrap_or_default();
        let form_data = Request::parse_query_string(&body_str);

        let name = form_data.get("name").cloned().unwrap_or_default();
        let slug = form_data.get("slug").cloned().unwrap_or_default();

        if name.is_empty() {
            return Response::html(render_tag_form(None, Some("Name is required")));
        }

        let mut store = state.write().await;
        let new_id = store.tags.iter().map(|t| t.id).max().unwrap_or(0) + 1;

        store.tags.push(Tag {
            id: new_id,
            name,
            slug,
        });

        return Response::redirect("/admin/tags/");
    }

    Response::html(render_tag_form(None, None))
}

async fn change_tag_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    let pk: i64 = req.params.parse("pk").unwrap_or(0);

    if req.method == Method::Post {
        let body_str = req.body_string().unwrap_or_default();
        let form_data = Request::parse_query_string(&body_str);

        let name = form_data.get("name").cloned().unwrap_or_default();
        let slug = form_data.get("slug").cloned().unwrap_or_default();

        let mut store = state.write().await;
        if let Some(tag) = store.tags.iter_mut().find(|t| t.id == pk) {
            tag.name = name;
            tag.slug = slug;
        }

        return Response::redirect("/admin/tags/");
    }

    let store = state.read().await;
    match store.tags.iter().find(|t| t.id == pk) {
        Some(t) => Response::html(render_tag_form(Some(t), None)),
        None => Response::not_found(),
    }
}

async fn delete_tag_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    let pk: i64 = req.params.parse("pk").unwrap_or(0);

    if req.method == Method::Post {
        let mut store = state.write().await;
        store.tags.retain(|t| t.id != pk);
        return Response::redirect("/admin/tags/");
    }

    let store = state.read().await;
    match store.tags.iter().find(|t| t.id == pk) {
        Some(t) => {
            let name_ref = &t.name;
            let heading = html! {
                h1.class("text-2xl font-semibold text-gray-900 mb-6") {
                    "Delete Tag"
                }
            };
            let warning = html! {
                div.class("bg-amber-50 border border-amber-200 rounded-lg p-6 mb-6") {
                    h4.class("text-lg font-semibold text-amber-800 mb-2") {
                        "Are you sure?"
                    }
                    p.class("text-amber-700 mb-2") {
                        "You are about to delete the tag: "
                        strong { #name_ref }
                    }
                    p.class("text-amber-700") {
                        "This action cannot be undone."
                    }
                }
            };
            let confirm_btn = html! {
                button.type_("submit").class("px-6 py-2 bg-red-600 text-white font-medium rounded-lg hover:bg-red-700 transition-colors duration-200") { "Confirm Delete" }
            };
            let cancel_link = html! {
                a.href("/admin/tags/").class("px-6 py-2 border border-gray-300 text-gray-700 font-medium rounded-lg hover:bg-gray-50 transition-colors duration-200") { "Cancel" }
            };
            let confirm_btn_r = confirm_btn.render();
            let cancel_link_r = cancel_link.render();
            let content = Element::<Div>::new()
                .raw(heading.render())
                .child::<Div, _>(|d| {
                    d.class("delete-confirmation max-w-2xl")
                        .raw(warning.render())
                        .child::<Form, _>(|f| {
                            f.attr("method", "POST")
                                .class("flex gap-4")
                                .child::<Div, _>(|d| d.raw(&confirm_btn_r).raw(&cancel_link_r))
                        })
                })
                .render();
            Response::html(render_base("Delete Tag", &content, true))
        }
        None => Response::not_found(),
    }
}

async fn action_tags_handler(req: Request, state: AppState) -> Response {
    if !is_authenticated(&req, &state).await {
        return Response::redirect("/admin/login");
    }

    let body_str = req.body_string().unwrap_or_default();
    let form_data = Request::parse_query_string(&body_str);
    let action = form_data.get("action").map(|s| s.as_str()).unwrap_or("");

    let selected: Vec<i64> = body_str
        .split('&')
        .filter_map(|pair| {
            let (key, val) = pair.split_once('=')?;
            if key == "selected" {
                val.parse().ok()
            } else {
                None
            }
        })
        .collect();

    if action == "delete_selected" && !selected.is_empty() {
        let mut store = state.write().await;
        store.tags.retain(|t| !selected.contains(&t.id));
    }

    Response::redirect("/admin/tags/")
}

// ============================================================================
// Template Rendering
// ============================================================================

fn render_base(title: &str, content: &str, is_logged_in: bool) -> String {
    let tailwind_config_js = r#"
        tailwind.config = {
            darkMode: 'class',
            theme: {
                extend: {
                    colors: {
                        primary: '#3b82f6',
                        muted: '#6b7280',
                        destructive: '#ef4444',
                    }
                }
            }
        }
    "#;

    Document::new()
        .doctype()
        .root::<Html, _>(|html| {
            html.attr("lang", "en")
                .class("h-full")
                .child::<Head, _>(|head| {
                    head.child::<Meta, _>(|m| m.attr("charset", "UTF-8"))
                        .child::<Meta, _>(|m| {
                            m.attr("name", "viewport")
                                .attr("content", "width=device-width, initial-scale=1.0")
                        })
                        .child::<Title, _>(|t| t.text(format!("{} | Blog Admin", title)))
                        .child::<Script, _>(|s| s.attr("src", "https://cdn.tailwindcss.com"))
                        .child::<Script, _>(|s| s.raw(tailwind_config_js))
                })
                .child::<Body, _>(|body| {
                    body.class("h-full bg-gray-50").child::<Div, _>(|d| {
                        d.class("flex h-full")
                            .when(is_logged_in, |d| d.raw(render_sidebar()))
                            .child::<Main, _>(|m| {
                                m.class("flex-1 p-6 overflow-auto")
                                    .child::<Div, _>(|d| d.raw(content))
                            })
                    })
                })
        })
        .build()
}

fn render_sidebar() -> String {
    let nav_link_class = "block px-4 py-2 rounded-lg hover:bg-gray-700 hover:text-white transition-colors duration-200";

    let heading = html! {
        h2.class("text-white text-lg font-semibold mb-6") { "Blog Admin" }
    };
    let dashboard_link = html! { a.href("/admin/").class(#nav_link_class) { "Dashboard" } };
    let posts_link = html! { a.href("/admin/posts/").class(#nav_link_class) { "Posts" } };
    let comments_link = html! { a.href("/admin/comments/").class(#nav_link_class) { "Comments" } };
    let tags_link = html! { a.href("/admin/tags/").class(#nav_link_class) { "Tags" } };
    let logout_link = html! {
        a.href("/admin/logout").class("block px-4 py-2 rounded-lg text-red-400 hover:bg-gray-700 hover:text-red-300 transition-colors duration-200") { "Logout" }
    };
    let dash_r = dashboard_link.render();
    let posts_r = posts_link.render();
    let comments_r = comments_link.render();
    let tags_r = tags_link.render();
    let logout_r = logout_link.render();
    let hr = html! { hr.class("my-6 border-gray-600") };

    Element::<Nav>::new()
        .class("w-64 min-h-screen bg-gray-800 text-gray-300 flex-shrink-0")
        .child::<Div, _>(|d| {
            d.class("sticky top-0 p-4")
                .raw(heading.render())
                .child::<Ul, _>(|ul| {
                    ul.class("space-y-2")
                        .child::<Li, _>(|li| li.raw(&dash_r))
                        .child::<Li, _>(|li| li.raw(&posts_r))
                        .child::<Li, _>(|li| li.raw(&comments_r))
                        .child::<Li, _>(|li| li.raw(&tags_r))
                })
                .raw(hr.render())
                .child::<Ul, _>(|ul| ul.child::<Li, _>(|li| li.raw(&logout_r)))
        })
        .render()
}

fn render_login_page(error: Option<&str>) -> String {
    let input_class = "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-200";
    let error_html = error
        .map(|e| {
            (html! {
                div.class("mb-4 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg") {
                    #e
                }
            })
            .render()
        })
        .unwrap_or_default();

    let header = html! {
        div.class("px-6 py-4 border-b border-gray-200") {
            h4.class("text-xl font-semibold text-gray-900") { "Admin Login" }
        }
    };
    let username_label = html! {
        label.for_("username").class("block text-sm font-medium text-gray-700 mb-1") { "Username" }
    };
    let username_input = html! {
        input.type_("text").id("username").name("username").required.class(#input_class)
    };
    let password_label = html! {
        label.for_("password").class("block text-sm font-medium text-gray-700 mb-1") { "Password" }
    };
    let password_input = html! {
        input.type_("password").id("password").name("password").required.class(#input_class)
    };
    let submit_btn = html! {
        button.type_("submit").class("w-full px-4 py-2 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-colors duration-200") { "Login" }
    };
    let hint = html! {
        p.class("mt-4 text-sm text-gray-500") { "Default: admin / admin123" }
    };
    let username_label_r = username_label.render();
    let username_input_r = username_input.render();
    let password_label_r = password_label.render();
    let password_input_r = password_input.render();
    let submit_btn_r = submit_btn.render();

    let content = Element::<Div>::new()
        .class("min-h-screen flex items-center justify-center")
        .child::<Div, _>(|d| {
            d.class("w-full max-w-md").child::<Div, _>(|d| {
                d.class("bg-white rounded-lg shadow-sm border border-gray-200")
                    .raw(header.render())
                    .child::<Div, _>(|d| {
                        d.class("p-6")
                            .raw(&error_html)
                            .child::<Form, _>(|f| {
                                f.attr("method", "POST")
                                    .class("space-y-4")
                                    .child::<Div, _>(|d| {
                                        d.raw(&username_label_r).raw(&username_input_r)
                                    })
                                    .child::<Div, _>(|d| {
                                        d.raw(&password_label_r).raw(&password_input_r)
                                    })
                                    .child::<Div, _>(|d| d.raw(&submit_btn_r))
                            })
                            .raw(hint.render())
                    })
            })
        })
        .render();

    render_base("Login", &content, false)
}

fn render_dashboard_card(
    bg: &str,
    text_color: &str,
    label: &str,
    count: usize,
    href: &str,
    data_model: &str,
) -> String {
    let count_str = count.to_string();
    let card_class = format!("{} text-white rounded-lg shadow-sm p-6", bg);
    let btn_class = format!(
        "inline-block mt-4 px-4 py-2 bg-white {} text-sm font-medium rounded-lg hover:bg-gray-100 transition-colors duration-200",
        text_color
    );
    let heading = html! {
        h5.class("text-lg font-medium opacity-90") { #label }
    };
    let count_el = html! {
        p.class("text-4xl font-bold mt-2").data_model(#data_model) {
            span.class("count") { #count_str }
        }
    };
    let link = html! {
        a.href(#href).class(#btn_class) { "View all" }
    };
    (html! {
        div.class(#card_class) {}
    })
    .raw(heading.render())
    .raw(count_el.render())
    .raw(link.render())
    .render()
}

fn render_dashboard(store: &DataStore) -> String {
    let mut cards = String::new();
    cards.push_str(&render_dashboard_card(
        "bg-blue-600",
        "text-blue-600",
        "Posts",
        store.posts.len(),
        "/admin/posts/",
        "posts",
    ));
    cards.push_str(&render_dashboard_card(
        "bg-green-600",
        "text-green-600",
        "Comments",
        store.comments.len(),
        "/admin/comments/",
        "comments",
    ));
    cards.push_str(&render_dashboard_card(
        "bg-cyan-600",
        "text-cyan-600",
        "Tags",
        store.tags.len(),
        "/admin/tags/",
        "tags",
    ));

    let heading = html! {
        h1.class("text-2xl font-semibold text-gray-900 mb-6") { "Dashboard" }
    };
    let content = Element::<Div>::new()
        .raw(heading.render())
        .child::<Div, _>(|d| d.class("grid grid-cols-1 md:grid-cols-3 gap-6").raw(&cards))
        .render();

    render_base("Dashboard", &content, true)
}

fn render_post_list(
    posts: &[&Post],
    page: usize,
    search: &str,
    status_filter: Option<&str>,
    _admin: &AdminSite,
) -> String {
    let per_page = 25;
    let total_pages = posts.len().div_ceil(per_page);
    let start = (page - 1) * per_page;
    let page_posts: Vec<_> = posts.iter().skip(start).take(per_page).collect();

    let th_class = "px-4 py-3 text-left text-sm font-medium text-gray-600";
    let edit_btn_class = "px-3 py-1 text-sm border border-blue-600 text-blue-600 rounded hover:bg-blue-50 transition-colors duration-200";
    let del_btn_class = "px-3 py-1 text-sm border border-red-600 text-red-600 rounded hover:bg-red-50 transition-colors duration-200";

    let mut rows = String::new();
    for p in &page_posts {
        let badge_class = if p.status == "published" {
            "bg-green-100 text-green-800"
        } else {
            "bg-gray-100 text-gray-800"
        };
        let id_str = p.id.to_string();
        let edit_url = format!("/admin/posts/{}/change/", p.id);
        let del_url = format!("/admin/posts/{}/delete/", p.id);
        let checkbox = html! {
            input.type_("checkbox").name("selected").value(#&id_str).class("rounded border-gray-300")
        };
        let title_ref = &p.title;
        let title_link = html! {
            a.href(#&edit_url).class("text-blue-600 hover:text-blue-800 font-medium") { #title_ref }
        };
        let badge_cls = format!("px-2 py-1 text-xs font-medium rounded-full {}", badge_class);
        let status_ref = &p.status;
        let badge = html! {
            span.class(#badge_cls) { #status_ref }
        };
        let edit_link = html! {
            a.href(#&edit_url).class(#edit_btn_class) { "Edit" }
        };
        let del_link = html! {
            a.href(#del_url).class(#del_btn_class) { "Delete" }
        };
        let checkbox_r = checkbox.render();
        let title_link_r = title_link.render();
        let badge_r = badge.render();
        let edit_link_r = edit_link.render();
        let del_link_r = del_link.render();
        let created_ref = &p.created_at;
        Element::<Tr>::new()
            .class("border-b border-gray-100 hover:bg-gray-50")
            .child::<Td, _>(|td| td.class("px-4 py-3").raw(&checkbox_r))
            .child::<Td, _>(|td| td.class("px-4 py-3 text-gray-600").text(&id_str))
            .child::<Td, _>(|td| td.class("px-4 py-3").raw(&title_link_r))
            .child::<Td, _>(|td| td.class("px-4 py-3").raw(&badge_r))
            .child::<Td, _>(|td| {
                td.class("px-4 py-3 text-gray-500 text-sm")
                    .text(created_ref)
            })
            .child::<Td, _>(|td| {
                td.class("px-4 py-3 space-x-2")
                    .raw(&edit_link_r)
                    .raw(&del_link_r)
            })
            .render_to(&mut rows);
    }

    let pagination = render_pagination_nav(page, total_pages);

    let pub_selected = status_filter == Some("published");
    let draft_selected = status_filter == Some("draft");

    let page_heading = html! {
        h1.class("text-2xl font-semibold text-gray-900 page-title") { "Posts" }
    };
    let add_btn = html! {
        a.href("/admin/posts/add/").class("px-4 py-2 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 transition-colors duration-200") { "+ Add Post" }
    };
    let content = Element::<Div>::new()
        .child::<Div, _>(|d| {
            d.class("flex justify-between items-center mb-6")
                .raw(page_heading.render())
                .raw(add_btn.render())
        })
        .child::<Div, _>(|d| {
            d.class("bg-white rounded-lg shadow-sm border border-gray-200 mb-6")
                .child::<Div, _>(|d| {
                    d.class("p-4").child::<Form, _>(|f| {
                        f.class("flex gap-4").attr("method", "GET")
                            .child::<ironhtml_elements::Input, _>(|i| {
                                i.attr("type", "search").attr("name", "q")
                                    .class("flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500")
                                    .attr("placeholder", "Search...").attr("value", search)
                            })
                            .child::<SelectEl, _>(|s| {
                                let s = s.attr("name", "status")
                                    .class("px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 filter-select")
                                    .child::<Option_, _>(|o| o.attr("value", "").text("All statuses"));
                                let s = s.child::<Option_, _>(|o| {
                                    let o = o.attr("value", "published").text("Published");
                                    if pub_selected { o.bool_attr("selected") } else { o }
                                });
                                s.child::<Option_, _>(|o| {
                                    let o = o.attr("value", "draft").text("Draft");
                                    if draft_selected { o.bool_attr("selected") } else { o }
                                })
                            })
                            .child::<ironhtml_elements::Button, _>(|b| {
                                b.attr("type", "submit")
                                    .class("px-4 py-2 border border-blue-600 text-blue-600 font-medium rounded-lg hover:bg-blue-50 transition-colors duration-200")
                                    .text("Filter")
                            })
                    })
                })
        })
        .raw(render_action_form_wrapper(
            "/admin/posts/action/",
            &["ID", "Title", "Status", "Created", "Actions"],
            &[Some("?order=id"), Some("?order=title"), Some("?order=status"), Some("?order=created_at"), None],
            &rows,
            th_class,
        ))
        .raw(&pagination)
        .render();

    render_base("Posts", &content, true)
}

fn render_action_form_wrapper(
    action_url: &str,
    headers: &[&str],
    sort_links: &[Option<&str>],
    rows_html: &str,
    th_class: &str,
) -> String {
    // Build header cells as raw HTML (Th elements)
    let mut header_cells = String::new();
    let select_all_th = html! {
        th.class("px-4 py-3 text-left w-10") {
            input.type_("checkbox").id("select-all").class("rounded border-gray-300")
        }
    };
    select_all_th.render_to(&mut header_cells);
    for (h, link) in headers.iter().zip(sort_links.iter()) {
        if let Some(href) = link {
            let href_val = *href;
            let h_val = *h;
            let sort_link = html! {
                a.href(#href_val).class("hover:text-gray-900") { #h_val }
            };
            Element::<Th>::new()
                .class(th_class)
                .bool_attr("data-sortable")
                .raw(sort_link.render())
                .render_to(&mut header_cells);
        } else {
            Element::<Th>::new()
                .class(th_class)
                .text(*h)
                .render_to(&mut header_cells);
        }
    }

    // Build table as raw HTML since Tr/Tbody can't use .raw()
    let table_html = format!(
        "<table class=\"w-full list-view\">\
         <thead class=\"bg-gray-50 border-b border-gray-200\">\
         <tr>{header_cells}</tr></thead>\
         <tbody>{rows_html}</tbody></table>"
    );

    let apply_btn = html! {
        button.type_("submit").name("apply_action").class("px-4 py-2 border border-gray-300 text-gray-700 text-sm font-medium rounded-lg hover:bg-gray-50 transition-colors duration-200") { "Apply" }
    };
    let apply_btn_r = apply_btn.render();

    Element::<Form>::new()
        .attr("method", "POST")
        .attr("action", action_url)
        .child::<Div, _>(|d| {
            d.class("bg-white rounded-lg shadow-sm border border-gray-200")
                .child::<Div, _>(|d| {
                    d.class("px-4 py-3 border-b border-gray-200 flex items-center gap-4")
                        .child::<SelectEl, _>(|s| {
                            s.attr("name", "action")
                                .class("px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 action-select")
                                .child::<Option_, _>(|o| o.attr("value", "").text("-- Select action --"))
                                .child::<Option_, _>(|o| {
                                    o.attr("value", "delete_selected").text("Delete selected")
                                })
                        })
                        .raw(&apply_btn_r)
                })
                .child::<Div, _>(|d| d.class("overflow-x-auto").raw(&table_html))
        })
        .render()
}

fn render_pagination_nav(page: usize, total_pages: usize) -> String {
    if total_pages <= 1 {
        return String::new();
    }
    let mut links = String::new();
    for i in 1..=total_pages {
        let active_class = if i == page {
            "bg-blue-600 text-white"
        } else {
            "bg-white text-gray-700 hover:bg-gray-50"
        };
        let href = format!("?page={}", i);
        let i_str = i.to_string();
        let link_class = format!(
            "px-4 py-2 text-sm font-medium border border-gray-300 rounded-lg {}",
            active_class
        );
        let link = html! {
            a.href(#href).class(#link_class) { #i_str }
        };
        link.render_to(&mut links);
    }
    Element::<Nav>::new()
        .class("mt-6 flex justify-center")
        .child::<Div, _>(|d| d.class("flex gap-1").raw(&links))
        .render()
}

fn render_post_form(post: Option<&Post>, error: Option<&str>) -> String {
    let is_new = post.is_none();
    let title = if is_new { "Add Post" } else { "Edit Post" };

    let (post_title, post_slug, post_content, post_status) = post
        .map(|p| {
            (
                p.title.as_str(),
                p.slug.as_str(),
                p.content.as_str(),
                p.status.as_str(),
            )
        })
        .unwrap_or(("", "", "", "draft"));

    let input_class = "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-200";
    let label_class = "block text-sm font-medium text-gray-700 mb-1";
    let error_html = render_error_alert(error);
    let delete_btn = post.map(|p| {
        let href = format!("/admin/posts/{}/delete/", p.id);
        (html! {
            a.href(#href).class("ml-auto px-4 py-2 border border-red-600 text-red-600 font-medium rounded-lg hover:bg-red-50 transition-colors duration-200") { "Delete" }
        }).render()
    }).unwrap_or_default();

    let draft_selected = post_status == "draft";
    let pub_selected = post_status == "published";

    let heading = html! {
        h1.class("text-2xl font-semibold text-gray-900 mb-6") { #title }
    };
    let content_header = html! {
        div.class("px-6 py-4 border-b border-gray-200") {
            h5.class("text-lg font-medium text-gray-900") { "Content" }
        }
    };
    let title_label = html! {
        label.for_("title").class(#label_class) { "Title *" }
    };
    let title_input = html! {
        input.type_("text").id("title").name("title").required.class(#input_class).value(#post_title)
    };
    let slug_label = html! {
        label.for_("slug").class(#label_class) { "Slug" }
    };
    let slug_input = html! {
        input.type_("text").id("slug").name("slug").class(#input_class).value(#post_slug)
    };
    let content_label = html! {
        label.for_("content").class(#label_class) { "Content" }
    };
    let content_textarea = html! {
        textarea.id("content").name("content").rows("10").class(#input_class) { #post_content }
    };
    let pub_header = html! {
        div.class("px-6 py-4 border-b border-gray-200") {
            h5.class("text-lg font-medium text-gray-900") { "Publishing" }
        }
    };
    let status_label = html! {
        label.for_("status").class(#label_class) { "Status" }
    };
    let title_label_r = title_label.render();
    let title_input_r = title_input.render();
    let slug_label_r = slug_label.render();
    let slug_input_r = slug_input.render();
    let content_label_r = content_label.render();
    let content_textarea_r = content_textarea.render();
    let status_label_r = status_label.render();

    let content = Element::<Div>::new()
        .raw(heading.render())
        .raw(&error_html)
        .child::<Form, _>(|f| {
            f.attr("method", "POST")
                .class("space-y-6")
                .child::<Div, _>(|d| {
                    d.class("bg-white rounded-lg shadow-sm border border-gray-200")
                        .raw(content_header.render())
                        .child::<Div, _>(|d| {
                            d.class("p-6 space-y-4")
                                .child::<Div, _>(|d| d.raw(&title_label_r).raw(&title_input_r))
                                .child::<Div, _>(|d| d.raw(&slug_label_r).raw(&slug_input_r))
                                .child::<Div, _>(|d| {
                                    d.raw(&content_label_r).raw(&content_textarea_r)
                                })
                        })
                })
                .child::<Div, _>(|d| {
                    d.class("bg-white rounded-lg shadow-sm border border-gray-200")
                        .raw(pub_header.render())
                        .child::<Div, _>(|d| {
                            d.class("p-6").child::<Div, _>(|d| {
                                d.raw(&status_label_r).child::<SelectEl, _>(|s| {
                                    s.id("status")
                                        .attr("name", "status")
                                        .class(input_class)
                                        .child::<Option_, _>(|o| {
                                            let o = o.attr("value", "draft").text("Draft");
                                            if draft_selected {
                                                o.bool_attr("selected")
                                            } else {
                                                o
                                            }
                                        })
                                        .child::<Option_, _>(|o| {
                                            let o = o.attr("value", "published").text("Published");
                                            if pub_selected {
                                                o.bool_attr("selected")
                                            } else {
                                                o
                                            }
                                        })
                                })
                            })
                        })
                })
                .child::<Div, _>(|d| d.raw(render_form_buttons("/admin/posts/", &delete_btn)))
        })
        .render();

    render_base(title, &content, true)
}

fn render_error_alert(error: Option<&str>) -> String {
    error
        .map(|e| {
            (html! {
                div.class("mb-6 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg") {
                    #e
                }
            })
            .render()
        })
        .unwrap_or_default()
}

fn render_form_buttons(cancel_url: &str, delete_btn_html: &str) -> String {
    let save_btn = html! {
        button.type_("submit").class("px-6 py-2 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-colors duration-200") { "Save" }
    };
    let cancel_link = html! {
        a.href(#cancel_url).class("px-6 py-2 border border-gray-300 text-gray-700 font-medium rounded-lg hover:bg-gray-50 transition-colors duration-200") { "Cancel" }
    };
    (html! { div.class("flex gap-4") {} })
        .raw(save_btn.render())
        .raw(cancel_link.render())
        .raw(delete_btn_html)
        .render()
}

fn render_delete_page(
    page_title: &str,
    item_label: &str,
    item_name: &str,
    cancel_url: &str,
) -> String {
    let heading = html! {
        h1.class("text-2xl font-semibold text-gray-900 mb-6") { #page_title }
    };
    let msg = format!("You are about to delete the {}: ", item_label);
    let warning = html! {
        div.class("bg-amber-50 border border-amber-200 rounded-lg p-6 mb-6") {
            h4.class("text-lg font-semibold text-amber-800 mb-2") { "Are you sure?" }
            p.class("text-amber-700 mb-2") {
                #msg
                strong.class("font-semibold") { #item_name }
            }
            p.class("text-amber-700") { "This action cannot be undone." }
        }
    };
    let confirm_btn = html! {
        button.type_("submit").class("px-6 py-2 bg-red-600 text-white font-medium rounded-lg hover:bg-red-700 focus:ring-2 focus:ring-red-500 focus:ring-offset-2 transition-colors duration-200") { "Confirm Delete" }
    };
    let cancel_link = html! {
        a.href(#cancel_url).class("px-6 py-2 border border-gray-300 text-gray-700 font-medium rounded-lg hover:bg-gray-50 transition-colors duration-200") { "Cancel" }
    };
    let confirm_btn_r = confirm_btn.render();
    let cancel_link_r = cancel_link.render();
    let content = Element::<Div>::new()
        .raw(heading.render())
        .child::<Div, _>(|d| {
            d.class("delete-confirmation max-w-2xl")
                .raw(warning.render())
                .child::<Form, _>(|f| {
                    f.attr("method", "POST")
                        .class("flex gap-4")
                        .child::<Div, _>(|d| d.raw(&confirm_btn_r).raw(&cancel_link_r))
                })
        })
        .render();
    render_base(page_title, &content, true)
}

fn render_delete_confirmation(post: &Post) -> String {
    let cancel_url = format!("/admin/posts/{}/change/", post.id);
    render_delete_page("Delete Post", "post", &post.title, &cancel_url)
}

fn render_list_page(
    title: &str,
    add_url: &str,
    search: &str,
    action_url: &str,
    headers: &[&str],
    rows_html: &str,
) -> String {
    let th_class = "px-4 py-3 text-left text-sm font-medium text-gray-600";
    let no_sort: Vec<Option<&str>> = headers.iter().map(|_| None).collect();

    let add_label = format!("+ Add {}", title.trim_end_matches('s'));
    let heading = html! {
        h1.class("text-2xl font-semibold text-gray-900") { #title }
    };
    let add_btn = html! {
        a.href(#add_url).class("px-4 py-2 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 transition-colors duration-200") { #add_label }
    };

    let content = Element::<Div>::new()
        .child::<Div, _>(|d| {
            d.class("flex justify-between items-center mb-6")
                .raw(heading.render())
                .raw(add_btn.render())
        })
        .child::<Div, _>(|d| {
            d.class("bg-white rounded-lg shadow-sm border border-gray-200 mb-6")
                .child::<Div, _>(|d| {
                    d.class("p-4").child::<Form, _>(|f| {
                        f.class("flex gap-4").attr("method", "GET")
                            .child::<ironhtml_elements::Input, _>(|i| {
                                i.attr("type", "search").attr("name", "q")
                                    .class("flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500")
                                    .attr("placeholder", "Search...").attr("value", search)
                            })
                            .child::<ironhtml_elements::Button, _>(|b| {
                                b.attr("type", "submit")
                                    .class("px-4 py-2 border border-blue-600 text-blue-600 font-medium rounded-lg hover:bg-blue-50 transition-colors duration-200")
                                    .text("Filter")
                            })
                    })
                })
        })
        .raw(render_action_form_wrapper(action_url, headers, &no_sort, rows_html, th_class))
        .render();

    render_base(title, &content, true)
}

fn render_comment_list(comments: &[&Comment], search: &str) -> String {
    let edit_btn = "px-3 py-1 text-sm border border-blue-600 text-blue-600 rounded hover:bg-blue-50 transition-colors duration-200";
    let del_btn = "px-3 py-1 text-sm border border-red-600 text-red-600 rounded hover:bg-red-50 transition-colors duration-200";

    let mut rows = String::new();
    for c in comments {
        let id_str = c.id.to_string();
        let post_id_str = c.post_id.to_string();
        let edit_url = format!("/admin/comments/{}/change/", c.id);
        let del_url = format!("/admin/comments/{}/delete/", c.id);
        let checkbox = html! {
            input.type_("checkbox").name("selected").value(#&id_str).class("rounded border-gray-300")
        };
        let id_link = html! {
            a.href(#&edit_url).class("text-blue-600 hover:text-blue-800 font-medium") { #&id_str }
        };
        let edit_link = html! {
            a.href(#&edit_url).class(#edit_btn) { "Edit" }
        };
        let del_link = html! {
            a.href(#del_url).class(#del_btn) { "Delete" }
        };
        let checkbox_r = checkbox.render();
        let id_link_r = id_link.render();
        let edit_link_r = edit_link.render();
        let del_link_r = del_link.render();
        let author_ref = &c.author;
        let created_ref = &c.created_at;
        Element::<Tr>::new()
            .class("border-b border-gray-100 hover:bg-gray-50")
            .child::<Td, _>(|td| td.class("px-4 py-3").raw(&checkbox_r))
            .child::<Td, _>(|td| td.class("px-4 py-3").raw(&id_link_r))
            .child::<Td, _>(|td| td.class("px-4 py-3 text-gray-600").text(&post_id_str))
            .child::<Td, _>(|td| td.class("px-4 py-3").text(author_ref))
            .child::<Td, _>(|td| {
                td.class("px-4 py-3 text-gray-500 text-sm")
                    .text(created_ref)
            })
            .child::<Td, _>(|td| {
                td.class("px-4 py-3 space-x-2")
                    .raw(&edit_link_r)
                    .raw(&del_link_r)
            })
            .render_to(&mut rows);
    }

    render_list_page(
        "Comments",
        "/admin/comments/add/",
        search,
        "/admin/comments/action/",
        &["ID", "Post ID", "Author", "Created", "Actions"],
        &rows,
    )
}

fn render_comment_form(comment: Option<&Comment>, error: Option<&str>) -> String {
    let is_new = comment.is_none();
    let title = if is_new {
        "Add Comment"
    } else {
        "Edit Comment"
    };

    let (c_post_id, c_author, c_content) = comment
        .map(|c| (c.post_id.to_string(), c.author.clone(), c.content.clone()))
        .unwrap_or(("0".to_string(), String::new(), String::new()));

    let input_class = "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-200";
    let label_class = "block text-sm font-medium text-gray-700 mb-1";
    let error_html = render_error_alert(error);
    let delete_btn = comment.map(|c| {
        let href = format!("/admin/comments/{}/delete/", c.id);
        (html! {
            a.href(#href).class("ml-auto px-4 py-2 border border-red-600 text-red-600 font-medium rounded-lg hover:bg-red-50 transition-colors duration-200") { "Delete" }
        }).render()
    }).unwrap_or_default();

    let heading = html! {
        h1.class("text-2xl font-semibold text-gray-900 mb-6") { #title }
    };
    let section_header = html! {
        div.class("px-6 py-4 border-b border-gray-200") {
            h5.class("text-lg font-medium text-gray-900") { "Comment Details" }
        }
    };
    let post_id_label = html! {
        label.for_("post_id").class(#label_class) { "Post ID" }
    };
    let post_id_input = html! {
        input.type_("number").id("post_id").name("post_id").class(#input_class).value(#c_post_id)
    };
    let author_label = html! {
        label.for_("author").class(#label_class) { "Author *" }
    };
    let author_input = html! {
        input.type_("text").id("author").name("author").required.class(#input_class).value(#c_author)
    };
    let content_label = html! {
        label.for_("content").class(#label_class) { "Content" }
    };
    let content_textarea = html! {
        textarea.id("content").name("content").rows("6").class(#input_class) { #c_content }
    };
    let post_id_label_r = post_id_label.render();
    let post_id_input_r = post_id_input.render();
    let author_label_r = author_label.render();
    let author_input_r = author_input.render();
    let content_label_r = content_label.render();
    let content_textarea_r = content_textarea.render();

    let content = Element::<Div>::new()
        .raw(heading.render())
        .raw(&error_html)
        .child::<Form, _>(|f| {
            f.attr("method", "POST")
                .class("space-y-6")
                .child::<Div, _>(|d| {
                    d.class("bg-white rounded-lg shadow-sm border border-gray-200")
                        .raw(section_header.render())
                        .child::<Div, _>(|d| {
                            d.class("p-6 space-y-4")
                                .child::<Div, _>(|d| d.raw(&post_id_label_r).raw(&post_id_input_r))
                                .child::<Div, _>(|d| d.raw(&author_label_r).raw(&author_input_r))
                                .child::<Div, _>(|d| {
                                    d.raw(&content_label_r).raw(&content_textarea_r)
                                })
                        })
                })
                .child::<Div, _>(|d| d.raw(render_form_buttons("/admin/comments/", &delete_btn)))
        })
        .render();

    render_base(title, &content, true)
}

fn render_tag_list(tags: &[&Tag], search: &str) -> String {
    let edit_btn = "px-3 py-1 text-sm border border-blue-600 text-blue-600 rounded hover:bg-blue-50 transition-colors duration-200";
    let del_btn = "px-3 py-1 text-sm border border-red-600 text-red-600 rounded hover:bg-red-50 transition-colors duration-200";

    let mut rows = String::new();
    for t in tags {
        let id_str = t.id.to_string();
        let edit_url = format!("/admin/tags/{}/change/", t.id);
        let del_url = format!("/admin/tags/{}/delete/", t.id);
        let checkbox = html! {
            input.type_("checkbox").name("selected").value(#&id_str).class("rounded border-gray-300")
        };
        let name_ref = &t.name;
        let name_link = html! {
            a.href(#&edit_url).class("text-blue-600 hover:text-blue-800 font-medium") { #name_ref }
        };
        let edit_link = html! {
            a.href(#&edit_url).class(#edit_btn) { "Edit" }
        };
        let del_link = html! {
            a.href(#del_url).class(#del_btn) { "Delete" }
        };
        let checkbox_r = checkbox.render();
        let name_link_r = name_link.render();
        let edit_link_r = edit_link.render();
        let del_link_r = del_link.render();
        let slug_ref = &t.slug;
        Element::<Tr>::new()
            .class("border-b border-gray-100 hover:bg-gray-50")
            .child::<Td, _>(|td| td.class("px-4 py-3").raw(&checkbox_r))
            .child::<Td, _>(|td| td.class("px-4 py-3 text-gray-600").text(&id_str))
            .child::<Td, _>(|td| td.class("px-4 py-3").raw(&name_link_r))
            .child::<Td, _>(|td| td.class("px-4 py-3 text-gray-500").text(slug_ref))
            .child::<Td, _>(|td| {
                td.class("px-4 py-3 space-x-2")
                    .raw(&edit_link_r)
                    .raw(&del_link_r)
            })
            .render_to(&mut rows);
    }

    render_list_page(
        "Tags",
        "/admin/tags/add/",
        search,
        "/admin/tags/action/",
        &["ID", "Name", "Slug", "Actions"],
        &rows,
    )
}

fn render_tag_form(tag: Option<&Tag>, error: Option<&str>) -> String {
    let is_new = tag.is_none();
    let title = if is_new { "Add Tag" } else { "Edit Tag" };

    let (t_name, t_slug) = tag
        .map(|t| (t.name.as_str(), t.slug.as_str()))
        .unwrap_or(("", ""));

    let input_class = "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-200";
    let label_class = "block text-sm font-medium text-gray-700 mb-1";
    let error_html = render_error_alert(error);
    let delete_btn = tag.map(|t| {
        let href = format!("/admin/tags/{}/delete/", t.id);
        (html! {
            a.href(#href).class("ml-auto px-4 py-2 border border-red-600 text-red-600 font-medium rounded-lg hover:bg-red-50 transition-colors duration-200") { "Delete" }
        }).render()
    }).unwrap_or_default();

    let heading = html! {
        h1.class("text-2xl font-semibold text-gray-900 mb-6") { #title }
    };
    let section_header = html! {
        div.class("px-6 py-4 border-b border-gray-200") {
            h5.class("text-lg font-medium text-gray-900") { "Tag Details" }
        }
    };
    let name_label = html! {
        label.for_("name").class(#label_class) { "Name *" }
    };
    let name_input = html! {
        input.type_("text").id("name").name("name").required.class(#input_class).value(#t_name)
    };
    let slug_label = html! {
        label.for_("slug").class(#label_class) { "Slug" }
    };
    let slug_input = html! {
        input.type_("text").id("slug").name("slug").class(#input_class).value(#t_slug)
    };
    let name_label_r = name_label.render();
    let name_input_r = name_input.render();
    let slug_label_r = slug_label.render();
    let slug_input_r = slug_input.render();

    let content = Element::<Div>::new()
        .raw(heading.render())
        .raw(&error_html)
        .child::<Form, _>(|f| {
            f.attr("method", "POST")
                .class("space-y-6")
                .child::<Div, _>(|d| {
                    d.class("bg-white rounded-lg shadow-sm border border-gray-200")
                        .raw(section_header.render())
                        .child::<Div, _>(|d| {
                            d.class("p-6 space-y-4")
                                .child::<Div, _>(|d| d.raw(&name_label_r).raw(&name_input_r))
                                .child::<Div, _>(|d| d.raw(&slug_label_r).raw(&slug_input_r))
                        })
                })
                .child::<Div, _>(|d| d.raw(render_form_buttons("/admin/tags/", &delete_btn)))
        })
        .render();

    render_base(title, &content, true)
}

// ============================================================================
// Authentication helpers
// ============================================================================

async fn is_authenticated(req: &Request, state: &AppState) -> bool {
    let cookie = req.get_header("cookie").unwrap_or("");
    let session_id = cookie
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            if part.starts_with("session_id=") {
                Some(part.trim_start_matches("session_id="))
            } else {
                None
            }
        })
        .unwrap_or("");

    if session_id.is_empty() {
        return false;
    }

    let store = state.read().await;
    store.sessions.contains_key(session_id)
}

// ============================================================================
// HTTP Server Integration
// ============================================================================

fn build_router(state: AppState) -> Router {
    let s = state.clone();
    let s2 = state.clone();
    let s3 = state.clone();
    let s4 = state.clone();
    let s5 = state.clone();
    let s6 = state.clone();
    let s7 = state.clone();
    let s8 = state.clone();
    let s9 = state.clone();
    let s10 = state.clone();
    let s11 = state.clone();
    let s12 = state.clone();
    let s13 = state.clone();
    let s14 = state.clone();
    let s15 = state.clone();
    let s16 = state.clone();
    let s17 = state.clone();
    let s18 = state.clone();
    let s19 = state.clone();
    let s20 = state.clone();
    let s21 = state.clone();
    let s22 = state.clone();
    let s23 = state.clone();
    let s24 = state.clone();

    Router::new()
        // Auth routes
        .get("/admin/login", move |req| {
            let st = state.clone();
            async move { login_handler(req, st).await }
        })
        .post("/admin/login", move |req| {
            let st = s.clone();
            async move { login_handler(req, st).await }
        })
        .get("/admin/logout", move |req| {
            let st = s2.clone();
            async move { logout_handler(req, st).await }
        })
        // Dashboard
        .get("/admin/", move |req| {
            let st = s3.clone();
            async move { dashboard_handler(req, st).await }
        })
        // Posts CRUD
        .get("/admin/posts/", move |req| {
            let st = s4.clone();
            async move { list_posts_handler(req, st).await }
        })
        .post("/admin/posts/action/", move |req| {
            let st = s5.clone();
            async move { action_posts_handler(req, st).await }
        })
        .get("/admin/posts/add/", move |req| {
            let st = s6.clone();
            async move { add_post_handler(req, st).await }
        })
        .post("/admin/posts/add/", move |req| {
            let st = s7.clone();
            async move { add_post_handler(req, st).await }
        })
        .get("/admin/posts/{pk}/change/", move |req| {
            let st = s8.clone();
            async move { change_post_handler(req, st).await }
        })
        .post("/admin/posts/{pk}/change/", move |req| {
            let st = s9.clone();
            async move { change_post_handler(req, st).await }
        })
        .get("/admin/posts/{pk}/delete/", move |req| {
            let st = s10.clone();
            async move { delete_post_handler(req, st).await }
        })
        .post("/admin/posts/{pk}/delete/", move |req| {
            let st = s11.clone();
            async move { delete_post_handler(req, st).await }
        })
        // Comments CRUD
        .get("/admin/comments/", move |req| {
            let st = s12.clone();
            async move { list_comments_handler(req, st).await }
        })
        .post("/admin/comments/action/", move |req| {
            let st = s13.clone();
            async move { action_comments_handler(req, st).await }
        })
        .get("/admin/comments/add/", move |req| {
            let st = s14.clone();
            async move { add_comment_handler(req, st).await }
        })
        .post("/admin/comments/add/", move |req| {
            let st = s15.clone();
            async move { add_comment_handler(req, st).await }
        })
        .get("/admin/comments/{pk}/change/", move |req| {
            let st = s16.clone();
            async move { change_comment_handler(req, st).await }
        })
        .post("/admin/comments/{pk}/change/", move |req| {
            let st = s17.clone();
            async move { change_comment_handler(req, st).await }
        })
        .get("/admin/comments/{pk}/delete/", move |req| {
            let st = s18.clone();
            async move { delete_comment_handler(req, st).await }
        })
        .post("/admin/comments/{pk}/delete/", move |req| {
            let st = s19.clone();
            async move { delete_comment_handler(req, st).await }
        })
        // Tags CRUD
        .get("/admin/tags/", move |req| {
            let st = s20.clone();
            async move { list_tags_handler(req, st).await }
        })
        .post("/admin/tags/action/", move |req| {
            let st = s21.clone();
            async move { action_tags_handler(req, st).await }
        })
        .get("/admin/tags/add/", move |req| {
            let st = s22.clone();
            async move { add_tag_handler(req, st).await }
        })
        .post("/admin/tags/add/", move |req| {
            let st = s23.clone();
            async move { add_tag_handler(req, st).await }
        })
        .get("/admin/tags/{pk}/change/", {
            let st = s24.clone();
            move |req| {
                let st = st.clone();
                async move { change_tag_handler(req, st).await }
            }
        })
        .post("/admin/tags/{pk}/change/", {
            let st = s24.clone();
            move |req| {
                let st = st.clone();
                async move { change_tag_handler(req, st).await }
            }
        })
        .get("/admin/tags/{pk}/delete/", {
            let st = s24.clone();
            move |req| {
                let st = st.clone();
                async move { delete_tag_handler(req, st).await }
            }
        })
        .post("/admin/tags/{pk}/delete/", {
            let st = s24.clone();
            move |req| {
                let st = st.clone();
                async move { delete_tag_handler(req, st).await }
            }
        })
        // Redirect root to admin
        .get("/", |_| async { Response::redirect("/admin/") })
}

async fn handle_request(
    req: HyperRequest<hyper::body::Incoming>,
    router: Arc<Router>,
) -> Result<HyperResponse<Full<Bytes>>, Infallible> {
    use http_body_util::BodyExt;

    // Convert hyper request to oxide_router Request
    let method = Method::parse(req.method().as_str()).unwrap_or(Method::Get);
    let uri = req.uri();
    let path = uri.path().to_string();

    let mut oxide_req = Request::new(method, &path);

    // Parse query string
    if let Some(query) = uri.query() {
        oxide_req.query = Request::parse_query_string(query);
    }

    // Copy headers
    for (key, value) in req.headers() {
        if let Ok(v) = value.to_str() {
            oxide_req.headers.insert(key.to_string(), v.to_string());
        }
    }

    // Read body
    let body_bytes = req
        .collect()
        .await
        .map(|b| b.to_bytes())
        .unwrap_or_default();
    oxide_req.body = body_bytes.to_vec();

    // Handle request
    let oxide_res = router.handle(oxide_req).await;

    // Convert oxide_router Response to hyper Response
    let mut builder = HyperResponse::builder().status(
        StatusCode::from_u16(oxide_res.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
    );

    for (key, value) in &oxide_res.headers {
        builder = builder.header(key.as_str(), value.as_str());
    }

    let response = builder
        .body(Full::new(Bytes::from(oxide_res.body)))
        .unwrap();

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Print type-safe query examples first to demonstrate compile-time validation
    print_type_safe_query_examples();

    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    let state = Arc::new(RwLock::new(init_sample_data()));
    let router = Arc::new(build_router(state));

    let listener = TcpListener::bind(addr).await?;
    println!("Blog Admin running at http://{}", addr);
    println!("Login with: admin / admin123");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let router = router.clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                let router = router.clone();
                handle_request(req, router)
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
