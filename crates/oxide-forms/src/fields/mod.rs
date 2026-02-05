//! Form field definitions.

mod checkbox;
mod hidden;
mod select;
mod text;

pub use checkbox::{boolean_field, checkbox_field};
pub use hidden::hidden_field;
pub use select::{choice_field, multiple_choice_field};
pub use text::{char_field, email_field, password_field, text_field, url_field};
