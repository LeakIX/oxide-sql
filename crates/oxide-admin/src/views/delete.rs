//! Admin delete confirmation view.

use ironhtml::typed::Element;
use ironhtml_elements::{Button, Div, Form, Li, Strong, Ul, A, I, P};

use oxide_router::Response;

use crate::site::{AdminSite, ModelRegistration};

use super::render_admin_page;

/// Data for the delete confirmation view.
pub struct DeleteViewData {
    /// String representation of the object.
    pub object_str: String,
    /// Primary key of the object.
    pub pk: String,
    /// Related objects that will also be deleted (model name, count).
    pub related_objects: Vec<(String, usize)>,
}

/// Renders the delete confirmation view.
pub fn delete_view(
    site: &AdminSite,
    registration: &ModelRegistration,
    data: DeleteViewData,
    user_name: Option<String>,
) -> Response {
    let related_html = if data.related_objects.is_empty() {
        String::new()
    } else {
        Element::<Div>::new()
            .class("alert alert-warning mt-3")
            .child::<Strong, _>(|s| {
                s.text(
                    "The following related objects \
                     will also be deleted:",
                )
            })
            .child::<Ul, _>(|ul| {
                ul.class("mb-0 mt-2").children(
                    data.related_objects.iter(),
                    |(model, count), li: Element<Li>| {
                        let text = format!("{} {} object(s)", count, model);
                        li.text(&text)
                    },
                )
            })
            .render()
    };

    let model_name = registration.verbose_name.to_lowercase();
    let list_url = site.list_url(&registration.slug);
    let delete_url = site.delete_url(&registration.slug, &data.pk);

    let content = Element::<Div>::new()
        .class("card")
        .child::<Div, _>(|d| {
            d.class("card-header bg-danger text-white")
                .child::<I, _>(|i| i.class("bi bi-exclamation-triangle me-2"))
                .text("Confirm Deletion")
        })
        .child::<Div, _>(|d| {
            d.class("card-body")
                .child::<P, _>(|p| {
                    p.class("lead")
                        .text(
                            "Are you sure you want to \
                               delete the ",
                        )
                        .text(&model_name)
                        .text(" \"")
                        .child::<Strong, _>(|s| s.text(&data.object_str))
                        .text("\"?")
                })
                .child::<P, _>(|p| p.class("text-muted").text("This action cannot be undone."))
                .raw(&related_html)
        })
        .child::<Div, _>(|d| {
            d.class(
                "card-footer d-flex \
                 justify-content-between",
            )
            .child::<A, _>(|a| {
                a.attr("href", &list_url)
                    .class("btn btn-outline-secondary")
                    .child::<I, _>(|i| i.class("bi bi-x-lg me-1"))
                    .text("No, take me back")
            })
            .child::<Form, _>(|f| {
                f.attr("method", "post")
                    .attr("action", &delete_url)
                    .child::<Button, _>(|b| {
                        b.attr("type", "submit")
                            .class("btn btn-danger")
                            .child::<I, _>(|i| i.class("bi bi-trash me-1"))
                            .text("Yes, I'm sure")
                    })
            })
        })
        .render();

    let breadcrumbs = vec![
        ("Home".to_string(), Some(format!("{}/", site.url_prefix))),
        (
            registration.verbose_name_plural.clone(),
            Some(site.list_url(&registration.slug)),
        ),
        (
            data.object_str.clone(),
            Some(site.change_url(&registration.slug, &data.pk)),
        ),
        ("Delete".to_string(), None),
    ];

    render_admin_page(
        site,
        &format!("Delete {}", registration.verbose_name.to_lowercase()),
        content,
        breadcrumbs,
        Vec::new(),
        user_name,
    )
}

/// Renders a success page after deletion.
pub fn delete_success_view(
    site: &AdminSite,
    registration: &ModelRegistration,
    object_str: &str,
    user_name: Option<String>,
) -> Response {
    let messages = vec![(
        "success".to_string(),
        format!(
            "The {} \"{}\" was deleted successfully.",
            registration.verbose_name.to_lowercase(),
            object_str
        ),
    )];

    let model_name = registration.verbose_name.to_lowercase();
    let list_url = site.list_url(&registration.slug);

    let content = Element::<Div>::new()
        .child::<P, _>(|p| {
            let text = format!("The {} was successfully deleted.", model_name);
            p.text(&text)
        })
        .child::<A, _>(|a| {
            a.attr("href", &list_url)
                .class("btn btn-primary")
                .child::<I, _>(|i| i.class("bi bi-arrow-left me-1"))
                .text("Back to list")
        })
        .render();

    let breadcrumbs = vec![
        ("Home".to_string(), Some(format!("{}/", site.url_prefix))),
        (
            registration.verbose_name_plural.clone(),
            Some(site.list_url(&registration.slug)),
        ),
        ("Deleted".to_string(), None),
    ];

    render_admin_page(
        site,
        "Deletion successful",
        content,
        breadcrumbs,
        messages,
        user_name,
    )
}

/// Result of a delete operation.
#[derive(Debug)]
pub enum DeleteResult {
    /// Deletion was successful.
    Success,
    /// Object was not found.
    NotFound,
    /// Deletion failed due to constraints or other errors.
    Failed(String),
}

impl DeleteResult {
    /// Creates a success result.
    pub fn success() -> Self {
        Self::Success
    }

    /// Creates a not found result.
    pub fn not_found() -> Self {
        Self::NotFound
    }

    /// Creates a failed result.
    pub fn failed(reason: impl Into<String>) -> Self {
        Self::Failed(reason.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::ModelAdmin;

    #[test]
    fn test_delete_result() {
        let success = DeleteResult::success();
        assert!(matches!(success, DeleteResult::Success));

        let not_found = DeleteResult::not_found();
        assert!(matches!(not_found, DeleteResult::NotFound));

        let failed = DeleteResult::failed("Foreign key constraint");
        match failed {
            DeleteResult::Failed(reason) => {
                assert!(reason.contains("constraint"));
            }
            _ => panic!("Expected Failed variant"),
        }
    }
}
