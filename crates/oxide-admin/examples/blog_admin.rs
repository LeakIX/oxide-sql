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
            .header("X-Message", format!("Post created successfully"));
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

// ============================================================================
// Template Rendering
// ============================================================================

fn render_base(title: &str, content: &str, is_logged_in: bool) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en" class="h-full">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} | Blog Admin</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    colors: {{
                        primary: '#3b82f6',
                        muted: '#6b7280',
                        destructive: '#ef4444',
                    }}
                }}
            }}
        }}
    </script>
</head>
<body class="h-full bg-gray-50">
    <div class="flex h-full">
        {sidebar}
        <main class="flex-1 p-6 overflow-auto">
            {content}
        </main>
    </div>
</body>
</html>"#,
        title = title,
        sidebar = if is_logged_in {
            render_sidebar()
        } else {
            String::new()
        },
        content = content,
    )
}

fn render_sidebar() -> String {
    r#"<nav class="w-64 min-h-screen bg-gray-800 text-gray-300 flex-shrink-0">
        <div class="sticky top-0 p-4">
            <h2 class="text-white text-lg font-semibold mb-6">Blog Admin</h2>
            <ul class="space-y-2">
                <li>
                    <a href="/admin/"
                       class="block px-4 py-2 rounded-lg hover:bg-gray-700 hover:text-white
                              transition-colors duration-200">
                        Dashboard
                    </a>
                </li>
                <li>
                    <a href="/admin/posts/"
                       class="block px-4 py-2 rounded-lg hover:bg-gray-700 hover:text-white
                              transition-colors duration-200">
                        Posts
                    </a>
                </li>
                <li>
                    <a href="/admin/comments/"
                       class="block px-4 py-2 rounded-lg hover:bg-gray-700 hover:text-white
                              transition-colors duration-200">
                        Comments
                    </a>
                </li>
                <li>
                    <a href="/admin/tags/"
                       class="block px-4 py-2 rounded-lg hover:bg-gray-700 hover:text-white
                              transition-colors duration-200">
                        Tags
                    </a>
                </li>
            </ul>
            <hr class="my-6 border-gray-600">
            <ul>
                <li>
                    <a href="/admin/logout"
                       class="block px-4 py-2 rounded-lg text-red-400 hover:bg-gray-700
                              hover:text-red-300 transition-colors duration-200">
                        Logout
                    </a>
                </li>
            </ul>
        </div>
    </nav>"#
        .to_string()
}

fn render_login_page(error: Option<&str>) -> String {
    let error_html = error
        .map(|e| {
            format!(
                r#"<div class="mb-4 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg">
                    {}
                </div>"#,
                e
            )
        })
        .unwrap_or_default();

    let content = format!(
        r#"<div class="min-h-screen flex items-center justify-center">
            <div class="w-full max-w-md">
                <div class="bg-white rounded-lg shadow-sm border border-gray-200">
                    <div class="px-6 py-4 border-b border-gray-200">
                        <h4 class="text-xl font-semibold text-gray-900">Admin Login</h4>
                    </div>
                    <div class="p-6">
                        {error_html}
                        <form method="POST" class="space-y-4">
                            <div>
                                <label for="username"
                                       class="block text-sm font-medium text-gray-700 mb-1">
                                    Username
                                </label>
                                <input type="text" id="username" name="username" required
                                       class="w-full px-4 py-2 border border-gray-300 rounded-lg
                                              focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                                              transition-colors duration-200">
                            </div>
                            <div>
                                <label for="password"
                                       class="block text-sm font-medium text-gray-700 mb-1">
                                    Password
                                </label>
                                <input type="password" id="password" name="password" required
                                       class="w-full px-4 py-2 border border-gray-300 rounded-lg
                                              focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                                              transition-colors duration-200">
                            </div>
                            <button type="submit"
                                    class="w-full px-4 py-2 bg-blue-600 text-white font-medium
                                           rounded-lg hover:bg-blue-700 focus:ring-2
                                           focus:ring-blue-500 focus:ring-offset-2
                                           transition-colors duration-200">
                                Login
                            </button>
                        </form>
                        <p class="mt-4 text-sm text-gray-500">
                            Default: admin / admin123
                        </p>
                    </div>
                </div>
            </div>
        </div>"#,
        error_html = error_html,
    );

    render_base("Login", &content, false)
}

