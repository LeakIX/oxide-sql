//! List view template.

use ironhtml::typed::Element;
use ironhtml_elements::{
    Button, Div, Form, Input, Li, Nav, Option_ as OptEl, Select, Span, Table, Tbody, Td, Th, Thead,
    Tr, Ul, A, I,
};

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

    let total_str = ctx.total_items.to_string();

    Element::<Div>::new()
        .child::<Div, _>(|d| {
            d.class(
                "d-flex justify-content-between \
                 align-items-center mb-3",
            )
            .child::<Div, _>(|d| d.class("d-flex gap-2").raw(&search_bar))
            .child::<A, _>(|a| {
                a.attr("href", &ctx.add_url)
                    .class("btn btn-primary")
                    .child::<I, _>(|i| i.class("bi bi-plus-lg me-1"))
                    .text("Add ")
                    .text(&ctx.model_name)
            })
        })
        .child::<Div, _>(|row| {
            let row = row.class("row");
            let row = if has_filters {
                row.child::<Div, _>(|d| d.class("col-md-3").raw(&filters_sidebar))
            } else {
                row
            };
            row.child::<Div, _>(|d| {
                d.class(content_class).child::<Div, _>(|card| {
                    card.class("card")
                        .child::<Div, _>(|ch| {
                            ch.class(
                                "card-header bg-white d-flex \
                                 justify-content-between \
                                 align-items-center",
                            )
                            .child::<Span, _>(|s| {
                                s.class("fw-semibold")
                                    .text(&total_str)
                                    .text(" ")
                                    .text(&ctx.model_verbose_name)
                            })
                            .raw(&actions_bar)
                        })
                        .child::<Div, _>(|cb| cb.class("card-body p-0").raw(&table))
                        .child::<Div, _>(|cf| cf.class("card-footer bg-white").raw(&pagination))
                })
            })
        })
        .render()
}

fn render_search_bar(ctx: &ListViewContext) -> String {
    if ctx.search_fields.is_empty() {
        return String::new();
    }

    let value = ctx.search_query.as_deref().unwrap_or("");

    Element::<Form>::new()
        .attr("method", "get")
        .class("d-flex")
        .child::<Div, _>(|d| {
            d.class("input-group")
                .child::<Input, _>(|i| {
                    i.attr("type", "text")
                        .attr("name", "q")
                        .class("form-control")
                        .attr("placeholder", "Search...")
                        .attr("value", value)
                })
                .child::<Button, _>(|b| {
                    b.class("btn btn-outline-secondary")
                        .attr("type", "submit")
                        .child::<I, _>(|i| i.class("bi bi-search"))
                })
        })
        .render()
}

fn render_filters(filters: &[ListFilter], active: &[(String, String)]) -> String {
    if filters.is_empty() {
        return String::new();
    }

    let mut html = String::new();
    for filter in filters {
        let active_value = active
            .iter()
            .find(|(k, _)| k == &filter.name)
            .map(|(_, v)| v.as_str());

        let all_class = if active_value.is_none() {
            "list-group-item list-group-item-action active"
        } else {
            "list-group-item list-group-item-action"
        };

        let el = Element::<Div>::new()
            .class("card mb-3")
            .child::<Div, _>(|d| d.class("card-header").text(&filter.label))
            .child::<Div, _>(|d| {
                let d = d
                    .class("list-group list-group-flush")
                    .child::<A, _>(|a| a.attr("href", "?").class(all_class).text("All"));
                d.children(filter.options.iter(), |opt, a: Element<A>| {
                    let (value, label) = opt;
                    let is_active = active_value == Some(value.as_str());
                    let class = if is_active {
                        "list-group-item \
                             list-group-item-action active"
                    } else {
                        "list-group-item \
                             list-group-item-action"
                    };
                    let href = format!("?{}={}", filter.name, value);
                    a.attr("href", &href).class(class).text(label.as_str())
                })
            });
        el.render_to(&mut html);
    }
    html
}

fn render_actions(actions: &[(String, String)]) -> String {
    if actions.is_empty() {
        return String::new();
    }

    Element::<Form>::new()
        .attr("method", "post")
        .attr("action", "")
        .class("bulk-actions")
        .child::<Select, _>(|s| {
            let s = s
                .attr("name", "action")
                .class("form-select form-select-sm")
                .attr("style", "width: auto;")
                .child::<OptEl, _>(|o| o.attr("value", "").text("---------"));
            s.children(actions.iter(), |action, o: Element<OptEl>| {
                let (value, label) = action;
                o.attr("value", value.as_str()).text(label.as_str())
            })
        })
        .child::<Button, _>(|b| {
            b.attr("type", "submit")
                .class("btn btn-sm btn-outline-secondary")
                .text("Go")
        })
        .render()
}

