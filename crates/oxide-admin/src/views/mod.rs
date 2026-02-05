//! Admin view handlers.
//!
//! This module contains the HTTP handlers for admin views:
//! - `index` - Dashboard showing registered models
//! - `list` - Changelist view with pagination, search, and filters
//! - `detail` - Add/change form view
//! - `delete` - Delete confirmation view

mod delete;
mod detail;
mod index;
mod list;

pub use delete::{delete_success_view, delete_view, DeleteResult, DeleteViewData};
pub use detail::{
    add_view, change_view, get_redirect_url, AddViewData, ChangeViewData, FieldsetData, FormResult,
    InlineData, InlineRowData,
};
pub use index::index_view;
pub use list::{build_order_clause, build_search_clause, list_view, ListViewData, ListViewParams};

use oxide_router::Response;

use crate::site::AdminSite;
use crate::templates::{render_base, AdminContext};

/// Helper to render a page with the base admin layout.
pub fn render_admin_page(
    site: &AdminSite,
    page_title: &str,
    content: String,
    breadcrumbs: Vec<(String, Option<String>)>,
    messages: Vec<(String, String)>,
    user_name: Option<String>,
) -> Response {
    let ctx = AdminContext {
        site_title: site.name.clone(),
        site_header: site.name.clone(),
        user_name,
        models: site.model_list(),
        breadcrumbs,
        page_title: page_title.to_string(),
        content,
        messages,
    };

    Response::html(render_base(&ctx))
}
