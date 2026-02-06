//! # oxide-admin
//!
//! A Django-like admin interface for Rust with Bootstrap 5 UI.
//!
//! This crate provides automatic CRUD interfaces for your database models,
//! similar to Django's admin site. It includes:
//!
//! - Automatic list views with pagination, search, and filtering
//! - Add/change forms with validation
//! - Delete confirmation
//! - Bulk actions
//! - Customizable field display and fieldsets
//! - Inline editing for related models
//!
//! ## Quick Start
//!
//! Run the blog admin example:
//!
//! ```bash
//! cargo run -p oxide-admin --example blog_admin
//! ```
//!
//! Then open <http://localhost:3000/admin/> and login with
//! `admin` / `admin123`.
//!
//! See the [`blog_admin`
//! example](https://github.com/LeakIX/oxide-sql/blob/main/crates/oxide-admin/examples/blog_admin.rs)
//! for full source code.
//!
//! ## ModelAdmin Options
//!
//! The `ModelAdmin` struct provides many configuration options:
//!
//! - `list_display` - Columns to show in the list view
//! - `list_display_links` - Columns that link to the edit page
//! - `list_filter` - Columns that can be filtered
//! - `search_fields` - Columns that are searchable
//! - `ordering` - Default sort order (prefix with `-` for descending)
//! - `list_per_page` - Items per page
//! - `fieldsets` - Group fields in the edit form
//! - `readonly_fields` - Fields that cannot be edited
//!
//! ## Actions
//!
//! Built-in bulk actions:
//!
//! - `DeleteSelectedAction` - Delete selected items
//! - `ActivateSelectedAction` - Mark items as active
//! - `DeactivateSelectedAction` - Mark items as inactive
//! - `ExportCsvAction` - Export to CSV
//!
//! Custom actions can be created by implementing the `Action` trait.
//!
//! ## Filters
//!
//! Built-in filters:
//!
//! - `BooleanFilter` - Yes/No filter
//! - `ChoicesFilter` - Filter with predefined choices
//! - `DateRangeFilter` - Filter by date range
//! - `RangeFilter` - Filter by numeric range
//! - `NullFilter` - Filter by null/not null
//!
//! ## Templates
//!
//! The admin uses Bootstrap 5 for styling and includes:
//!
//! - Responsive sidebar navigation
//! - Dark/light mode support
//! - Mobile-friendly design
//! - Accessible forms and tables

pub mod actions;
pub mod error;
pub mod filters;
pub mod options;
pub mod site;
pub mod templates;
pub mod views;

// Re-export main types
pub use actions::{
    ActivateSelectedAction, CustomAction, DeactivateSelectedAction, DeleteSelectedAction,
    ExportCsvAction,
};
pub use error::{AdminError, Result};
pub use filters::{BooleanFilter, ChoicesFilter, DateRangeFilter, Filter, NullFilter, RangeFilter};
pub use options::{Action, ActionResult, Fieldset, InlineAdmin, ModelAdmin};
pub use site::{AdminSite, ModelRegistration};
pub use templates::{
    render_base, render_detail_view, render_list_view, AdminContext, DetailViewContext,
    ListViewContext,
};
pub use views::{
    add_view, build_order_clause, build_search_clause, change_view, delete_success_view,
    delete_view, get_redirect_url, index_view, list_view, render_admin_page, AddViewData,
    ChangeViewData, DeleteResult, DeleteViewData, FieldsetData, FormResult, InlineData,
    InlineRowData, ListViewData, ListViewParams,
};
