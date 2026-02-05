//! Detail/edit view template.

use super::html_escape;

/// Context for rendering a detail/edit view.
#[derive(Debug, Clone)]
pub struct DetailViewContext {
    /// Model name.
    pub model_name: String,
    /// Whether this is a new object (add) or existing (change).
    pub is_new: bool,
    /// Object string representation (for existing objects).
    pub object_str: Option<String>,
    /// Form HTML (rendered by oxide-forms).
    pub form_html: String,
    /// Fieldsets for grouping fields.
    pub fieldsets: Vec<Fieldset>,
    /// Inline formsets.
    pub inlines: Vec<InlineFormset>,
    /// URL to return to list view.
    pub list_url: String,
    /// Delete URL (for existing objects).
    pub delete_url: Option<String>,
    /// Form action URL.
    pub action_url: String,
    /// Validation errors.
    pub errors: Vec<String>,
}

/// A fieldset groups related fields together.
#[derive(Debug, Clone)]
pub struct Fieldset {
    /// Optional title.
    pub name: Option<String>,
    /// Field HTML for each field in this set.
    pub fields: Vec<String>,
    /// CSS classes (e.g., "collapse").
    pub classes: Vec<String>,
    /// Optional description.
    pub description: Option<String>,
}

/// An inline formset for related objects.
#[derive(Debug, Clone)]
pub struct InlineFormset {
    /// Related model name.
    pub model_name: String,
    /// Verbose name (plural).
    pub verbose_name: String,
    /// Column headers.
    pub columns: Vec<String>,
    /// Existing rows (edit forms).
    pub rows: Vec<InlineRow>,
    /// Number of empty forms to show.
    pub extra: usize,
    /// Empty form template.
    pub empty_form: String,
}

/// A row in an inline formset.
#[derive(Debug, Clone)]
pub struct InlineRow {
    /// Row ID.
    pub id: String,
    /// Field inputs for this row.
    pub fields: Vec<String>,
    /// Delete checkbox.
    pub delete_checkbox: String,
}

impl Default for DetailViewContext {
    fn default() -> Self {
        Self {
            model_name: String::new(),
            is_new: true,
            object_str: None,
            form_html: String::new(),
            fieldsets: Vec::new(),
            inlines: Vec::new(),
            list_url: String::new(),
            delete_url: None,
            action_url: String::new(),
            errors: Vec::new(),
        }
    }
}

/// Renders the detail view content.
pub fn render_detail_view(ctx: &DetailViewContext) -> String {
    let _title = if ctx.is_new {
        format!("Add {}", ctx.model_name)
    } else {
        format!(
            "Change {}",
            ctx.object_str.as_deref().unwrap_or(&ctx.model_name)
        )
    };

    let errors_html = render_errors(&ctx.errors);
    let fieldsets_html = render_fieldsets(&ctx.fieldsets, &ctx.form_html);
    let inlines_html = render_inlines(&ctx.inlines);
    let delete_button = render_delete_button(&ctx.delete_url);

    format!(
        r#"<form method="post" action="{action_url}" enctype="multipart/form-data">
    {errors_html}

    <div class="row">
        <div class="col-lg-8">
            {fieldsets_html}
            {inlines_html}
        </div>
        <div class="col-lg-4">
            <div class="card sticky-top" style="top: 1rem;">
                <div class="card-header">
                    <strong>Actions</strong>
                </div>
                <div class="card-body">
                    <div class="d-grid gap-2">
                        <button type="submit" name="_save" class="btn btn-primary">
                            <i class="bi bi-check-lg me-1"></i>Save
                        </button>
                        <button type="submit" name="_continue" class="btn btn-outline-primary">
                            <i class="bi bi-arrow-repeat me-1"></i>Save and continue editing
                        </button>
                        <button type="submit" name="_addanother" class="btn btn-outline-secondary">
                            <i class="bi bi-plus-lg me-1"></i>Save and add another
                        </button>
                        <a href="{list_url}" class="btn btn-outline-secondary">
                            <i class="bi bi-x-lg me-1"></i>Cancel
                        </a>
                        {delete_button}
                    </div>
                </div>
            </div>
        </div>
    </div>
</form>"#,
        action_url = html_escape(&ctx.action_url),
        errors_html = errors_html,
        fieldsets_html = fieldsets_html,
        inlines_html = inlines_html,
        list_url = html_escape(&ctx.list_url),
        delete_button = delete_button,
    )
}

fn render_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        return String::new();
    }

    let items: Vec<String> = errors
        .iter()
        .map(|e| format!("<li>{}</li>", html_escape(e)))
        .collect();

    format!(
        r#"<div class="alert alert-danger" role="alert">
            <strong>Please correct the errors below:</strong>
            <ul class="mb-0 mt-2">{}</ul>
        </div>"#,
        items.join("\n")
    )
}

