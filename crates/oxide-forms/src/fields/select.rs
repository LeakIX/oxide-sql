//! Select field types.

use crate::form::FormFieldDef;
use crate::validation::RequiredValidator;
use crate::widgets::BootstrapSelect;

/// Creates a choice field (select/dropdown).
pub fn choice_field(
    name: &str,
    label: &str,
    choices: Vec<(&str, &str)>,
    required: bool,
) -> FormFieldDef {
    let widget = BootstrapSelect::new(
        choices
            .into_iter()
            .map(|(v, l)| (v.to_string(), l.to_string()))
            .collect::<Vec<_>>(),
    );

    let mut field = FormFieldDef::new(name, label, widget);

    if required {
        field = field.required().validator(RequiredValidator::new());
    }

    field
}

/// Creates a multiple choice field (multi-select).
pub fn multiple_choice_field(
    name: &str,
    label: &str,
    choices: Vec<(&str, &str)>,
    required: bool,
) -> FormFieldDef {
    let widget = BootstrapSelect::new(
        choices
            .into_iter()
            .map(|(v, l)| (v.to_string(), l.to_string()))
            .collect::<Vec<_>>(),
    )
    .no_blank();

    let mut field = FormFieldDef::new(name, label, widget).attr("multiple", "multiple");

    if required {
        field = field.required().validator(RequiredValidator::new());
    }

    field
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choice_field() {
        let choices = vec![("draft", "Draft"), ("published", "Published")];
        let field = choice_field("status", "Status", choices, true);
        assert_eq!(field.name, "status");
        assert!(field.required);
    }

    #[test]
    fn test_multiple_choice_field() {
        let choices = vec![("tag1", "Tag 1"), ("tag2", "Tag 2")];
        let field = multiple_choice_field("tags", "Tags", choices, false);
        assert_eq!(field.name, "tags");
        assert!(!field.required);
    }
}
