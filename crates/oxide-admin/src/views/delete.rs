//! Admin delete confirmation view.

use oxide_router::Response;

use crate::site::{AdminSite, ModelRegistration};
use crate::templates::html_escape;

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
        let items: Vec<String> = data
            .related_objects
            .iter()
            .map(|(model, count)| format!("<li>{} {} object(s)</li>", count, html_escape(model)))
            .collect();

        format!(
            r#"<div class="alert alert-warning mt-3">
                <strong>The following related objects will also be deleted:</strong>
                <ul class="mb-0 mt-2">{}</ul>
            </div>"#,
            items.join("\n")
        )
    };

    let content = format!(
        r#"<div class="card">
            <div class="card-header bg-danger text-white">
                <i class="bi bi-exclamation-triangle me-2"></i>
                Confirm Deletion
            </div>
            <div class="card-body">
                <p class="lead">
                    Are you sure you want to delete the {model_name}
                    "<strong>{object_str}</strong>"?
                </p>
                <p class="text-muted">
                    This action cannot be undone.
                </p>
                {related_html}
            </div>
            <div class="card-footer d-flex justify-content-between">
                <a href="{list_url}" class="btn btn-outline-secondary">
                    <i class="bi bi-x-lg me-1"></i>No, take me back
                </a>
                <form method="post" action="{delete_url}">
                    <button type="submit" class="btn btn-danger">
                        <i class="bi bi-trash me-1"></i>Yes, I'm sure
                    </button>
                </form>
            </div>
        </div>"#,
        model_name = html_escape(&registration.verbose_name.to_lowercase()),
        object_str = html_escape(&data.object_str),
        related_html = related_html,
        list_url = html_escape(&site.list_url(&registration.slug)),
        delete_url = html_escape(&site.delete_url(&registration.slug, &data.pk)),
    );

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

    let content = format!(
        r#"<p>
            The {model_name} was successfully deleted.
        </p>
        <a href="{list_url}" class="btn btn-primary">
            <i class="bi bi-arrow-left me-1"></i>Back to list
        </a>"#,
        model_name = html_escape(&registration.verbose_name.to_lowercase()),
        list_url = html_escape(&site.list_url(&registration.slug)),
    );

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