fn render_fieldsets(fieldsets: &[Fieldset], form_html: &str) -> String {
    if fieldsets.is_empty() {
        // No fieldsets defined, render form directly in a card
        return format!(
            r#"<div class="card mb-4">
                <div class="card-body">
                    {}
                </div>
            </div>"#,
            form_html
        );
    }

    fieldsets
        .iter()
        .map(|fieldset| {
            let title = fieldset
                .name
                .as_ref()
                .map(|n| format!(r#"<div class="card-header">{}</div>"#, html_escape(n)))
                .unwrap_or_default();

            let description = fieldset
                .description
                .as_ref()
                .map(|d| {
                    format!(
                        r#"<p class="text-muted mb-3">{}</p>"#,
                        html_escape(d)
                    )
                })
                .unwrap_or_default();

            let is_collapsed = fieldset.classes.contains(&"collapse".to_string());
            let _collapse_class = if is_collapsed { " collapse" } else { "" };
            let collapse_id = fieldset
                .name
                .as_ref()
                .map(|n| n.to_lowercase().replace(' ', "_"))
                .unwrap_or_else(|| "fieldset".to_string());

            let fields_html = fieldset.fields.join("\n");

            if is_collapsed {
                format!(
                    "<div class=\"card mb-4\">\
                        <div class=\"card-header d-flex justify-content-between align-items-center\">\
                            <span>{}</span>\
                            <button class=\"btn btn-link btn-sm\" type=\"button\" data-bs-toggle=\"collapse\" data-bs-target=\"#{collapse_id}\">\
                                <i class=\"bi bi-chevron-down\"></i>\
                            </button>\
                        </div>\
                        <div id=\"{collapse_id}\" class=\"collapse\">\
                            <div class=\"card-body\">\
                                {description}\
                                {fields_html}\
                            </div>\
                        </div>\
                    </div>",
                    html_escape(fieldset.name.as_deref().unwrap_or("")),
                    collapse_id = collapse_id,
                    description = description,
                    fields_html = fields_html,
                )
            } else {
                format!(
                    "<div class=\"card mb-4\">\
                        {title}\
                        <div class=\"card-body\">\
                            {description}\
                            {fields_html}\
                        </div>\
                    </div>",
                    title = title,
                    description = description,
                    fields_html = fields_html,
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_inlines(inlines: &[InlineFormset]) -> String {
    inlines
        .iter()
        .map(|inline| {
            let headers: Vec<String> = inline
                .columns
                .iter()
                .map(|col| format!("<th>{}</th>", html_escape(col)))
                .collect();

            let rows: Vec<String> = inline
                .rows
                .iter()
                .map(|row| {
                    let cells: Vec<String> = row
                        .fields
                        .iter()
                        .map(|field| format!("<td>{}</td>", field))
                        .collect();

                    format!(
                        r#"<tr data-inline-row="{}">
                            {}
                            <td class="text-center">{}</td>
                        </tr>"#,
                        html_escape(&row.id),
                        cells.join("\n"),
                        row.delete_checkbox
                    )
                })
                .collect();

            // Add empty forms
            let empty_rows: Vec<String> = (0..inline.extra)
                .map(|i| {
                    format!(
                        r#"<tr data-inline-row="__prefix__" data-inline-index="{}">
                            {}
                            <td class="text-center">
                                <button type="button" class="btn btn-sm btn-outline-danger remove-inline-row">
                                    <i class="bi bi-x"></i>
                                </button>
                            </td>
                        </tr>"#,
                        i,
                        inline.empty_form
                    )
                })
                .collect();

            format!(
                r#"<div class="card mb-4">
                    <div class="card-header d-flex justify-content-between align-items-center">
                        <span><strong>{verbose_name}</strong></span>
                        <button type="button" class="btn btn-sm btn-outline-primary add-inline-row">
                            <i class="bi bi-plus me-1"></i>Add another
                        </button>
                    </div>
                    <div class="card-body p-0">
                        <div class="table-responsive">
                            <table class="table table-sm mb-0">
                                <thead class="table-light">
                                    <tr>
                                        {headers}
                                        <th style="width: 60px;">Delete</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows}
                                    {empty_rows}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>"#,
                verbose_name = html_escape(&inline.verbose_name),
                headers = headers.join("\n"),
                rows = rows.join("\n"),
                empty_rows = empty_rows.join("\n"),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_delete_button(delete_url: &Option<String>) -> String {
    match delete_url {
        Some(url) => format!(
            r#"<hr>
            <a href="{}" class="btn btn-outline-danger w-100">
                <i class="bi bi-trash me-1"></i>Delete
            </a>"#,
            html_escape(url)
        ),
        None => String::new(),
    }
}
