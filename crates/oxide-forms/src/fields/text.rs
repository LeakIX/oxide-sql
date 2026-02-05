//! Text field types.

use crate::form::FormFieldDef;
use crate::validation::{
    EmailValidator, MaxLengthValidator, MinLengthValidator, RequiredValidator, UrlValidator,
};
use crate::widgets::{BootstrapTextInput, BootstrapTextarea};

/// Creates a character field (text input with max length).
pub fn CharField(name: &str, label: &str, max_length: usize, required: bool) -> FormFieldDef {
    let mut field = FormFieldDef::new(name, label, BootstrapTextInput::new())
        .validator(MaxLengthValidator::new(max_length));

    if required {
        field = field.required().validator(RequiredValidator::new());
    }

    field
}

/// Creates a text field (textarea).
pub fn TextField(name: &str, label: &str, rows: usize, required: bool) -> FormFieldDef {
    let mut field = FormFieldDef::new(name, label, BootstrapTextarea::new(rows));

    if required {
        field = field.required().validator(RequiredValidator::new());
    }

    field
}

/// Creates an email field.
pub fn EmailField(name: &str, label: &str, required: bool) -> FormFieldDef {
    let mut field = FormFieldDef::new(name, label, BootstrapTextInput::email())
        .validator(EmailValidator::new());

    if required {
        field = field.required().validator(RequiredValidator::new());
    }

    field
}

/// Creates a password field.
pub fn PasswordField(name: &str, label: &str, min_length: Option<usize>) -> FormFieldDef {
    let mut field = FormFieldDef::new(name, label, BootstrapTextInput::password())
        .required()
        .validator(RequiredValidator::new());

    if let Some(min) = min_length {
        field = field.validator(MinLengthValidator::new(min));
    }

    field
}

/// Creates a URL field.
pub fn UrlField(name: &str, label: &str, required: bool) -> FormFieldDef {
    let mut field = FormFieldDef::new(name, label, BootstrapTextInput::new())
        .validator(UrlValidator::new())
        .attr("placeholder", "https://");

    if required {
        field = field.required().validator(RequiredValidator::new());
    }

    field
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_field() {
        let field = CharField("username", "Username", 150, true);
        assert_eq!(field.name, "username");
        assert!(field.required);
        assert_eq!(field.validators.len(), 2); // MaxLength + Required
    }

    #[test]
    fn test_email_field() {
        let field = EmailField("email", "Email Address", true);
        assert_eq!(field.name, "email");
        assert!(field.required);
    }

    #[test]
    fn test_password_field() {
        let field = PasswordField("password", "Password", Some(8));
        assert!(field.required);
        assert_eq!(field.validators.len(), 2); // Required + MinLength
    }
}
