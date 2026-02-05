//! List view template.

use super::html_escape;

/// Context for rendering a list view.
#[derive(Debug, Clone)]
pub struct ListViewContext {
    /// Model name.
    pub model_name: String,
    /// Model verbose name (plural).
    pub model_verbose_name: String,
    /// Column headers.
    pub columns: Vec<String>,
    /// Rows of data (each row is a list of cell values).
    pub rows: Vec<ListRow>,
    /// URL for adding a new item.
    pub add_url: String,
    /// Search query (if any).
    pub search_query: Option<String>,
    /// Search fields (if search is enabled).
    pub search_fields: Vec<String>,
    /// Available filters.
    pub filters: Vec<ListFilter>,
    /// Active filter values.
    pub active_filters: Vec<(String, String)>,
    /// Available bulk actions.
    pub actions: Vec<(String, String)>,
    /// Current page number.
    pub page: usize,
    /// Total number of pages.
    pub total_pages: usize,
    /// Total number of items.
    pub total_items: usize,
    /// Items per page.
    pub per_page: usize,
}

/// A row in the list view.
#[derive(Debug, Clone)]
pub struct ListRow {
    /// Primary key value.
    pub pk: String,
    /// Cell values.
    pub cells: Vec<String>,
    /// URL for editing this item.
    pub edit_url: String,
    /// URL for deleting this item.
    pub delete_url: String,
}

/// A filter option for the list view.
#[derive(Debug, Clone)]
pub struct ListFilter {
    /// Filter field name.
    pub name: String,
    /// Filter display label.
    pub label: String,
    /// Filter options (value, label).
    pub options: Vec<(String, String)>,
}

impl Default for ListViewContext {
    fn default() -> Self {
        Self {
            model_name: String::new(),
            model_verbose_name: String::new(),
            columns: Vec::new(),
            rows: Vec::new(),
            add_url: String::new(),
            search_query: None,
            search_fields: Vec::new(),
            filters: Vec::new(),
            active_filters: Vec::new(),
            actions: vec![("delete_selected".to_string(), "Delete selected".to_string())],
            page: 1,
            total_pages: 1,
            total_items: 0,
            per_page: 25,
        }
    }
}

/// Renders the list view content.
pub fn render_list_view(ctx: &ListViewContext) -> String {
    let search_bar = render_search_bar(ctx);
    let filters_sidebar = render_filters(&ctx.filters, &ctx.active_filters);
    let actions_bar = render_actions(&ctx.actions);
    let table = render_table(ctx);
    let pagination = render_pagination(ctx);

    let has_filters = !ctx.filters.is_empty();
    let content_class = if has_filters { "col-md-9" } else { "col-12" };

    format!(
        r#"<div class="d-flex justify-content-between align-items-center mb-3">
    <div class="d-flex gap-2">
        {search_bar}
    </div>
    <a href="{add_url}" class="btn btn-primary">
        <i class="bi bi-plus-lg me-1"></i>Add {model_name}
    </a>
</div>

<div class="row">
    {filters_col}
    <div class="{content_class}">
        <div class="card">
            <div class="card-header bg-white d-flex justify-content-between align-items-center">
                <span class="fw-semibold">{total_items} {model_verbose_name}</span>
                {actions_bar}
            </div>
            <div class="card-body p-0">
                {table}
            </div>
            <div class="card-footer bg-white">
                {pagination}
            </div>
        </div>
    </div>
</div>"#,
        search_bar = search_bar,
        add_url = html_escape(&ctx.add_url),
        model_name = html_escape(&ctx.model_name),
        model_verbose_name = html_escape(&ctx.model_verbose_name),
        total_items = ctx.total_items,
        actions_bar = actions_bar,
        table = table,
        pagination = pagination,
        content_class = content_class,
        filters_col = if has_filters {
            format!(r#"<div class="col-md-3">{}</div>"#, filters_sidebar)
        } else {
            String::new()
        },
    )
}

fn render_search_bar(ctx: &ListViewContext) -> String {
    if ctx.search_fields.is_empty() {
        return String::new();
    }

    let value = ctx.search_query.as_deref().unwrap_or("");

    format!(
        r#"<form method="get" class="d-flex">
            <div class="input-group">
                <input type="text" name="q" class="form-control" placeholder="Search..." value="{value}">
                <button class="btn btn-outline-secondary" type="submit">
                    <i class="bi bi-search"></i>
                </button>
            </div>
        </form>"#,
        value = html_escape(value)
    )
}

fn render_filters(filters: &[ListFilter], active: &[(String, String)]) -> String {
    if filters.is_empty() {
        return String::new();
    }

    let filter_groups: Vec<String> = filters
        .iter()
        .map(|filter| {
            let active_value = active
                .iter()
                .find(|(k, _)| k == &filter.name)
                .map(|(_, v)| v.as_str());

            let options: Vec<String> = filter
                .options
                .iter()
                .map(|(value, label)| {
                    let is_active = active_value == Some(value.as_str());
                    let class = if is_active {
                        "list-group-item list-group-item-action active"
                    } else {
                        "list-group-item list-group-item-action"
                    };
                    format!(
                        r#"<a href="?{name}={value}" class="{class}">{label}</a>"#,
                        name = html_escape(&filter.name),
                        value = html_escape(value),
                        class = class,
                        label = html_escape(label)
                    )
                })
                .collect();

            format!(
                r#"<div class="card mb-3">
                    <div class="card-header">{label}</div>
                    <div class="list-group list-group-flush">
                        <a href="?" class="list-group-item list-group-item-action{all_class}">All</a>
                        {options}
                    </div>
                </div>"#,
                label = html_escape(&filter.label),
                options = options.join("\n"),
                all_class = if active_value.is_none() { " active" } else { "" }
            )
        })
        .collect();

    filter_groups.join("\n")
}

fn render_actions(actions: &[(String, String)]) -> String {
    if actions.is_empty() {
        return String::new();
    }

    let options: Vec<String> = actions
        .iter()
        .map(|(value, label)| {
            format!(
                r#"<option value="{}">{}</option>"#,
                html_escape(value),
                html_escape(label)
            )
        })
        .collect();

    format!(
        r#"<form method="post" action="" class="bulk-actions">
            <select name="action" class="form-select form-select-sm" style="width: auto;">
                <option value="">---------</option>
                {options}
            </select>
            <button type="submit" class="btn btn-sm btn-outline-secondary">Go</button>
        </form>"#,
        options = options.join("\n")
    )
}

fn render_table(ctx: &ListViewContext) -> String {
    let headers: Vec<String> = ctx
        .columns
        .iter()
        .map(|col| format!("<th>{}</th>", html_escape(col)))
        .collect();

    let rows: Vec<String> = ctx
        .rows
        .iter()
        .map(|row| {
            let cells: Vec<String> = row
                .cells
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    if i == 0 {
                        // First column is a link to edit
                        format!(
                            r#"<td><a href="{}">{}</a></td>"#,
                            html_escape(&row.edit_url),
                            html_escape(cell)
                        )
                    } else {
                        format!("<td>{}</td>", html_escape(cell))
                    }
                })
                .collect();

            format!(
                r#"<tr>
                    <td class="text-center">
                        <input type="checkbox" class="form-check-input row-select" name="selected" value="{}">
                    </td>
                    {}
                    <td class="table-actions">
                        <a href="{}" class="btn btn-sm btn-outline-primary me-1">
                            <i class="bi bi-pencil"></i>
                        </a>
                        <form action="{}" method="post" class="d-inline delete-confirm">
                            <button type="submit" class="btn btn-sm btn-outline-danger">
                                <i class="bi bi-trash"></i>
                            </button>
                        </form>
                    </td>
                </tr>"#,
                html_escape(&row.pk),
                cells.join("\n"),
                html_escape(&row.edit_url),
                html_escape(&row.delete_url)
            )
        })
        .collect();

    if rows.is_empty() {
        return r#"<div class="text-center text-muted py-5">No items found.</div>"#.to_string();
    }

    format!(
        r#"<div class="table-responsive">
            <table class="table table-striped table-hover mb-0">
                <thead class="table-light">
                    <tr>
                        <th class="text-center" style="width: 40px;">
                            <input type="checkbox" class="form-check-input select-all">
                        </th>
                        {headers}
                        <th style="width: 100px;">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {rows}
                </tbody>
            </table>
        </div>"#,
        headers = headers.join("\n"),
        rows = rows.join("\n")
    )
}

