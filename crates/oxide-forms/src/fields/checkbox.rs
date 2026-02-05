//! Checkbox field types.

use crate::form::FormFieldDef;
use crate::widgets::BootstrapCheckbox;

/// Creates a boolean field (checkbox).
pub fn boolean_field(name: &str, label: &str) -> FormFieldDef {
    FormFieldDef::new(name, label, BootstrapCheckbox::new().label(label))
}

/// Creates a checkbox field with custom styling.
pub fn checkbox_field(name: &str, label: &str, is_switch: bool) -> FormFieldDef {
    let widget = if is_switch {
        BootstrapCheckbox::switch().label(label)
    } else {
        BootstrapCheckbox::new().label(label)
    };

    FormFieldDef::new(name, label, widget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_field() {
        let field = boolean_field("active", "Is Active");
        assert_eq!(field.name, "active");
        assert!(!field.required);
    }

    #[test]
    fn test_checkbox_switch() {
        let field = checkbox_field("enabled", "Enable feature", true);
        assert_eq!(field.name, "enabled");
    }
}
