//! Bootstrap 5 HTML templates for the admin interface.

mod base;
pub mod detail;
pub mod list;

pub use base::{render_base, AdminContext};
pub use detail::{render_detail_view, DetailViewContext, Fieldset, InlineFormset, InlineRow};
pub use list::{render_list_view, ListFilter, ListRow, ListViewContext};

/// Escapes HTML special characters.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
