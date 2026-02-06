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

## Built-in Actions

- `DeleteSelectedAction` - Delete selected items
- `ActivateSelectedAction` - Mark items as active
- `DeactivateSelectedAction` - Mark items as inactive
- `ExportCsvAction` - Export to CSV

## Filters

Built-in filter types:

- `BooleanFilter` - Yes/No dropdown
- `ChoicesFilter` - Predefined choices
- `DateRangeFilter` - Date range picker
- `RangeFilter` - Numeric range
- `NullFilter` - Filter by null/not null

## API Reference

See the [`oxide_admin` rustdoc](pathname:///oxide-sql/rustdoc/oxide_admin/) for the full API with
code examples for creating admin sites, fieldsets, custom actions, and filters.
