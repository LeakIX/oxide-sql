//! Hidden field type.

use crate::form::FormFieldDef;
use crate::widgets::HiddenInput;

/// Creates a hidden field.
pub fn hidden_field(name: &str, initial: Option<&str>) -> FormFieldDef {
    let mut field = FormFieldDef::new(name, "", HiddenInput);

    if let Some(value) = initial {
        field = field.initial(value);
    }

    field
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hidden_field() {
        let field = hidden_field("csrf_token", Some("abc123"));
        assert_eq!(field.name, "csrf_token");
        assert_eq!(field.initial, Some("abc123".to_string()));
    }
}
