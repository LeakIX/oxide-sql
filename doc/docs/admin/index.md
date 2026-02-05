---
sidebar_position: 0
---

# Admin Interface

Oxide Admin provides a Django-like admin interface for your Rust applications.
It automatically generates CRUD interfaces for your database models.

## Quick Start

Run the blog admin example:

```bash
cargo run -p oxide-admin --example blog_admin
```

Then open http://localhost:3000/admin/ and login with `admin` / `admin123`.

## Features

- **Automatic List Views**: Pagination, search, and filtering
- **Add/Change Forms**: Automatic form generation with validation
- **Delete Confirmation**: Safe deletion with confirmation pages
- **Bulk Actions**: Select multiple items and apply actions
- **Customizable Fieldsets**: Group related fields together
- **Responsive Design**: Mobile-friendly interface

## Creating an Admin Site

```rust
use oxide_admin::{AdminSite, ModelAdmin, Fieldset};

let admin = AdminSite::new("Blog Admin")
    .register::<Post>(
        ModelAdmin::new()
            .list_display(&["id", "title", "status", "created_at"])
            .list_filter(&["status"])
            .search_fields(&["title", "content"])
            .fieldset(Fieldset::named("Content", &["title", "slug", "content"]))
            .fieldset(Fieldset::named("Publishing", &["status"]).collapse())
    )
    .register::<Comment>(
        ModelAdmin::new()
            .list_display(&["id", "author", "created_at"])
    );
```

## ModelAdmin Options

| Option | Description |
|--------|-------------|
| `list_display` | Columns shown in list view |
| `list_display_links` | Columns that link to edit page |
| `list_filter` | Columns available for filtering |
| `search_fields` | Columns searchable via search box |
| `ordering` | Default sort order (prefix `-` for descending) |
| `list_per_page` | Items per page (default: 25) |
| `fields` | Fields to show in detail view |
| `exclude` | Fields to exclude from detail view |
| `readonly_fields` | Fields that cannot be edited |
| `fieldsets` | Group fields with titles |
| `date_hierarchy` | Add date-based drill-down navigation |

## Fieldsets

Group related fields together:

```rust
use oxide_admin::Fieldset;

ModelAdmin::new()
    .fieldset(Fieldset::named("Content", &["title", "slug", "content"]))
    .fieldset(
        Fieldset::named("Publishing", &["status", "author_id"])
            .description("Control when and how this is published")
            .collapse()  // Collapsed by default
    )
```

## Built-in Actions

- `DeleteSelectedAction` - Delete selected items
- `ActivateSelectedAction` - Mark items as active
- `DeactivateSelectedAction` - Mark items as inactive
- `ExportCsvAction` - Export to CSV

## Custom Actions

```rust
use oxide_admin::{Action, ActionResult, CustomAction};

let custom_action = CustomAction::new(
    "publish_selected",
    "Publish selected posts",
    |pks| Box::pin(async move {
        ActionResult::success(
            format!("Published {} posts", pks.len()),
            pks.len()
        )
    })
);
```

## Filters

Built-in filter types:

- `BooleanFilter` - Yes/No dropdown
- `ChoicesFilter` - Predefined choices
- `DateRangeFilter` - Date range picker
- `RangeFilter` - Numeric range
- `NullFilter` - Filter by null/not null
