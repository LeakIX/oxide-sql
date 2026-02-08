//! Detail/edit view template.

use ironhtml::html;
use ironhtml::typed::Element;
use ironhtml_elements::{Div, Form, Li, Tbody, Td, Th, Tr, P};

/// Context for rendering a detail/edit view.
#[derive(Debug, Clone)]
pub struct DetailViewContext {
    /// Model name.
    pub model_name: String,
    /// Whether this is a new object (add) or existing (change).
    pub is_new: bool,
    /// Object string representation (for existing objects).
    pub object_str: Option<String>,
    /// Form HTML (rendered by oxide-forms).
    pub form_html: String,
    /// Fieldsets for grouping fields.
    pub fieldsets: Vec<Fieldset>,
    /// Inline formsets.
    pub inlines: Vec<InlineFormset>,
    /// URL to return to list view.
    pub list_url: String,
    /// Delete URL (for existing objects).
    pub delete_url: Option<String>,
    /// Form action URL.
    pub action_url: String,
    /// Validation errors.
    pub errors: Vec<String>,
}

/// A fieldset groups related fields together.
#[derive(Debug, Clone)]
pub struct Fieldset {
    /// Optional title.
    pub name: Option<String>,
    /// Field HTML for each field in this set.
    pub fields: Vec<String>,
    /// CSS classes (e.g., "collapse").
    pub classes: Vec<String>,
    /// Optional description.
    pub description: Option<String>,
}

/// An inline formset for related objects.
#[derive(Debug, Clone)]
pub struct InlineFormset {
    /// Related model name.
    pub model_name: String,
    /// Verbose name (plural).
    pub verbose_name: String,
    /// Column headers.
    pub columns: Vec<String>,
    /// Existing rows (edit forms).
    pub rows: Vec<InlineRow>,
    /// Number of empty forms to show.
    pub extra: usize,
    /// Empty form template.
    pub empty_form: String,
}

/// A row in an inline formset.
#[derive(Debug, Clone)]
pub struct InlineRow {
    /// Row ID.
    pub id: String,
    /// Field inputs for this row.
    pub fields: Vec<String>,
    /// Delete checkbox.
    pub delete_checkbox: String,
}

impl Default for DetailViewContext {
    fn default() -> Self {
        Self {
            model_name: String::new(),
            is_new: true,
            object_str: None,
            form_html: String::new(),
            fieldsets: Vec::new(),
            inlines: Vec::new(),
            list_url: String::new(),
            delete_url: None,
            action_url: String::new(),
            errors: Vec::new(),
        }
    }
}

/// Renders the detail view content.
pub fn render_detail_view(ctx: &DetailViewContext) -> String {
    let _title = if ctx.is_new {
        format!("Add {}", ctx.model_name)
    } else {
        format!(
            "Change {}",
            ctx.object_str.as_deref().unwrap_or(&ctx.model_name)
        )
    };

    let errors_html = render_errors(&ctx.errors);
    let fieldsets_html = render_fieldsets(&ctx.fieldsets, &ctx.form_html);
    let inlines_html = render_inlines(&ctx.inlines);
    let delete_button = render_delete_button(&ctx.delete_url);

    let actions_header = html! {
        div.class("card-header") {
            strong { "Actions" }
        }
    };

    Element::<Form>::new()
        .attr("method", "post")
        .attr("action", &ctx.action_url)
        .attr("enctype", "multipart/form-data")
        .child::<Div, _>(|d| d.raw(&errors_html))
        .child::<Div, _>(|d| {
            d.class("row")
                .child::<Div, _>(|d| d.class("col-lg-8").raw(&fieldsets_html).raw(&inlines_html))
                .child::<Div, _>(|d| {
                    d.class("col-lg-4").child::<Div, _>(|card| {
                        card.class("card sticky-top")
                            .attr("style", "top: 1rem;")
                            .raw(actions_header.render())
                            .child::<Div, _>(|cb| {
                                cb.class("card-body").child::<Div, _>(|g| {
                                    let g = g.class("d-grid gap-2");
                                    let g = render_action_buttons(g, &ctx.list_url);
                                    g.raw(&delete_button)
                                })
                            })
                    })
                })
        })
        .render()
}

fn render_action_buttons(wrapper: Element<Div>, list_url: &str) -> Element<Div> {
    let save_btn = html! {
        button.type_("submit")
            .name("_save")
            .class("btn btn-primary") {
            i.class("bi bi-check-lg me-1")
            "Save"
        }
    };
    let continue_btn = html! {
        button.type_("submit")
            .name("_continue")
            .class("btn btn-outline-primary") {
            i.class("bi bi-arrow-repeat me-1")
            "Save and continue editing"
        }
    };
    let add_btn = html! {
        button.type_("submit")
            .name("_addanother")
            .class("btn btn-outline-secondary") {
            i.class("bi bi-plus-lg me-1")
            "Save and add another"
        }
    };
    let cancel_link = html! {
        a.href(#list_url)
            .class("btn btn-outline-secondary") {
            i.class("bi bi-x-lg me-1")
            "Cancel"
        }
    };

    wrapper
        .raw(save_btn.render())
        .raw(continue_btn.render())
        .raw(add_btn.render())
        .raw(cancel_link.render())
}

fn render_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        return String::new();
    }

    let heading = html! {
        strong { "Please correct the errors below:" }
    };

    Element::<Div>::new()
        .class("alert alert-danger")
        .attr("role", "alert")
        .raw(heading.render())
        .child::<ironhtml_elements::Ul, _>(|ul| {
            ul.class("mb-0 mt-2")
                .children(errors.iter(), |e, li: Element<Li>| li.text(e.as_str()))
        })
        .render()
}