fn render_table(ctx: &ListViewContext) -> String {
    if ctx.rows.is_empty() {
        return Element::<Div>::new()
            .class("text-center text-muted py-5")
            .text("No items found.")
            .render();
    }

    Element::<Div>::new()
        .class("table-responsive")
        .child::<Table, _>(|t| {
            t.class("table table-striped table-hover mb-0")
                .child::<Thead, _>(|thead| {
                    thead.class("table-light").child::<Tr, _>(|tr| {
                        let tr = tr.child::<Th, _>(|th| {
                            th.class("text-center")
                                .attr("style", "width: 40px;")
                                .child::<Input, _>(|i| {
                                    i.attr("type", "checkbox").class(
                                        "form-check-input \
                                             select-all",
                                    )
                                })
                        });
                        let tr = tr.children(ctx.columns.iter(), |col, th: Element<Th>| {
                            th.text(col.as_str())
                        });
                        tr.child::<Th, _>(|th| th.attr("style", "width: 100px;").text("Actions"))
                    })
                })
                .child::<Tbody, _>(|tbody| {
                    tbody.children(ctx.rows.iter(), |row, tr: Element<Tr>| {
                        render_table_row(tr, row)
                    })
                })
        })
        .render()
}

fn render_table_row(tr: Element<Tr>, row: &ListRow) -> Element<Tr> {
    let tr = tr.child::<Td, _>(|td| {
        td.class("text-center").child::<Input, _>(|i| {
            i.attr("type", "checkbox")
                .class("form-check-input row-select")
                .attr("name", "selected")
                .attr("value", &row.pk)
        })
    });
    let tr = tr.children(
        row.cells.iter().enumerate(),
        |(i, cell), td: Element<Td>| {
            if i == 0 {
                td.child::<A, _>(|a| a.attr("href", &row.edit_url).text(cell.as_str()))
            } else {
                td.text(cell.as_str())
            }
        },
    );
    tr.child::<Td, _>(|td| {
        td.class("table-actions")
            .child::<A, _>(|a| {
                a.attr("href", &row.edit_url)
                    .class("btn btn-sm btn-outline-primary me-1")
                    .child::<I, _>(|i| i.class("bi bi-pencil"))
            })
            .child::<Form, _>(|f| {
                f.attr("action", &row.delete_url)
                    .attr("method", "post")
                    .class("d-inline delete-confirm")
                    .child::<Button, _>(|b| {
                        b.attr("type", "submit")
                            .class(
                                "btn btn-sm \
                                 btn-outline-danger",
                            )
                            .child::<I, _>(|i| i.class("bi bi-trash"))
                    })
            })
    })
}

fn render_pagination(ctx: &ListViewContext) -> String {
    if ctx.total_pages <= 1 {
        let total_str = ctx.total_items.to_string();
        return Element::<Div>::new()
            .class(
                "d-flex justify-content-between \
                 align-items-center",
            )
            .child::<Span, _>(|s| {
                s.class("text-muted")
                    .text("Showing ")
                    .text(&total_str)
                    .text(" items")
            })
            .render();
    }

    let start = (ctx.page - 1) * ctx.per_page + 1;
    let end = std::cmp::min(ctx.page * ctx.per_page, ctx.total_items);

    let showing_text = format!("Showing {}-{} of {} items", start, end, ctx.total_items);

    Element::<Div>::new()
        .class(
            "d-flex justify-content-between \
             align-items-center",
        )
        .child::<Span, _>(|s| s.class("text-muted").text(&showing_text))
        .child::<Nav, _>(|nav| {
            nav.child::<Ul, _>(|ul| {
                let mut ul = ul.class("pagination pagination-sm mb-0");

                // Previous button
                if ctx.page > 1 {
                    let prev_href = format!("?page={}", ctx.page - 1);
                    ul = ul.child::<Li, _>(|li| {
                        li.class("page-item").child::<A, _>(|a| {
                            a.class("page-link").attr("href", &prev_href).raw("&laquo;")
                        })
                    });
                } else {
                    ul = ul.child::<Li, _>(|li| {
                        li.class("page-item disabled")
                            .child::<Span, _>(|s| s.class("page-link").raw("&laquo;"))
                    });
                }

                // Page numbers
                for p in 1..=ctx.total_pages {
                    let p_str = p.to_string();
                    if p == ctx.page {
                        ul = ul.child::<Li, _>(|li| {
                            li.class("page-item active")
                                .child::<Span, _>(|s| s.class("page-link").text(&p_str))
                        });
                    } else if (p as isize - ctx.page as isize).abs() <= 2
                        || p == 1
                        || p == ctx.total_pages
                    {
                        let href = format!("?page={}", p);
                        ul = ul.child::<Li, _>(|li| {
                            li.class("page-item").child::<A, _>(|a| {
                                a.class("page-link").attr("href", &href).text(&p_str)
                            })
                        });
                    } else if (p as isize - ctx.page as isize).abs() == 3 {
                        ul = ul.child::<Li, _>(|li| {
                            li.class("page-item disabled")
                                .child::<Span, _>(|s| s.class("page-link").text("..."))
                        });
                    }
                }

                // Next button
                if ctx.page < ctx.total_pages {
                    let next_href = format!("?page={}", ctx.page + 1);
                    ul = ul.child::<Li, _>(|li| {
                        li.class("page-item").child::<A, _>(|a| {
                            a.class("page-link").attr("href", &next_href).raw("&raquo;")
                        })
                    });
                } else {
                    ul = ul.child::<Li, _>(|li| {
                        li.class("page-item disabled")
                            .child::<Span, _>(|s| s.class("page-link").raw("&raquo;"))
                    });
                }

                ul
            })
        })
        .render()
}
