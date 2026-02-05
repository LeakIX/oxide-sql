//! Bootstrap 5 form widgets.

use super::{html_escape, Widget, WidgetAttrs};

/// Bootstrap 5 text input widget.
#[derive(Debug, Clone)]
pub struct BootstrapTextInput {
    /// The HTML input type (text, email, password, etc.).
    pub input_type: String,
    /// Placeholder text.
    pub placeholder: Option<String>,
    /// Whether to use floating labels.
    pub floating_label: bool,
}

impl Default for BootstrapTextInput {
    fn default() -> Self {
        Self {
            input_type: "text".to_string(),
            placeholder: None,
            floating_label: false,
        }
    }
}

impl BootstrapTextInput {
    /// Creates a new text input.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a password input.
    pub fn password() -> Self {
        Self {
            input_type: "password".to_string(),
            ..Default::default()
        }
    }

    /// Creates an email input.
    pub fn email() -> Self {
        Self {
            input_type: "email".to_string(),
            ..Default::default()
        }
    }

    /// Creates a number input.
    pub fn number() -> Self {
        Self {
            input_type: "number".to_string(),
            ..Default::default()
        }
    }

    /// Sets the placeholder text.
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Enables floating labels.
    #[must_use]
    pub fn floating(mut self) -> Self {
        self.floating_label = true;
        self
    }
}

