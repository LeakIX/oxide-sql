//! Admin dashboard view.

use ironhtml::html;

use oxide_router::{Request, Response};

use crate::site::AdminSite;

use super::render_admin_page;

/// Renders the admin dashboard.
///
/// Shows a list of all registered models with links to their list views.
pub fn index_view(site: &AdminSite, _req: &Request, user_name: Option<String>) -> Response {
    let models = site.registered_models();

    let content = if models.is_empty() {
        html! {
            div.class("alert alert-info") {
                i.class("bi bi-info-circle me-2")
                "No models have been registered yet. Use "
                code { "AdminSite::register" }
                " to add models to the admin."
            }
        }
        .render()
    } else {
        let mut cards_html = String::new();
        for reg in &models {
            let list_url = site.list_url(&reg.slug);
            let add_url = site.add_url(&reg.slug);
            let manage_text = format!("Manage {} records", reg.verbose_name.to_lowercase());
            let vn_plural = &reg.verbose_name_plural;

            html! {
                div.class("col-md-4 col-lg-3 mb-4") {
                    div.class("card h-100") {
                        div.class("card-body") {
                            h5.class("card-title") {
                                i.class("bi bi-table me-2")
                                #vn_plural
                            }
                            p.class("card-text text-muted") {
                                #manage_text
                            }
                        }
                        div.class("card-footer bg-transparent") {
                            a.href(#list_url)
                                .class("btn btn-outline-primary btn-sm me-1") {
                                i.class("bi bi-list me-1")
                                "View"
                            }
                            a.href(#add_url)
                                .class("btn btn-primary btn-sm") {
                                i.class("bi bi-plus-lg me-1")
                                "Add"
                            }
                        }
                    }
                }
            }
            .render_to(&mut cards_html);
        }

        let mut result = String::new();
        html! { div.class("row") }
            .raw(&cards_html)
            .render_to(&mut result);
        html! {
            div.class("card mt-4") {
                div.class("card-header") {
                    i.class("bi bi-clock-history me-2")
                    "Recent Actions"
                }
                div.class("card-body") {
                    p.class("text-muted mb-0") {
                        "No recent actions."
                    }
                }
            }
        }
        .render_to(&mut result);
        result
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