fn render_dashboard(store: &DataStore) -> String {
    let content = format!(
        r#"<h1 class="text-2xl font-semibold text-gray-900 mb-6">Dashboard</h1>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="bg-blue-600 text-white rounded-lg shadow-sm p-6">
                <h5 class="text-lg font-medium opacity-90">Posts</h5>
                <p class="text-4xl font-bold mt-2" data-model="posts">
                    <span class="count">{post_count}</span>
                </p>
                <a href="/admin/posts/"
                   class="inline-block mt-4 px-4 py-2 bg-white text-blue-600 text-sm
                          font-medium rounded-lg hover:bg-gray-100
                          transition-colors duration-200">
                    View all
                </a>
            </div>
            <div class="bg-green-600 text-white rounded-lg shadow-sm p-6">
                <h5 class="text-lg font-medium opacity-90">Comments</h5>
                <p class="text-4xl font-bold mt-2" data-model="comments">
                    <span class="count">{comment_count}</span>
                </p>
                <a href="/admin/comments/"
                   class="inline-block mt-4 px-4 py-2 bg-white text-green-600 text-sm
                          font-medium rounded-lg hover:bg-gray-100
                          transition-colors duration-200">
                    View all
                </a>
            </div>
            <div class="bg-cyan-600 text-white rounded-lg shadow-sm p-6">
                <h5 class="text-lg font-medium opacity-90">Tags</h5>
                <p class="text-4xl font-bold mt-2" data-model="tags">
                    <span class="count">{tag_count}</span>
                </p>
                <a href="/admin/tags/"
                   class="inline-block mt-4 px-4 py-2 bg-white text-cyan-600 text-sm
                          font-medium rounded-lg hover:bg-gray-100
                          transition-colors duration-200">
                    View all
                </a>
            </div>
        </div>"#,
        post_count = store.posts.len(),
        comment_count = store.comments.len(),
        tag_count = store.tags.len(),
    );

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

    let rows: String = page_posts
        .iter()
        .map(|p| {
            let badge_class = if p.status == "published" {
                "bg-green-100 text-green-800"
            } else {
                "bg-gray-100 text-gray-800"
            };
            format!(
                r#"<tr class="border-b border-gray-100 hover:bg-gray-50">
                    <td class="px-4 py-3">
                        <input type="checkbox" name="selected" value="{}"
                               class="rounded border-gray-300">
                    </td>
                    <td class="px-4 py-3 text-gray-600">{}</td>
                    <td class="px-4 py-3">
                        <a href="/admin/posts/{}/change/"
                           class="text-blue-600 hover:text-blue-800 font-medium">{}</a>
                    </td>
                    <td class="px-4 py-3">
                        <span class="px-2 py-1 text-xs font-medium rounded-full {}">{}</span>
                    </td>
                    <td class="px-4 py-3 text-gray-500 text-sm">{}</td>
                    <td class="px-4 py-3 space-x-2">
                        <a href="/admin/posts/{}/change/"
                           class="px-3 py-1 text-sm border border-blue-600 text-blue-600
                                  rounded hover:bg-blue-50 transition-colors duration-200">
                            Edit
                        </a>
                        <a href="/admin/posts/{}/delete/"
                           class="px-3 py-1 text-sm border border-red-600 text-red-600
                                  rounded hover:bg-red-50 transition-colors duration-200">
                            Delete
                        </a>
                    </td>
                </tr>"#,
                p.id,
                p.id,
                p.id,
                html_escape(&p.title),
                badge_class,
                p.status,
                p.created_at,
                p.id,
                p.id,
            )
        })
        .collect();

    let pagination = if total_pages > 1 {
        let mut pag =
            String::from(r#"<nav class="mt-6 flex justify-center"><div class="flex gap-1">"#);
        for i in 1..=total_pages {
            let active_class = if i == page {
                "bg-blue-600 text-white"
            } else {
                "bg-white text-gray-700 hover:bg-gray-50"
            };
            pag.push_str(&format!(
                r#"<a href="?page={}"
                   class="px-4 py-2 text-sm font-medium border border-gray-300 rounded-lg {}">{}</a>"#,
                i, active_class, i
            ));
        }
        pag.push_str("</div></nav>");
        pag
    } else {
        String::new()
    };

    let status_selected = |s: &str| {
        if status_filter == Some(s) {
            "selected"
        } else {
            ""
        }
    };

    let content = format!(
        r#"<div class="flex justify-between items-center mb-6">
            <h1 class="text-2xl font-semibold text-gray-900 page-title">Posts</h1>
            <a href="/admin/posts/add/"
               class="px-4 py-2 bg-blue-600 text-white font-medium rounded-lg
                      hover:bg-blue-700 transition-colors duration-200">
                + Add Post
            </a>
        </div>

        <div class="bg-white rounded-lg shadow-sm border border-gray-200 mb-6">
            <div class="p-4">
                <form class="flex gap-4" method="GET">
                    <input type="search" name="q"
                           class="flex-1 px-4 py-2 border border-gray-300 rounded-lg
                                  focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                           placeholder="Search..." value="{search}">
                    <select name="status"
                            class="px-4 py-2 border border-gray-300 rounded-lg
                                   focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                                   filter-select">
                        <option value="">All statuses</option>
                        <option value="published" {pub_sel}>Published</option>
                        <option value="draft" {draft_sel}>Draft</option>
                    </select>
                    <button type="submit"
                            class="px-4 py-2 border border-blue-600 text-blue-600 font-medium
                                   rounded-lg hover:bg-blue-50 transition-colors duration-200">
                        Filter
                    </button>
                </form>
            </div>
        </div>

        <form method="POST" action="/admin/posts/action/">
            <div class="bg-white rounded-lg shadow-sm border border-gray-200">
                <div class="px-4 py-3 border-b border-gray-200 flex items-center gap-4">
                    <select name="action"
                            class="px-3 py-2 border border-gray-300 rounded-lg text-sm
                                   focus:ring-2 focus:ring-blue-500 action-select">
                        <option value="">-- Select action --</option>
                        <option value="delete_selected">Delete selected</option>
                    </select>
                    <button type="submit" name="apply_action"
                            class="px-4 py-2 border border-gray-300 text-gray-700 text-sm
                                   font-medium rounded-lg hover:bg-gray-50
                                   transition-colors duration-200">
                        Apply
                    </button>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full list-view">
                        <thead class="bg-gray-50 border-b border-gray-200">
                            <tr>
                                <th class="px-4 py-3 text-left w-10">
                                    <input type="checkbox" id="select-all"
                                           class="rounded border-gray-300">
                                </th>
                                <th class="px-4 py-3 text-left text-sm font-medium text-gray-600"
                                    data-sortable>
                                    <a href="?order=id" class="hover:text-gray-900">ID</a>
                                </th>
                                <th class="px-4 py-3 text-left text-sm font-medium text-gray-600"
                                    data-sortable>
                                    <a href="?order=title" class="hover:text-gray-900">Title</a>
                                </th>
                                <th class="px-4 py-3 text-left text-sm font-medium text-gray-600"
                                    data-sortable>
                                    <a href="?order=status" class="hover:text-gray-900">Status</a>
                                </th>
                                <th class="px-4 py-3 text-left text-sm font-medium text-gray-600"
                                    data-sortable>
                                    <a href="?order=created_at" class="hover:text-gray-900">
                                        Created
                                    </a>
                                </th>
                                <th class="px-4 py-3 text-left text-sm font-medium text-gray-600">
                                    Actions
                                </th>
                            </tr>
                        </thead>
                        <tbody>{rows}</tbody>
                    </table>
                </div>
            </div>
        </form>

        {pagination}"#,
        search = html_escape(search),
        pub_sel = status_selected("published"),
        draft_sel = status_selected("draft"),
        rows = rows,
        pagination = pagination,
    );

    render_base("Posts", &content, true)
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

    let error_html = error
        .map(|e| {
            format!(
                r#"<div class="mb-6 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg">
                    {}
                </div>"#,
                e
            )
        })
        .unwrap_or_default();

    let draft_sel = if post_status == "draft" {
        "selected"
    } else {
        ""
    };
    let pub_sel = if post_status == "published" {
        "selected"
    } else {
        ""
    };
    let delete_btn = if let Some(p) = post {
        format!(
            r#"<a href="/admin/posts/{}/delete/"
               class="ml-auto px-4 py-2 border border-red-600 text-red-600 font-medium
                      rounded-lg hover:bg-red-50 transition-colors duration-200">
                Delete
            </a>"#,
            p.id
        )
    } else {
        String::new()
    };

    let input_class = "w-full px-4 py-2 border border-gray-300 rounded-lg \
                       focus:ring-2 focus:ring-blue-500 focus:border-blue-500 \
                       transition-colors duration-200";

    let content = format!(
        r##"<h1 class="text-2xl font-semibold text-gray-900 mb-6">{title}</h1>

        {error_html}

        <form method="POST" class="space-y-6">
            <div class="bg-white rounded-lg shadow-sm border border-gray-200">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h5 class="text-lg font-medium text-gray-900">Content</h5>
                </div>
                <div class="p-6 space-y-4">
                    <div>
                        <label for="title"
                               class="block text-sm font-medium text-gray-700 mb-1">
                            Title *
                        </label>
                        <input type="text" id="title" name="title" required
                               class="{input_class}" value="{post_title}">
                    </div>
                    <div>
                        <label for="slug"
                               class="block text-sm font-medium text-gray-700 mb-1">
                            Slug
                        </label>
                        <input type="text" id="slug" name="slug"
                               class="{input_class}" value="{post_slug}">
                    </div>
                    <div>
                        <label for="content"
                               class="block text-sm font-medium text-gray-700 mb-1">
                            Content
                        </label>
                        <textarea id="content" name="content" rows="10"
                                  class="{input_class}">{post_content}</textarea>
                    </div>
                </div>
            </div>

            <div class="bg-white rounded-lg shadow-sm border border-gray-200">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h5 class="text-lg font-medium text-gray-900">Publishing</h5>
                </div>
                <div class="p-6">
                    <div>
                        <label for="status"
                               class="block text-sm font-medium text-gray-700 mb-1">
                            Status
                        </label>
                        <select id="status" name="status" class="{input_class}">
                            <option value="draft" {draft_sel}>Draft</option>
                            <option value="published" {pub_sel}>Published</option>
                        </select>
                    </div>
                </div>
            </div>

            <div class="flex gap-4">
                <button type="submit"
                        class="px-6 py-2 bg-blue-600 text-white font-medium rounded-lg
                               hover:bg-blue-700 focus:ring-2 focus:ring-blue-500
                               focus:ring-offset-2 transition-colors duration-200">
                    Save
                </button>
                <a href="/admin/posts/"
                   class="px-6 py-2 border border-gray-300 text-gray-700 font-medium
                          rounded-lg hover:bg-gray-50 transition-colors duration-200">
                    Cancel
                </a>
                {delete_btn}
            </div>
        </form>"##,
        title = title,
        error_html = error_html,
        input_class = input_class,
        post_title = html_escape(post_title),
        post_slug = html_escape(post_slug),
        post_content = html_escape(post_content),
        draft_sel = draft_sel,
        pub_sel = pub_sel,
        delete_btn = delete_btn,
    );

    render_base(title, &content, true)
}

fn render_delete_confirmation(post: &Post) -> String {
    let content = format!(
        r#"<h1 class="text-2xl font-semibold text-gray-900 mb-6">Delete Post</h1>

        <div class="delete-confirmation max-w-2xl">
            <div class="bg-amber-50 border border-amber-200 rounded-lg p-6 mb-6">
                <h4 class="text-lg font-semibold text-amber-800 mb-2">Are you sure?</h4>
                <p class="text-amber-700 mb-2">
                    You are about to delete the post:
                    <strong class="font-semibold">{}</strong>
                </p>
                <p class="text-amber-700">This action cannot be undone.</p>
            </div>

            <form method="POST" class="flex gap-4">
                <button type="submit"
                        class="px-6 py-2 bg-red-600 text-white font-medium rounded-lg
                               hover:bg-red-700 focus:ring-2 focus:ring-red-500
                               focus:ring-offset-2 transition-colors duration-200">
                    Confirm Delete
                </button>
                <a href="/admin/posts/{}/change/"
                   class="px-6 py-2 border border-gray-300 text-gray-700 font-medium
                          rounded-lg hover:bg-gray-50 transition-colors duration-200">
                    Cancel
                </a>
            </form>
        </div>"#,
        html_escape(&post.title),
        post.id,
    );

    render_base("Delete Post", &content, true)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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
    let state_clone = state.clone();
    let state_clone2 = state.clone();
    let state_clone3 = state.clone();
    let state_clone4 = state.clone();
    let state_clone5 = state.clone();
    let state_clone6 = state.clone();

    Router::new()
        // Auth routes
        .get("/admin/login", move |req| {
            let s = state.clone();
            async move { login_handler(req, s).await }
        })
        .post("/admin/login", move |req| {
            let s = state_clone.clone();
            async move { login_handler(req, s).await }
        })
        .get("/admin/logout", move |req| {
            let s = state_clone2.clone();
            async move { logout_handler(req, s).await }
        })
        // Dashboard
        .get("/admin/", move |req| {
            let s = state_clone3.clone();
            async move { dashboard_handler(req, s).await }
        })
        // Posts CRUD
        .get("/admin/posts/", move |req| {
            let s = state_clone4.clone();
            async move { list_posts_handler(req, s).await }
        })
        .get("/admin/posts/add/", {
            let s = state_clone5.clone();
            move |req| {
                let s = s.clone();
                async move { add_post_handler(req, s).await }
            }
        })
        .post("/admin/posts/add/", {
            let s = state_clone5.clone();
            move |req| {
                let s = s.clone();
                async move { add_post_handler(req, s).await }
            }
        })
        .get("/admin/posts/{pk}/change/", {
            let s = state_clone5.clone();
            move |req| {
                let s = s.clone();
                async move { change_post_handler(req, s).await }
            }
        })
        .post("/admin/posts/{pk}/change/", {
            let s = state_clone5.clone();
            move |req| {
                let s = s.clone();
                async move { change_post_handler(req, s).await }
            }
        })
        .get("/admin/posts/{pk}/delete/", {
            let s = state_clone6.clone();
            move |req| {
                let s = s.clone();
                async move { delete_post_handler(req, s).await }
            }
        })
        .post("/admin/posts/{pk}/delete/", {
            let s = state_clone6.clone();
            move |req| {
                let s = s.clone();
                async move { delete_post_handler(req, s).await }
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