fn render_pagination(ctx: &ListViewContext) -> String {
    if ctx.total_pages <= 1 {
        return format!(
            r#"<div class="d-flex justify-content-between align-items-center">
                <span class="text-muted">Showing {} items</span>
            </div>"#,
            ctx.total_items
        );
    }

    let start = (ctx.page - 1) * ctx.per_page + 1;
    let end = std::cmp::min(ctx.page * ctx.per_page, ctx.total_items);

    let mut pages = Vec::new();

    // Previous button
    if ctx.page > 1 {
        pages.push(format!(
            r#"<li class="page-item"><a class="page-link" href="?page={}">&laquo;</a></li>"#,
            ctx.page - 1
        ));
    } else {
        pages.push(
            r#"<li class="page-item disabled"><span class="page-link">&laquo;</span></li>"#
                .to_string(),
        );
    }

    // Page numbers
    for p in 1..=ctx.total_pages {
        if p == ctx.page {
            pages.push(format!(
                r#"<li class="page-item active"><span class="page-link">{}</span></li>"#,
                p
            ));
        } else if (p as isize - ctx.page as isize).abs() <= 2 || p == 1 || p == ctx.total_pages {
            pages.push(format!(
                r#"<li class="page-item"><a class="page-link" href="?page={}">{}</a></li>"#,
                p, p
            ));
        } else if (p as isize - ctx.page as isize).abs() == 3 {
            pages.push(
                r#"<li class="page-item disabled"><span class="page-link">...</span></li>"#
                    .to_string(),
            );
        }
    }

    // Next button
    if ctx.page < ctx.total_pages {
        pages.push(format!(
            r#"<li class="page-item"><a class="page-link" href="?page={}">&raquo;</a></li>"#,
            ctx.page + 1
        ));
    } else {
        pages.push(
            r#"<li class="page-item disabled"><span class="page-link">&raquo;</span></li>"#
                .to_string(),
        );
    }

    format!(
        r#"<div class="d-flex justify-content-between align-items-center">
            <span class="text-muted">Showing {start}-{end} of {total} items</span>
            <nav>
                <ul class="pagination pagination-sm mb-0">
                    {pages}
                </ul>
            </nav>
        </div>"#,
        start = start,
        end = end,
        total = ctx.total_items,
        pages = pages.join("\n")
    )
}
