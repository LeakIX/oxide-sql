//! List view template.

use ironhtml::html;
use ironhtml::typed::Element;
use ironhtml_elements::{
    Div, Form, Li, Nav, Option_ as OptEl, Select, Table, Tbody, Td, Th, Thead, Tr, Ul, A,
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
    let verbose = &ctx.model_verbose_name;
    let model_name = &ctx.model_name;
    let add_url = &ctx.add_url;

    let add_link = html! {
        a.class("btn btn-primary").href(#add_url) {
            i.class("bi bi-plus-lg me-1")
            "Add "
            #model_name
        }
    };

    let header_span = html! {
        span.class("fw-semibold") {
            #total_str
            " "
            #verbose
        }
    };

    Element::<Div>::new()
        .child::<Div, _>(|d| {
            d.class(
                "d-flex justify-content-between \
                 align-items-center mb-3",
            )
            .child::<Div, _>(|d| d.class("d-flex gap-2").raw(&search_bar))
            .raw(add_link.render())
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
                            .raw(header_span.render())
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

    html! {
        form.method("get").class("d-flex") {
            div.class("input-group") {
                input.type_("text")
                    .name("q")
                    .class("form-control")
                    .placeholder("Search...")
                    .value(#value)
                button.class("btn btn-outline-secondary")
                    .type_("submit") {
                    i.class("bi bi-search")
                }
            }
        }
    }
    .render()
}

fn render_filters(filters: &[ListFilter], active: &[(String, String)]) -> String {
    if filters.is_empty() {
        return String::new();
    }

    let mut out = String::new();
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

        let all_link = html! {
            a.href("?").class(#all_class) { "All" }
        };

        let el = Element::<Div>::new()
            .class("card mb-3")
            .child::<Div, _>(|d| d.class("card-header").text(&filter.label))
            .child::<Div, _>(|d| {
                let d = d
                    .class("list-group list-group-flush")
                    .raw(all_link.render());
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
        el.render_to(&mut out);
    }
    out
}

fn render_actions(actions: &[(String, String)]) -> String {
    if actions.is_empty() {
        return String::new();
    }

    let go_btn = html! {
        button.type_("submit")
            .class("btn btn-sm btn-outline-secondary") {
            "Go"
        }
    };

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
        .child::<Div, _>(|d| d.raw(go_btn.render()))
        .render()
}

fn render_table(ctx: &ListViewContext) -> String {
    if ctx.rows.is_empty() {
        return html! {
            div.class("text-center text-muted py-5") {
                "No items found."
            }
        }
        .render();
    }

    Element::<Div>::new()
        .class("table-responsive")
        .child::<Table, _>(|t| {
            t.class("table table-striped table-hover mb-0")
                .child::<Thead, _>(|thead| {
                    thead.class("table-light").child::<Tr, _>(|tr| {
                        let checkbox = html! {
                            input.type_("checkbox")
                                .class("form-check-input select-all")
                        };
                        let tr = tr.child::<Th, _>(|th| {
                            th.class("text-center")
                                .attr("style", "width: 40px;")
                                .raw(checkbox.render())
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
    let checkbox = html! {
        input.type_("checkbox")
            .class("form-check-input row-select")
            .name("selected")
            .value(#&row.pk)
    };
    let tr = tr.child::<Td, _>(|td| td.class("text-center").raw(checkbox.render()));
    let tr = tr.children(
        row.cells.iter().enumerate(),
        |(i, cell), td: Element<Td>| {
            if i == 0 {
                let edit_url = &row.edit_url;
                let link = html! {
                    a.href(#edit_url) { #cell }
                };
                td.raw(link.render())
            } else {
                td.text(cell.as_str())
            }
        },
    );
    let edit_url = &row.edit_url;
    let delete_url = &row.delete_url;

    let edit_btn = html! {
        a.href(#edit_url)
            .class("btn btn-sm btn-outline-primary me-1") {
            i.class("bi bi-pencil")
        }
    };

    let delete_form = html! {
        form.action(#delete_url)
            .method("post")
            .class("d-inline delete-confirm") {
            button.type_("submit")
                .class("btn btn-sm btn-outline-danger") {
                i.class("bi bi-trash")
            }
        }
    };

    tr.child::<Td, _>(|td| {
        td.class("table-actions")
            .raw(edit_btn.render())
            .raw(delete_form.render())
    })
}

fn render_pagination(ctx: &ListViewContext) -> String {
    if ctx.total_pages <= 1 {
        let total_str = ctx.total_items.to_string();
        let showing = html! {
            span.class("text-muted") {
                "Showing "
                #total_str
                " items"
            }
        };
        let showing_html = showing.render();
        return html! {
            div.class(
                "d-flex justify-content-between align-items-center"
            ) {
                #showing_html
            }
        }
        .render();
    }

    let start = (ctx.page - 1) * ctx.per_page + 1;
    let end = std::cmp::min(ctx.page * ctx.per_page, ctx.total_items);

    let showing_text = format!("Showing {}-{} of {} items", start, end, ctx.total_items);

    let showing_span = html! {
        span.class("text-muted") { #showing_text }
    };

    Element::<Div>::new()
        .class(
            "d-flex justify-content-between \
             align-items-center",
        )
        .raw(showing_span.render())
        .child::<Nav, _>(|nav| {
            nav.child::<Ul, _>(|ul| {
                let mut ul = ul.class("pagination pagination-sm mb-0");

                // Previous button
                if ctx.page > 1 {
                    let prev_href = format!("?page={}", ctx.page - 1);
                    let link = html! {
                        a.class("page-link").href(#prev_href) {
                            "&laquo;"
                        }
                    };
                    ul = ul.child::<Li, _>(|li| li.class("page-item").raw(link.render()));
                } else {
                    let disabled = html! {
                        span.class("page-link") { "&laquo;" }
                    };
                    ul = ul
                        .child::<Li, _>(|li| li.class("page-item disabled").raw(disabled.render()));
                }

                // Page numbers
                for p in 1..=ctx.total_pages {
                    let p_str = p.to_string();
                    if p == ctx.page {
                        let active_span = html! {
                            span.class("page-link") { #p_str }
                        };
                        ul = ul.child::<Li, _>(|li| {
                            li.class("page-item active").raw(active_span.render())
                        });
                    } else if (p as isize - ctx.page as isize).abs() <= 2
                        || p == 1
                        || p == ctx.total_pages
                    {
                        let href = format!("?page={}", p);
                        let link = html! {
                            a.class("page-link").href(#href) {
                                #p_str
                            }
                        };
                        ul = ul.child::<Li, _>(|li| li.class("page-item").raw(link.render()));
                    } else if (p as isize - ctx.page as isize).abs() == 3 {
                        let dots = html! {
                            span.class("page-link") { "..." }
                        };
                        ul = ul
                            .child::<Li, _>(|li| li.class("page-item disabled").raw(dots.render()));
                    }
                }

                // Next button
                if ctx.page < ctx.total_pages {
                    let next_href = format!("?page={}", ctx.page + 1);
                    let link = html! {
                        a.class("page-link").href(#next_href) {
                            "&raquo;"
                        }
                    };
                    ul = ul.child::<Li, _>(|li| li.class("page-item").raw(link.render()));
                } else {
                    let disabled = html! {
                        span.class("page-link") { "&raquo;" }
                    };
                    ul = ul
                        .child::<Li, _>(|li| li.class("page-item disabled").raw(disabled.render()));
                }

                ul
            })
        })
        .render()
}
