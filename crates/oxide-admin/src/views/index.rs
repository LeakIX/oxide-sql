//! Admin dashboard view.

use oxide_router::{Request, Response};

use crate::site::AdminSite;
use crate::templates::html_escape;

use super::render_admin_page;

/// Renders the admin dashboard.
///
/// Shows a list of all registered models with links to their list views.
pub fn index_view(site: &AdminSite, _req: &Request, user_name: Option<String>) -> Response {
    let models = site.registered_models();

    let model_cards: Vec<String> = models
        .iter()
        .map(|reg| {
            let list_url = site.list_url(&reg.slug);
            let add_url = site.add_url(&reg.slug);

            format!(
                r#"<div class="col-md-4 col-lg-3 mb-4">
                    <div class="card h-100">
                        <div class="card-body">
                            <h5 class="card-title">
                                <i class="bi bi-table me-2"></i>{verbose_name_plural}
                            </h5>
                            <p class="card-text text-muted">
                                Manage {verbose_name} records
                            </p>
                        </div>
                        <div class="card-footer bg-transparent">
                            <a href="{list_url}" class="btn btn-outline-primary btn-sm me-1">
                                <i class="bi bi-list me-1"></i>View
                            </a>
                            <a href="{add_url}" class="btn btn-primary btn-sm">
                                <i class="bi bi-plus-lg me-1"></i>Add
                            </a>
                        </div>
                    </div>
                </div>"#,
                verbose_name_plural = html_escape(&reg.verbose_name_plural),
                verbose_name = html_escape(&reg.verbose_name.to_lowercase()),
                list_url = html_escape(&list_url),
                add_url = html_escape(&add_url),
            )
        })
        .collect();

    let content = if model_cards.is_empty() {
        r#"<div class="alert alert-info">
            <i class="bi bi-info-circle me-2"></i>
            No models have been registered yet.
            Use <code>AdminSite::register</code> to add models to the admin.
        </div>"#
            .to_string()
    } else {
        format!(
            r#"<div class="row">
                {cards}
            </div>

            <div class="card mt-4">
                <div class="card-header">
                    <i class="bi bi-clock-history me-2"></i>Recent Actions
                </div>
                <div class="card-body">
                    <p class="text-muted mb-0">No recent actions.</p>
                </div>
            </div>"#,
            cards = model_cards.join("\n")
        )
    };

    let breadcrumbs = vec![("Home".to_string(), None)];

    render_admin_page(
        site,
        "Dashboard",
        content,
        breadcrumbs,
        Vec::new(),
        user_name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_router::Request;

    #[test]
    fn test_index_view_empty() {
        let site = AdminSite::new("Test Admin");
        let req = Request::get("/admin/");

        let response = index_view(&site, &req, None);
        assert_eq!(response.status, 200);

        let body = response.body_string().unwrap();
        assert!(body.contains("No models have been registered"));
    }
}