fn render_fieldsets(fieldsets: &[Fieldset], form_html: &str) -> String {
    if fieldsets.is_empty() {
        return Element::<Div>::new()
            .class("card mb-4")
            .child::<Div, _>(|d| d.class("card-body").raw(form_html))
            .render();
    }

    let mut html_out = String::new();
    for fieldset in fieldsets {
        let is_collapsed = fieldset.classes.contains(&"collapse".to_string());
        let collapse_id = fieldset
            .name
            .as_ref()
            .map(|n| n.to_lowercase().replace(' ', "_"))
            .unwrap_or_else(|| "fieldset".to_string());

        let fields_html = fieldset.fields.join("\n");
        let desc = fieldset.description.as_deref();

        let el = if is_collapsed {
            let cid = collapse_id.clone();
            let target = format!("#{collapse_id}");
            let name_text = fieldset.name.as_deref().unwrap_or("");
            let header_span = html! {
                span { #name_text }
            };
            let collapse_btn = html! {
                button.class("btn btn-link btn-sm")
                    .type_("button")
                    .data_bs_toggle("collapse")
                    .data_bs_target(#target) {
                    i.class("bi bi-chevron-down")
                }
            };
            Element::<Div>::new()
                .class("card mb-4")
                .child::<Div, _>(|d| {
                    d.class(
                        "card-header d-flex \
                         justify-content-between \
                         align-items-center",
                    )
                    .raw(header_span.render())
                    .raw(collapse_btn.render())
                })
                .child::<Div, _>(|d| {
                    d.id(&cid).class("collapse").child::<Div, _>(|d| {
                        let d = d.class("card-body");
                        let d = if let Some(desc) = desc {
                            d.child::<P, _>(|p| p.class("text-muted mb-3").text(desc))
                        } else {
                            d
                        };
                        d.raw(&fields_html)
                    })
                })
        } else {
            Element::<Div>::new()
                .class("card mb-4")
                .when(fieldset.name.is_some(), |d| {
                    d.child::<Div, _>(|d| {
                        d.class("card-header")
                            .text(fieldset.name.as_deref().unwrap_or(""))
                    })
                })
                .child::<Div, _>(|d| {
                    let d = d.class("card-body");
                    let d = if let Some(desc) = desc {
                        d.child::<P, _>(|p| p.class("text-muted mb-3").text(desc))
                    } else {
                        d
                    };
                    d.raw(&fields_html)
                })
        };
        el.render_to(&mut html_out);
    }
    html_out
}

fn render_inlines(inlines: &[InlineFormset]) -> String {
    use ironhtml_elements::{Table, Tbody, Thead};

    let mut html_out = String::new();
    for inline in inlines {
        let verbose_name = &inline.verbose_name;
        let header = html! {
            div.class(
                "card-header d-flex \
                 justify-content-between \
                 align-items-center"
            ) {
                span {
                    strong { #verbose_name }
                }
                button.type_("button")
                    .class(
                        "btn btn-sm \
                         btn-outline-primary \
                         add-inline-row"
                    ) {
                    i.class("bi bi-plus me-1")
                    "Add another"
                }
            }
        };

        let el = Element::<Div>::new()
            .class("card mb-4")
            .raw(header.render())
            .child::<Div, _>(|d| {
                d.class("card-body p-0").child::<Div, _>(|d| {
                    d.class("table-responsive").child::<Table, _>(|t| {
                        t.class("table table-sm mb-0")
                            .child::<Thead, _>(|thead| {
                                thead.class("table-light").child::<Tr, _>(|tr| {
                                    let tr = tr
                                        .children(inline.columns.iter(), |col, th: Element<Th>| {
                                            th.text(col.as_str())
                                        });
                                    tr.child::<Th, _>(|th| {
                                        th.attr("style", "width: 60px;").text("Delete")
                                    })
                                })
                            })
                            .child::<Tbody, _>(|tbody| {
                                let tbody =
                                    tbody.children(inline.rows.iter(), |row, tr: Element<Tr>| {
                                        let tr = tr.data("inline-row", &row.id);
                                        let tr = tr.children(
                                            row.fields.iter(),
                                            |field, td: Element<Td>| td.raw(field.as_str()),
                                        );
                                        tr.child::<Td, _>(|td| {
                                            td.class("text-center").raw(&row.delete_checkbox)
                                        })
                                    });
                                render_empty_rows(tbody, inline)
                            })
                    })
                })
            });
        el.render_to(&mut html_out);
    }
    html_out
}

fn render_empty_rows(mut tbody: Element<Tbody>, inline: &InlineFormset) -> Element<Tbody> {
    for i in 0..inline.extra {
        let idx = i.to_string();
        let remove_btn = html! {
            button.type_("button")
                .class(
                    "btn btn-sm \
                     btn-outline-danger \
                     remove-inline-row"
                ) {
                i.class("bi bi-x")
            }
        };
        let remove_html = remove_btn.render();
        tbody = tbody.child::<Tr, _>(|tr: Element<Tr>| {
            tr.data("inline-row", "__prefix__")
                .data("inline-index", &idx)
                .child::<Td, _>(|td: Element<Td>| td.raw(&inline.empty_form))
                .child::<Td, _>(|td: Element<Td>| td.class("text-center").raw(&remove_html))
        });
    }
    tbody
}

fn render_delete_button(delete_url: &Option<String>) -> String {
    match delete_url {
        Some(url) => {
            let mut out = html! { hr }.render();
            let link = html! {
                a.href(#url)
                    .class(
                        "btn btn-outline-danger w-100"
                    ) {
                    i.class("bi bi-trash me-1")
                    "Delete"
                }
            };
            link.render_to(&mut out);
            out
        }
        None => String::new(),
    }
}
