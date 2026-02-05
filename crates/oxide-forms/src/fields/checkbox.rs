//! Checkbox field types.

use crate::form::FormFieldDef;
use crate::widgets::BootstrapCheckbox;

/// Creates a boolean field (checkbox).
#[allow(non_snake_case)]
pub fn BooleanField(name: &str, label: &str) -> FormFieldDef {
    FormFieldDef::new(name, label, BootstrapCheckbox::new().label(label))
}

/// Creates a checkbox field with custom styling.
#[allow(non_snake_case)]
pub fn CheckboxField(name: &str, label: &str, is_switch: bool) -> FormFieldDef {
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
        let field = BooleanField("active", "Is Active");
        assert_eq!(field.name, "active");
        assert!(!field.required);
    }

    #[test]
    fn test_checkbox_switch() {
        let field = CheckboxField("enabled", "Enable feature", true);
        assert_eq!(field.name, "enabled");
    }
}
