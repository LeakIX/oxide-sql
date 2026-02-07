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
//! ```rust
//! use oxide_forms::{
//!     FormFieldDef, FormBuilder, ValidationErrors,
//!     render_bootstrap_form,
//! };
//! use oxide_forms::widgets::BootstrapTextInput;
//! use oxide_forms::validation::RequiredValidator;
//! use std::collections::HashMap;
//!
//! // Define form fields
//! let fields = FormBuilder::new()
//!     .field(
//!         FormFieldDef::new(
//!             "username", "Username",
//!             BootstrapTextInput::new(),
//!         )
//!         .required()
//!         .validator(RequiredValidator::new())
//!         .help_text("Choose a unique username")
//!     )
//!     .field(
//!         FormFieldDef::new(
//!             "email", "Email",
//!             BootstrapTextInput::email(),
//!         )
//!         .required()
//!     )
//!     .build();
//!
//! // Render as Bootstrap 5 form
//! let values = HashMap::new();
//! let errors = ValidationErrors::new();
//! let html = render_bootstrap_form(
//!     &fields, &values, &errors, "/submit", "POST",
//! );
//! ```
//!
//! ## Using Field Helpers
//!
//! ```rust
//! use oxide_forms::fields::{
//!     char_field, email_field, password_field, choice_field,
//! };
//!
//! let fields = vec![
//!     char_field("username", "Username", 150, true),
//!     email_field("email", "Email", true),
//!     password_field("password", "Password", Some(8)),
//!     choice_field("role", "Role", vec![
//!         ("user", "User"),
//!         ("admin", "Administrator"),
//!     ], true),
//! ];
//! ```
//!
//! ## Validation
//!
//! ```rust
//! use oxide_forms::FormFieldDef;
//! use oxide_forms::ValidationErrors;
//! use oxide_forms::widgets::BootstrapTextInput;
//! use oxide_forms::validation::{
//!     RequiredValidator, EmailValidator, Validator,
//! };
//!
//! let field = FormFieldDef::new(
//!     "email", "Email", BootstrapTextInput::email(),
//! )
//! .validator(RequiredValidator::new())
//! .validator(EmailValidator::new());
//!
//! // Validate a value
//! let value = "user@example.com";
//! let mut errors = ValidationErrors::new();
//! for validator in &field.validators {
//!     if let Err(msg) = validator.validate(value) {
//!         errors.add(&field.name, msg);
//!     }
//! }
//! assert!(errors.is_empty());
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
