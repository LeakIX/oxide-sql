//! Form trait and form rendering.

use std::collections::HashMap;

use ironhtml::html;
use ironhtml::typed::Element;
use ironhtml_elements::{Div, Li, Ul};

use crate::error::{Result, ValidationErrors};
use crate::validation::Validator;
use crate::widgets::{Widget, WidgetAttrs};

/// Definition of a form field.
pub struct FormFieldDef {
    /// Field name.
    pub name: String,
    /// Field label.
    pub label: String,
    /// Whether the field is required.
    pub required: bool,
    /// The widget to render.
    pub widget: Box<dyn Widget>,
    /// Help text.
    pub help_text: Option<String>,
    /// Initial value.
    pub initial: Option<String>,
    /// Validators.
    pub validators: Vec<Box<dyn Validator>>,
    /// Widget attributes.
    pub attrs: WidgetAttrs,
    /// Whether the field is disabled.
    pub disabled: bool,
}

impl std::fmt::Debug for FormFieldDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormFieldDef")
            .field("name", &self.name)
            .field("label", &self.label)
            .field("required", &self.required)
            .field("help_text", &self.help_text)
            .field("initial", &self.initial)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl FormFieldDef {
    /// Creates a new field definition.
    pub fn new(
        name: impl Into<String>,
        label: impl Into<String>,
        widget: impl Widget + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            required: false,
            widget: Box::new(widget),
            help_text: None,
            initial: None,
            validators: Vec::new(),
            attrs: WidgetAttrs::new(),
            disabled: false,
        }
    }

    /// Makes the field required.
    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Sets help text.
    #[must_use]
    pub fn help_text(mut self, text: impl Into<String>) -> Self {
        self.help_text = Some(text.into());
        self
    }

    /// Sets initial value.
    #[must_use]
    pub fn initial(mut self, value: impl Into<String>) -> Self {
        self.initial = Some(value.into());
        self
    }

    /// Adds a validator.
    #[must_use]
    pub fn validator(mut self, validator: impl Validator + 'static) -> Self {
        self.validators.push(Box::new(validator));
        self
    }

    /// Sets a widget attribute.
    #[must_use]
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.set(key, value);
        self
    }

    /// Disables the field.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Trait for form types.
pub trait Form: Sized {
    /// Returns the field definitions for this form.
    fn fields() -> Vec<FormFieldDef>;

    /// Validates the form data.
    fn validate(&self) -> std::result::Result<(), ValidationErrors>;

    /// Creates a form instance from submitted data.
    fn from_data(data: &HashMap<String, String>) -> Result<Self>;

    /// Renders the form as Bootstrap 5 HTML.
    fn as_bootstrap(&self) -> String;
}

/// Renders a form field with Bootstrap 5 styling.
pub fn render_bootstrap_field(
    field: &FormFieldDef,
    value: Option<&str>,
    errors: &[String],
) -> String {
    let id = format!("id_{}", field.name);
    let has_errors = !errors.is_empty();

    let required_marker = if field.required { " *" } else { "" };
    let label_text = format!("{}{}", field.label, required_marker);

    // Prepare widget attrs
    let mut attrs = field.attrs.clone();
    attrs.set("id", &id);

    if has_errors {
        let current_class = attrs.get("class").cloned().unwrap_or_default();
        attrs.set("class", format!("{current_class} is-invalid").trim());
    }

    if field.disabled {
        attrs.set("disabled", "disabled");
    }

    if field.required {
        attrs.set("required", "required");
    }

    let actual_value = value.or(field.initial.as_deref());
    let widget_html = field.widget.render(&field.name, actual_value, &attrs);

    let label_el = html! {
        label.for_(#id).class("form-label") { #label_text }
    };

    let help_text = field.help_text.clone();

    html! { div.class("mb-3") }
        .raw(label_el.render())
        .raw(&widget_html)
        .children(errors, |error, div: Element<Div>| {
            div.class("invalid-feedback").text(error)
        })
        .when(help_text.is_some(), |d| {
            d.child::<Div, _>(|h| {
                h.class("form-text")
                    .text(help_text.as_deref().unwrap_or(""))
            })
        })
        .render()
}

/// Renders a complete form with Bootstrap 5 styling.
pub fn render_bootstrap_form(
    fields: &[FormFieldDef],
    values: &HashMap<String, String>,
    errors: &ValidationErrors,
    action: &str,
    method: &str,
) -> String {
    let mut form = html! {
        form.action(#action).method(#method)
    };

    // Non-field errors
    if let Some(form_errors) = errors.get("__all__") {
        form = form.child::<Div, _>(|d| {
            d.class("alert alert-danger")
                .attr("role", "alert")
                .child::<Ul, _>(|ul| {
                    ul.class("mb-0")
                        .children(form_errors.iter(), |e, li: Element<Li>| li.text(e))
                })
        });
    }

    // Render each field inside a wrapper div
    for field in fields {
        let value = values.get(&field.name).map(String::as_str);
        let field_errors = errors.get(&field.name).cloned().unwrap_or_default();
        let field_html = render_bootstrap_field(field, value, &field_errors);
        form = form.child::<Div, _>(|d| d.raw(&field_html));
    }

    // Submit button
    form = form.child::<Div, _>(|d| {
        let btn = html! {
            button.type_("submit").class("btn btn-primary") {
                "Submit"
            }
        };
        d.raw(btn.render())
    });

    form.render()
}

/// A simple form builder for creating forms programmatically.
#[derive(Debug, Default)]
pub struct FormBuilder {
    fields: Vec<FormFieldDef>,
}

impl FormBuilder {
    /// Creates a new form builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a field to the form.
    #[must_use]
    pub fn field(mut self, field: FormFieldDef) -> Self {
        self.fields.push(field);
        self
    }

    /// Returns the field definitions.
    pub fn build(self) -> Vec<FormFieldDef> {
        self.fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::BootstrapTextInput;

    #[test]
    fn test_field_def_builder() {
        let field = FormFieldDef::new("username", "Username", BootstrapTextInput::new())
            .required()
            .help_text("Choose a unique username")
            .attr("placeholder", "Enter username");

        assert_eq!(field.name, "username");
        assert_eq!(field.label, "Username");
        assert!(field.required);
        assert_eq!(
            field.help_text,
            Some("Choose a unique username".to_string())
        );
    }

    #[test]
    fn test_render_field_no_errors() {
        let field = FormFieldDef::new("email", "Email", BootstrapTextInput::email())
            .required()
            .help_text("We will never share your email");

        let html = render_bootstrap_field(&field, Some("test@example.com"), &[]);
        assert!(html.contains("form-label"));
        assert!(html.contains("Email *"));
        assert!(html.contains("We will never share your email"));
        assert!(!html.contains("is-invalid"));
    }

    #[test]
    fn test_render_field_with_errors() {
        let field = FormFieldDef::new("email", "Email", BootstrapTextInput::email());

        let html = render_bootstrap_field(&field, None, &["Invalid email address".to_string()]);
        assert!(html.contains("is-invalid"));
        assert!(html.contains("Invalid email address"));
    }

    #[test]
    fn test_form_builder() {
        let fields = FormBuilder::new()
            .field(FormFieldDef::new("name", "Name", BootstrapTextInput::new()).required())
            .field(FormFieldDef::new(
                "email",
                "Email",
                BootstrapTextInput::email(),
            ))
            .build();

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "name");
        assert_eq!(fields[1].name, "email");
    }
}
