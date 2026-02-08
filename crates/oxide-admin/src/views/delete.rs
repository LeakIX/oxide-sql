//! Admin delete confirmation view.

use ironhtml::html;
use ironhtml::typed::Element;
use ironhtml_elements::{Li, Ul};

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
        html! {
            div.class("alert alert-warning mt-3") {
                strong {
                    "The following related objects will also be deleted:"
                }
            }
        }
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
    let obj_str = &data.object_str;

    let confirm_text = format!("Are you sure you want to delete the {} \"", model_name);

    let cancel_btn = html! {
        a.href(#list_url).class("btn btn-outline-secondary") {
            i.class("bi bi-x-lg me-1")
            "No, take me back"
        }
    };

    let submit_btn = html! {
        button.type_("submit").class("btn btn-danger") {
            i.class("bi bi-trash me-1")
            "Yes, I'm sure"
        }
    };

    let content = html! {
        div.class("card") {
            div.class("card-header bg-danger text-white") {
                i.class("bi bi-exclamation-triangle me-2")
                "Confirm Deletion"
            }
            div.class("card-body") {
                p.class("lead") {
                    #confirm_text
                    strong { #obj_str }
                    "\"?"
                }
                p.class("text-muted") {
                    "This action cannot be undone."
                }
            }
        }
    }
    .child::<ironhtml_elements::Div, _>(|d| d.class("card-body").raw(&related_html))
    .child::<ironhtml_elements::Div, _>(|d| {
        d.class("card-footer d-flex justify-content-between")
            .raw(cancel_btn.render())
            .child::<ironhtml_elements::Form, _>(|f| {
                f.attr("method", "post")
                    .attr("action", &delete_url)
                    .child::<ironhtml_elements::Div, _>(|d| d.raw(submit_btn.render()))
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
    let deleted_text = format!("The {} was successfully deleted.", model_name);

    let content = html! {
        div {
            p { #deleted_text }
            a.href(#list_url).class("btn btn-primary") {
                i.class("bi bi-arrow-left me-1")
                "Back to list"
            }
        }
    }
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
