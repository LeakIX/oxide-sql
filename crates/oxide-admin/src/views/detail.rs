//! Admin detail view (add/change).

use oxide_router::Response;

use crate::site::{AdminSite, ModelRegistration};
use crate::templates::detail::{
    render_detail_view, DetailViewContext, Fieldset, InlineFormset, InlineRow,
};

use super::render_admin_page;

/// Data for rendering the add view.
pub struct AddViewData {
    /// Rendered form fields HTML.
    pub form_html: String,
    /// Fieldsets with their field HTML.
    pub fieldsets: Vec<FieldsetData>,
    /// Inline formsets for related models.
    pub inlines: Vec<InlineData>,
}

/// Data for a fieldset.
pub struct FieldsetData {
    /// Fieldset name/title.
    pub name: Option<String>,
    /// Field HTML for each field.
    pub fields: Vec<String>,
    /// CSS classes.
    pub classes: Vec<String>,
    /// Description.
    pub description: Option<String>,
}

impl From<FieldsetData> for Fieldset {
    fn from(data: FieldsetData) -> Self {
        Self {
            name: data.name,
            fields: data.fields,
            classes: data.classes,
            description: data.description,
        }
    }
}

/// Data for an inline formset.
pub struct InlineData {
    /// Model name.
    pub model_name: String,
    /// Verbose name plural.
    pub verbose_name: String,
    /// Column headers.
    pub columns: Vec<String>,
    /// Existing rows.
    pub rows: Vec<InlineRowData>,
    /// Number of extra empty forms.
    pub extra: usize,
    /// Empty form template HTML.
    pub empty_form: String,
}

/// Data for an inline row.
pub struct InlineRowData {
    /// Row ID/pk.
    pub id: String,
    /// Field HTML.
    pub fields: Vec<String>,
    /// Delete checkbox HTML.
    pub delete_checkbox: String,
}

impl From<InlineRowData> for InlineRow {
    fn from(data: InlineRowData) -> Self {
        Self {
            id: data.id,
            fields: data.fields,
            delete_checkbox: data.delete_checkbox,
        }
    }
}

impl From<InlineData> for InlineFormset {
    fn from(data: InlineData) -> Self {
        Self {
            model_name: data.model_name,
            verbose_name: data.verbose_name,
            columns: data.columns,
            rows: data.rows.into_iter().map(Into::into).collect(),
            extra: data.extra,
            empty_form: data.empty_form,
        }
    }
}

/// Renders the add view for a model.
pub fn add_view(
    site: &AdminSite,
    registration: &ModelRegistration,
    data: AddViewData,
    errors: Vec<String>,
    messages: Vec<(String, String)>,
    user_name: Option<String>,
) -> Response {
    let ctx = DetailViewContext {
        model_name: registration.name.clone(),
        is_new: true,
        object_str: None,
        form_html: data.form_html,
        fieldsets: data.fieldsets.into_iter().map(Into::into).collect(),
        inlines: data.inlines.into_iter().map(Into::into).collect(),
        list_url: site.list_url(&registration.slug),
        delete_url: None,
        action_url: site.add_url(&registration.slug),
        errors,
    };

    let content = render_detail_view(&ctx);

    let breadcrumbs = vec![
        ("Home".to_string(), Some(format!("{}/", site.url_prefix))),
        (
            registration.verbose_name_plural.clone(),
            Some(site.list_url(&registration.slug)),
        ),
        (format!("Add {}", registration.verbose_name), None),
    ];

    render_admin_page(
        site,
        &format!("Add {}", registration.verbose_name.to_lowercase()),
        content,
        breadcrumbs,
        messages,
        user_name,
    )
}

/// Data for rendering the change view.
pub struct ChangeViewData {
    /// String representation of the object.
    pub object_str: String,
    /// Primary key of the object.
    pub pk: String,
    /// Rendered form fields HTML.
    pub form_html: String,
    /// Fieldsets with their field HTML.
    pub fieldsets: Vec<FieldsetData>,
    /// Inline formsets for related models.
    pub inlines: Vec<InlineData>,
}

/// Renders the change view for a model.
pub fn change_view(
    site: &AdminSite,
    registration: &ModelRegistration,
    data: ChangeViewData,
    errors: Vec<String>,
    messages: Vec<(String, String)>,
    user_name: Option<String>,
) -> Response {
    let ctx = DetailViewContext {
        model_name: registration.name.clone(),
        is_new: false,
        object_str: Some(data.object_str.clone()),
        form_html: data.form_html,
        fieldsets: data.fieldsets.into_iter().map(Into::into).collect(),
        inlines: data.inlines.into_iter().map(Into::into).collect(),
        list_url: site.list_url(&registration.slug),
        delete_url: Some(site.delete_url(&registration.slug, &data.pk)),
        action_url: site.change_url(&registration.slug, &data.pk),
        errors,
    };

    let content = render_detail_view(&ctx);

    let breadcrumbs = vec![
        ("Home".to_string(), Some(format!("{}/", site.url_prefix))),
        (
            registration.verbose_name_plural.clone(),
            Some(site.list_url(&registration.slug)),
        ),
        (data.object_str, None),
    ];

    render_admin_page(
        site,
        &format!("Change {}", registration.verbose_name.to_lowercase()),
        content,
        breadcrumbs,
        messages,
        user_name,
    )
}

/// Result of processing a form submission.
#[derive(Debug)]
pub enum FormResult {
    /// Form was valid, object was saved. Contains the PK.
    Success { pk: String, message: String },
    /// Form had validation errors.
    ValidationError { errors: Vec<String> },
    /// Save operation failed.
    SaveError { error: String },
}

impl FormResult {
    /// Creates a success result.
    pub fn success(pk: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Success {
            pk: pk.into(),
            message: message.into(),
        }
    }

    /// Creates a validation error result.
    pub fn validation_errors(errors: Vec<String>) -> Self {
        Self::ValidationError { errors }
    }

    /// Creates a save error result.
    pub fn save_error(error: impl Into<String>) -> Self {
        Self::SaveError {
            error: error.into(),
        }
    }
}

/// Determines the redirect URL based on the submit button.
pub fn get_redirect_url(site: &AdminSite, slug: &str, pk: &str, submit_action: &str) -> String {
    match submit_action {
        "_continue" => site.change_url(slug, pk),
        "_addanother" => site.add_url(slug),
        _ => site.list_url(slug), // _save or default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_redirect_url() {
        let site = AdminSite::new("Test");

        assert_eq!(
            get_redirect_url(&site, "user", "1", "_save"),
            "/admin/user/"
        );
        assert_eq!(
            get_redirect_url(&site, "user", "1", "_continue"),
            "/admin/user/1/change/"
        );
        assert_eq!(
            get_redirect_url(&site, "user", "1", "_addanother"),
            "/admin/user/add/"
        );
    }

    #[test]
    fn test_form_result() {
        let success = FormResult::success("123", "Saved successfully");
        match success {
            FormResult::Success { pk, message } => {
                assert_eq!(pk, "123");
                assert!(message.contains("successfully"));
            }
            _ => panic!("Expected Success variant"),
        }
    }
}
