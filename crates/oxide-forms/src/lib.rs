//! # oxide-forms
//!
//! Django-like form generation and validation with Bootstrap 5 widgets.
//!
//! This crate provides:
//! - Form field definitions with validation
//! - Bootstrap 5 form widgets
//! - Form rendering helpers
//! - Validation error handling
//!
//! ## Quick Start
//!
//! ```ignore
//! use oxide_forms::{FormFieldDef, FormBuilder, render_bootstrap_form};
//! use oxide_forms::widgets::BootstrapTextInput;
//! use oxide_forms::validation::RequiredValidator;
//!
//! // Define form fields
//! let fields = FormBuilder::new()
//!     .field(
//!         FormFieldDef::new("username", "Username", BootstrapTextInput::new())
//!             .required()
//!             .validator(RequiredValidator::new())
//!             .help_text("Choose a unique username")
//!     )
//!     .field(
//!         FormFieldDef::new("email", "Email", BootstrapTextInput::email())
//!             .required()
//!     )
//!     .build();
//!
//! // Render as Bootstrap 5 form
//! let html = render_bootstrap_form(&fields, &values, &errors, "/submit", "POST");
//! ```
//!
//! ## Using Field Helpers
//!
//! ```ignore
//! use oxide_forms::fields::{CharField, EmailField, PasswordField, ChoiceField};
//!
//! let fields = vec![
//!     CharField("username", "Username", 150, true),
//!     EmailField("email", "Email", true),
//!     PasswordField("password", "Password", Some(8)),
//!     ChoiceField("role", "Role", vec![
//!         ("user", "User"),
//!         ("admin", "Administrator"),
//!     ], true),
//! ];
//! ```
//!
//! ## Validation
//!
//! ```ignore
//! use oxide_forms::validation::{RequiredValidator, MaxLengthValidator, EmailValidator};
//!
//! let field = FormFieldDef::new("email", "Email", BootstrapTextInput::email())
//!     .validator(RequiredValidator::new())
//!     .validator(EmailValidator::new());
//!
//! // Validate a value
//! for validator in &field.validators {
//!     if let Err(msg) = validator.validate(value) {
//!         errors.add(&field.name, msg);
//!     }
//! }
//! ```
//!
//! ## Widgets
//!
//! Available Bootstrap 5 widgets:
//! - `BootstrapTextInput` - Text, email, password, number inputs
//! - `BootstrapTextarea` - Multi-line text input
//! - `BootstrapSelect` - Dropdown select
//! - `BootstrapCheckbox` - Checkbox with optional switch style
//! - `BootstrapRadioSelect` - Radio button group

mod error;
pub mod fields;
mod form;
pub mod validation;
pub mod widgets;

pub use error::{FormError, Result, ValidationErrors};
pub use form::{render_bootstrap_field, render_bootstrap_form, Form, FormBuilder, FormFieldDef};
