//! Admin list/changelist view.

use std::collections::HashMap;

use oxide_router::{Request, Response};

use crate::site::{AdminSite, ModelRegistration};
use crate::templates::list::{render_list_view, ListFilter, ListRow, ListViewContext};

use super::render_admin_page;

/// Parameters for the list view.
#[derive(Debug, Clone, Default)]
pub struct ListViewParams {
    /// Current page number (1-indexed).
    pub page: usize,
    /// Search query.
    pub search: Option<String>,
    /// Active filters (field -> value).
    pub filters: HashMap<String, String>,
    /// Sort column (prefix with - for descending).
    pub ordering: Option<String>,
}

impl ListViewParams {
    /// Parses list view parameters from a request.
    pub fn from_request(req: &Request) -> Self {
        let page = req
            .query
            .get("page")
            .and_then(|p| p.parse().ok())
            .unwrap_or(1)
            .max(1);

        let search = req.query.get("q").cloned().filter(|s| !s.is_empty());

        let ordering = req.query.get("o").cloned().filter(|s| !s.is_empty());

        // All other query params are treated as filters
        let filters: HashMap<String, String> = req
            .query
            .iter()
            .filter(|(k, _)| k.as_str() != "page" && k.as_str() != "q" && k.as_str() != "o")
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Self {
            page,
            search,
            filters,
            ordering,
        }
    }

    /// Builds a query string from parameters.
    pub fn to_query_string(&self) -> String {
        let mut parts = Vec::new();

        if self.page > 1 {
            parts.push(format!("page={}", self.page));
        }
        if let Some(ref q) = self.search {
            parts.push(format!("q={}", urlencoding_simple(q)));
        }
        if let Some(ref o) = self.ordering {
            parts.push(format!("o={}", urlencoding_simple(o)));
        }
        for (k, v) in &self.filters {
            parts.push(format!(
                "{}={}",
                urlencoding_simple(k),
                urlencoding_simple(v)
            ));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }
}

/// Data needed to render the list view.
pub struct ListViewData {
    /// Column headers.
    pub columns: Vec<String>,
    /// Row data.
    pub rows: Vec<ListRow>,
    /// Total count of items (before pagination).
    pub total_count: usize,
    /// Available filters.
    pub available_filters: Vec<ListFilter>,
}

/// Renders the list view for a model.
///
/// This function is called by the admin site after fetching the data.
/// The actual data fetching is done by the caller using the ORM.
pub fn list_view(
    site: &AdminSite,
    registration: &ModelRegistration,
    params: &ListViewParams,
    data: ListViewData,
    user_name: Option<String>,
) -> Response {
    let per_page = registration.admin.list_per_page;
    let total_pages = (data.total_count + per_page - 1) / per_page;

    let active_filters: Vec<(String, String)> = params
        .filters
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let actions: Vec<(String, String)> =
        vec![("delete_selected".to_string(), "Delete selected".to_string())];

    let ctx = ListViewContext {
        model_name: registration.name.clone(),
        model_verbose_name: registration.verbose_name_plural.clone(),
        columns: data.columns,
        rows: data.rows,
        add_url: site.add_url(&registration.slug),
        search_query: params.search.clone(),
        search_fields: registration.admin.search_fields.clone(),
        filters: data.available_filters,
        active_filters,
        actions,
        page: params.page,
        total_pages,
        total_items: data.total_count,
        per_page,
    };

    let content = render_list_view(&ctx);

    let breadcrumbs = vec![
        ("Home".to_string(), Some(format!("{}/", site.url_prefix))),
        (registration.verbose_name_plural.clone(), None),
    ];

    render_admin_page(
        site,
        &format!(
            "Select {} to change",
            registration.verbose_name.to_lowercase()
        ),
        content,
        breadcrumbs,
        Vec::new(),
        user_name,
    )
}

/// Builds SQL WHERE clauses for search.
pub fn build_search_clause(search_fields: &[String], query: &str) -> Option<(String, Vec<String>)> {
    if search_fields.is_empty() || query.is_empty() {
        return None;
    }

    let conditions: Vec<String> = search_fields
        .iter()
        .map(|field| format!("{} LIKE ?", field))
        .collect();

    let search_pattern = format!("%{}%", query);
    let params: Vec<String> = search_fields
        .iter()
        .map(|_| search_pattern.clone())
        .collect();

    Some((format!("({})", conditions.join(" OR ")), params))
}

/// Builds SQL ORDER BY clause.
pub fn build_order_clause(ordering: &[String]) -> String {
    if ordering.is_empty() {
        return String::new();
    }

    let parts: Vec<String> = ordering
        .iter()
        .map(|col| {
            if let Some(stripped) = col.strip_prefix('-') {
                format!("{} DESC", stripped)
            } else {
                format!("{} ASC", col)
            }
        })
        .collect();

    format!("ORDER BY {}", parts.join(", "))
}

/// Simple URL encoding.
fn urlencoding_simple(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_view_params_parsing() {
        let mut req = Request::get("/admin/user/");
        req.query.insert("page".to_string(), "2".to_string());
        req.query.insert("q".to_string(), "john".to_string());
        req.query.insert("status".to_string(), "active".to_string());

        let params = ListViewParams::from_request(&req);
        assert_eq!(params.page, 2);
        assert_eq!(params.search, Some("john".to_string()));
        assert_eq!(params.filters.get("status"), Some(&"active".to_string()));
    }

    #[test]
    fn test_build_search_clause() {
        let fields = vec!["title".to_string(), "content".to_string()];
        let (clause, params) = build_search_clause(&fields, "hello").unwrap();

        assert!(clause.contains("title LIKE ?"));
        assert!(clause.contains("content LIKE ?"));
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "%hello%");
    }

    #[test]
    fn test_build_order_clause() {
        let ordering = vec!["-created_at".to_string(), "title".to_string()];
        let clause = build_order_clause(&ordering);

        assert_eq!(clause, "ORDER BY created_at DESC, title ASC");
    }

    #[test]
    fn test_build_order_clause_empty() {
        let ordering: Vec<String> = Vec::new();
        let clause = build_order_clause(&ordering);

        assert_eq!(clause, "");
    }
}
