//! Admin dashboard view.

use ironhtml::typed::Element;
use ironhtml_elements::{Code, Div, A, H5, I, P};

use oxide_router::{Request, Response};

use crate::site::AdminSite;

use super::render_admin_page;

/// Renders the admin dashboard.
///
/// Shows a list of all registered models with links to their list views.
pub fn index_view(site: &AdminSite, _req: &Request, user_name: Option<String>) -> Response {
    let models = site.registered_models();

    let content = if models.is_empty() {
        Element::<Div>::new()
            .class("alert alert-info")
            .child::<I, _>(|i| i.class("bi bi-info-circle me-2"))
            .text("No models have been registered yet. Use ")
            .child::<Code, _>(|c| c.text("AdminSite::register"))
            .text(" to add models to the admin.")
            .render()
    } else {
        let mut cards_html = String::new();
        for reg in &models {
            let list_url = site.list_url(&reg.slug);
            let add_url = site.add_url(&reg.slug);
            let verbose_lower = reg.verbose_name.to_lowercase();
            let manage_text = format!("Manage {} records", verbose_lower);

            Element::<Div>::new()
                .class("col-md-4 col-lg-3 mb-4")
                .child::<Div, _>(|card| {
                    card.class("card h-100")
                        .child::<Div, _>(|cb| {
                            cb.class("card-body")
                                .child::<H5, _>(|h| {
                                    h.class("card-title")
                                        .child::<I, _>(|i| {
                                            i.class(
                                                "bi bi-table \
                                                 me-2",
                                            )
                                        })
                                        .text(&reg.verbose_name_plural)
                                })
                                .child::<P, _>(|p| {
                                    p.class("card-text text-muted").text(&manage_text)
                                })
                        })
                        .child::<Div, _>(|cf| {
                            cf.class("card-footer bg-transparent")
                                .child::<A, _>(|a| {
                                    a.attr("href", &list_url)
                                        .class(
                                            "btn \
                                         btn-outline-primary \
                                         btn-sm me-1",
                                        )
                                        .child::<I, _>(|i| i.class("bi bi-list me-1"))
                                        .text("View")
                                })
                                .child::<A, _>(|a| {
                                    a.attr("href", &add_url)
                                        .class(
                                            "btn btn-primary \
                                         btn-sm",
                                        )
                                        .child::<I, _>(|i| {
                                            i.class(
                                                "bi bi-plus-lg \
                                             me-1",
                                            )
                                        })
                                        .text("Add")
                                })
                        })
                })
                .render_to(&mut cards_html);
        }

        let mut html = String::new();
        Element::<Div>::new()
            .class("row")
            .raw(&cards_html)
            .render_to(&mut html);
        Element::<Div>::new()
            .class("card mt-4")
            .child::<Div, _>(|d| {
                d.class("card-header")
                    .child::<I, _>(|i| i.class("bi bi-clock-history me-2"))
                    .text("Recent Actions")
            })
            .child::<Div, _>(|d| {
                d.class("card-body")
                    .child::<P, _>(|p| p.class("text-muted mb-0").text("No recent actions."))
            })
            .render_to(&mut html);
        html
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