impl Widget for BootstrapTextInput {
    fn render(&self, name: &str, value: Option<&str>, attrs: &WidgetAttrs) -> String {
        let value_attr = value
            .map(|v| format!(r#" value="{}""#, html_escape(v)))
            .unwrap_or_default();

        let placeholder_attr = self
            .placeholder
            .as_ref()
            .map(|p| format!(r#" placeholder="{}""#, html_escape(p)))
            .unwrap_or_default();

        let id = attrs
            .get("id")
            .cloned()
            .unwrap_or_else(|| format!("id_{name}"));

        let mut class = "form-control".to_string();
        if let Some(extra_class) = attrs.get("class") {
            class = format!("{class} {extra_class}");
        }

        let extra_attrs: String = attrs
            .attrs
            .iter()
            .filter(|(k, _)| k.as_str() != "class" && k.as_str() != "id")
            .map(|(k, v)| format!(r#" {k}="{v}""#))
            .collect();

        format!(
            r#"<input type="{}" class="{}" id="{}" name="{}"{}{}{extra_attrs}>"#,
            self.input_type, class, id, name, value_attr, placeholder_attr
        )
    }

    fn input_type(&self) -> &str {
        &self.input_type
    }
}

/// Bootstrap 5 textarea widget.
#[derive(Debug, Clone)]
pub struct BootstrapTextarea {
    /// Number of rows.
    pub rows: usize,
    /// Placeholder text.
    pub placeholder: Option<String>,
}

impl Default for BootstrapTextarea {
    fn default() -> Self {
        Self {
            rows: 4,
            placeholder: None,
        }
    }
}

impl BootstrapTextarea {
    /// Creates a new textarea with the specified rows.
    pub fn new(rows: usize) -> Self {
        Self {
            rows,
            placeholder: None,
        }
    }

    /// Sets the placeholder text.
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }
}

impl Widget for BootstrapTextarea {
    fn render(&self, name: &str, value: Option<&str>, attrs: &WidgetAttrs) -> String {
        let content = value.map(html_escape).unwrap_or_default();
        let id = attrs
            .get("id")
            .cloned()
            .unwrap_or_else(|| format!("id_{name}"));

        let placeholder_attr = self
            .placeholder
            .as_ref()
            .map(|p| format!(r#" placeholder="{}""#, html_escape(p)))
            .unwrap_or_default();

        let mut class = "form-control".to_string();
        if let Some(extra_class) = attrs.get("class") {
            class = format!("{class} {extra_class}");
        }

        format!(
            r#"<textarea class="{}" id="{}" name="{}" rows="{}"{placeholder_attr}>{}</textarea>"#,
            class, id, name, self.rows, content
        )
    }

    fn input_type(&self) -> &str {
        "textarea"
    }
}

/// Bootstrap 5 select widget.
#[derive(Debug, Clone)]
pub struct BootstrapSelect {
    /// Available choices (value, label).
    pub choices: Vec<(String, String)>,
    /// Whether to include an empty option.
    pub include_blank: bool,
    /// Label for blank option.
    pub blank_label: String,
}

impl Default for BootstrapSelect {
    fn default() -> Self {
        Self {
            choices: Vec::new(),
            include_blank: true,
            blank_label: "---------".to_string(),
        }
    }
}

impl BootstrapSelect {
    /// Creates a new select with the given choices.
    pub fn new(choices: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        Self {
            choices: choices
                .into_iter()
                .map(|(v, l)| (v.into(), l.into()))
                .collect(),
            ..Default::default()
        }
    }

    /// Disables the blank option.
    #[must_use]
    pub fn no_blank(mut self) -> Self {
        self.include_blank = false;
        self
    }

    /// Sets the blank label.
    #[must_use]
    pub fn blank_label(mut self, label: impl Into<String>) -> Self {
        self.blank_label = label.into();
        self
    }
}

impl Widget for BootstrapSelect {
    fn render(&self, name: &str, value: Option<&str>, attrs: &WidgetAttrs) -> String {
        let id = attrs
            .get("id")
            .cloned()
            .unwrap_or_else(|| format!("id_{name}"));

        let mut class = "form-select".to_string();
        if let Some(extra_class) = attrs.get("class") {
            class = format!("{class} {extra_class}");
        }

        let mut options = String::new();

        if self.include_blank {
            options.push_str(&format!(
                r#"<option value="">{}</option>"#,
                html_escape(&self.blank_label)
            ));
        }

        for (opt_value, label) in &self.choices {
            let selected = value.is_some_and(|v| v == opt_value);
            let selected_attr = if selected { " selected" } else { "" };
            options.push_str(&format!(
                r#"<option value="{}"{selected_attr}>{}</option>"#,
                html_escape(opt_value),
                html_escape(label)
            ));
        }

        format!(
            r#"<select class="{}" id="{}" name="{}">{}</select>"#,
            class, id, name, options
        )
    }

    fn input_type(&self) -> &str {
        "select"
    }
}

/// Bootstrap 5 checkbox widget.
#[derive(Debug, Clone, Default)]
pub struct BootstrapCheckbox {
    /// Label for the checkbox.
    pub label: Option<String>,
    /// Whether to use switch style.
    pub is_switch: bool,
}

impl BootstrapCheckbox {
    /// Creates a new checkbox.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a switch-style checkbox.
    pub fn switch() -> Self {
        Self {
            is_switch: true,
            ..Default::default()
        }
    }

    /// Sets the label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Widget for BootstrapCheckbox {
    fn render(&self, name: &str, value: Option<&str>, attrs: &WidgetAttrs) -> String {
        let id = attrs
            .get("id")
            .cloned()
            .unwrap_or_else(|| format!("id_{name}"));
        let checked = value.is_some_and(|v| v == "true" || v == "on" || v == "1");
        let checked_attr = if checked { " checked" } else { "" };

        let wrapper_class = if self.is_switch {
            "form-check form-switch"
        } else {
            "form-check"
        };

        let label_html = self
            .label
            .as_ref()
            .map(|l| {
                format!(
                    r#"<label class="form-check-label" for="{}">{}</label>"#,
                    id,
                    html_escape(l)
                )
            })
            .unwrap_or_default();

        format!(
            r#"<div class="{}">
  <input class="form-check-input" type="checkbox" id="{}" name="{}" value="true"{checked_attr}>
  {}
</div>"#,
            wrapper_class, id, name, label_html
        )
    }

    fn input_type(&self) -> &str {
        "checkbox"
    }
}

/// Bootstrap 5 radio select widget.
#[derive(Debug, Clone, Default)]
pub struct BootstrapRadioSelect {
    /// Available choices (value, label).
    pub choices: Vec<(String, String)>,
    /// Whether to display inline.
    pub inline: bool,
}

impl BootstrapRadioSelect {
    /// Creates a new radio select with the given choices.
    pub fn new(choices: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        Self {
            choices: choices
                .into_iter()
                .map(|(v, l)| (v.into(), l.into()))
                .collect(),
            inline: false,
        }
    }

    /// Makes the radios display inline.
    #[must_use]
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }
}

impl Widget for BootstrapRadioSelect {
    fn render(&self, name: &str, value: Option<&str>, _attrs: &WidgetAttrs) -> String {
        let wrapper_class = if self.inline {
            "form-check form-check-inline"
        } else {
            "form-check"
        };

        let mut html = String::new();

        for (i, (opt_value, label)) in self.choices.iter().enumerate() {
            let id = format!("id_{name}_{i}");
            let checked = value.is_some_and(|v| v == opt_value);
            let checked_attr = if checked { " checked" } else { "" };

            html.push_str(&format!(
                r#"<div class="{}">
  <input class="form-check-input" type="radio" id="{}" name="{}" value="{}"{checked_attr}>
  <label class="form-check-label" for="{}">{}</label>
</div>
"#,
                wrapper_class,
                id,
                name,
                html_escape(opt_value),
                id,
                html_escape(label)
            ));
        }

        html
    }

    fn input_type(&self) -> &str {
        "radio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_text_input() {
        let widget = BootstrapTextInput::new().placeholder("Enter name");
        let html = widget.render("username", None, &WidgetAttrs::new());
        assert!(html.contains(r#"class="form-control""#));
        assert!(html.contains(r#"name="username""#));
        assert!(html.contains(r#"placeholder="Enter name""#));
    }

    #[test]
    fn test_bootstrap_password() {
        let widget = BootstrapTextInput::password();
        let html = widget.render("password", None, &WidgetAttrs::new());
        assert!(html.contains(r#"type="password""#));
    }

    #[test]
    fn test_bootstrap_textarea() {
        let widget = BootstrapTextarea::new(6);
        let html = widget.render("content", Some("Hello"), &WidgetAttrs::new());
        assert!(html.contains(r#"class="form-control""#));
        assert!(html.contains(r#"rows="6""#));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn test_bootstrap_select() {
        let widget = BootstrapSelect::new(vec![("1", "Option 1"), ("2", "Option 2")]);
        let html = widget.render("choice", Some("2"), &WidgetAttrs::new());
        assert!(html.contains(r#"class="form-select""#));
        assert!(html.contains(r#"value="2" selected"#));
    }

    #[test]
    fn test_bootstrap_checkbox() {
        let widget = BootstrapCheckbox::new().label("I agree");
        let html = widget.render("agree", Some("true"), &WidgetAttrs::new());
        assert!(html.contains("form-check"));
        assert!(html.contains("checked"));
        assert!(html.contains("I agree"));
    }

    #[test]
    fn test_bootstrap_switch() {
        let widget = BootstrapCheckbox::switch();
        let html = widget.render("enabled", None, &WidgetAttrs::new());
        assert!(html.contains("form-switch"));
    }

    #[test]
    fn test_bootstrap_radio_select() {
        let widget = BootstrapRadioSelect::new(vec![("a", "Option A"), ("b", "Option B")]);
        let html = widget.render("choice", Some("b"), &WidgetAttrs::new());
        assert!(html.contains("form-check"));
        assert!(html.contains(r#"value="b" checked"#));
    }
}
