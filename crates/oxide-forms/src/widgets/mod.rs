//! Form widgets for rendering HTML inputs.

mod bootstrap;

pub use bootstrap::{
    BootstrapCheckbox, BootstrapRadioSelect, BootstrapSelect, BootstrapTextInput, BootstrapTextarea,
};

use std::collections::HashMap;

/// Attributes that can be applied to a widget.
#[derive(Debug, Clone, Default)]
pub struct WidgetAttrs {
    /// HTML attributes.
    pub attrs: HashMap<String, String>,
}

impl WidgetAttrs {
    /// Creates new empty widget attributes.
    pub fn new() -> Self {
        Self {
            attrs: HashMap::new(),
        }
    }

    /// Sets an attribute.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attrs.insert(key.into(), value.into());
    }

    /// Gets an attribute.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.attrs.get(key)
    }

    /// Renders attributes as an HTML attribute string.
    pub fn to_html(&self) -> String {
        self.attrs
            .iter()
            .map(|(k, v)| format!(r#"{k}="{v}""#))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Builder method to set an attribute.
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.set(key, value);
        self
    }
}

/// Trait for form widgets that render HTML inputs.
pub trait Widget: Send + Sync {
    /// Renders the widget as HTML.
    ///
    /// # Arguments
    /// * `name` - The field name (used for the name attribute)
    /// * `value` - The current value (if any)
    /// * `attrs` - Additional HTML attributes
    fn render(&self, name: &str, value: Option<&str>, attrs: &WidgetAttrs) -> String;

    /// Returns the HTML input type.
    fn input_type(&self) -> &str {
        "text"
    }
}

/// A hidden input widget.
#[derive(Debug, Clone, Default)]
pub struct HiddenInput;

impl Widget for HiddenInput {
    fn render(&self, name: &str, value: Option<&str>, attrs: &WidgetAttrs) -> String {
        let value_attr = value
            .map(|v| format!(r#" value="{}""#, html_escape(v)))
            .unwrap_or_default();
        let extra_attrs = if attrs.attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.to_html())
        };
        format!(r#"<input type="hidden" name="{name}"{value_attr}{extra_attrs}>"#)
    }

    fn input_type(&self) -> &str {
        "hidden"
    }
}

/// A simple text input widget (non-Bootstrap).
#[derive(Debug, Clone, Default)]
pub struct TextInput;

impl Widget for TextInput {
    fn render(&self, name: &str, value: Option<&str>, attrs: &WidgetAttrs) -> String {
        let value_attr = value
            .map(|v| format!(r#" value="{}""#, html_escape(v)))
            .unwrap_or_default();
        let extra_attrs = if attrs.attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.to_html())
        };
        format!(r#"<input type="text" name="{name}"{value_attr}{extra_attrs}>"#)
    }
}

/// A simple textarea widget (non-Bootstrap).
#[derive(Debug, Clone)]
pub struct Textarea {
    /// Number of rows.
    pub rows: usize,
    /// Number of columns.
    pub cols: usize,
}

impl Default for Textarea {
    fn default() -> Self {
        Self { rows: 4, cols: 40 }
    }
}

impl Widget for Textarea {
    fn render(&self, name: &str, value: Option<&str>, attrs: &WidgetAttrs) -> String {
        let content = value.map(html_escape).unwrap_or_default();
        let extra_attrs = if attrs.attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.to_html())
        };
        format!(
            r#"<textarea name="{}" rows="{}" cols="{}"{extra_attrs}>{}</textarea>"#,
            name, self.rows, self.cols, content
        )
    }

    fn input_type(&self) -> &str {
        "textarea"
    }
}

/// Escapes HTML special characters.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hidden_input() {
        let widget = HiddenInput;
        let html = widget.render("csrf_token", Some("abc123"), &WidgetAttrs::new());
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="csrf_token""#));
        assert!(html.contains(r#"value="abc123""#));
    }

    #[test]
    fn test_text_input() {
        let widget = TextInput;
        let html = widget.render("username", None, &WidgetAttrs::new());
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"name="username""#));
    }

    #[test]
    fn test_textarea() {
        let widget = Textarea::default();
        let html = widget.render("content", Some("Hello"), &WidgetAttrs::new());
        assert!(html.contains(r#"name="content""#));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("\"test\""), "&quot;test&quot;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_widget_attrs() {
        let attrs = WidgetAttrs::new()
            .with("class", "form-control")
            .with("id", "my-input");
        let html = attrs.to_html();
        assert!(html.contains(r#"class="form-control""#));
        assert!(html.contains(r#"id="my-input""#));
    }
}
