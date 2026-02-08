//! Base layout template.

use ironhtml::typed::{Document, Element};
use ironhtml_elements::{
    Body, Button, Div, Head, Hr, Html, Li, Link, Meta, Nav, Ol, Script, Style, Title, Ul, A, H2,
    H5, I,
};

/// Context for rendering the admin layout.
#[derive(Debug, Clone)]
pub struct AdminContext {
    /// Site title.
    pub site_title: String,
    /// Site header.
    pub site_header: String,
    /// Current user name (if logged in).
    pub user_name: Option<String>,
    /// List of registered models (name, url).
    pub models: Vec<(String, String)>,
    /// Breadcrumbs (label, url).
    pub breadcrumbs: Vec<(String, Option<String>)>,
    /// Page title.
    pub page_title: String,
    /// Main content HTML.
    pub content: String,
    /// Flash messages (type, message).
    pub messages: Vec<(String, String)>,
}

impl Default for AdminContext {
    fn default() -> Self {
        Self {
            site_title: "Oxide Admin".to_string(),
            site_header: "Administration".to_string(),
            user_name: None,
            models: Vec::new(),
            breadcrumbs: vec![("Home".to_string(), Some("/admin/".to_string()))],
            page_title: "Dashboard".to_string(),
            content: String::new(),
            messages: Vec::new(),
        }
    }
}

const CSS: &str = "\
:root { --sidebar-width: 280px; }\n\
body { min-height: 100vh; }\n\
.sidebar { position: fixed; top: 0; left: 0; bottom: 0; \
width: var(--sidebar-width); background-color: #212529; \
padding-top: 0; z-index: 1000; overflow-y: auto; }\n\
.sidebar .nav-link { color: rgba(255,255,255,0.75); \
padding: 0.75rem 1rem; border-radius: 0; }\n\
.sidebar .nav-link:hover { color: rgba(255,255,255,1); \
background-color: rgba(255,255,255,0.1); }\n\
.sidebar .nav-link.active { color: #fff; \
background-color: #0d6efd; }\n\
.sidebar-header { padding: 1rem; \
background-color: #0d6efd; color: white; }\n\
.sidebar-header h5 { margin: 0; font-weight: 600; }\n\
.main-content { margin-left: var(--sidebar-width); \
min-height: 100vh; }\n\
.navbar { padding-left: 1.5rem; padding-right: 1.5rem; }\n\
.content-wrapper { padding: 1.5rem; }\n\
.table-actions { white-space: nowrap; }\n\
.bulk-actions { display: flex; gap: 0.5rem; \
align-items: center; }\n\
@media (max-width: 768px) {\n\
  .sidebar { transform: translateX(-100%); \
transition: transform 0.3s ease; }\n\
  .sidebar.show { transform: translateX(0); }\n\
  .main-content { margin-left: 0; }\n\
}";

const JS: &str = "\
document.querySelectorAll('.select-all').forEach(checkbox=>{\
checkbox.addEventListener('change',function(){\
const table=this.closest('table');\
table.querySelectorAll('.row-select').forEach(cb=>{\
cb.checked=this.checked;});});});\
document.querySelectorAll('.delete-confirm').forEach(form=>{\
form.addEventListener('submit',function(e){\
if(!confirm('Are you sure you want to delete this item?')){\
e.preventDefault();}});});";

/// Renders the base admin layout.
pub fn render_base(ctx: &AdminContext) -> String {
    let title_str = format!("{} | {}", ctx.page_title, ctx.site_title);

    Document::new()
        .doctype()
        .root::<Html, _>(|html| {
            html.attr("lang", "en")
                .data("bs-theme", "light")
                .child::<Head, _>(|head| {
                    head.child::<Meta, _>(|m| {
                        m.attr("charset", "UTF-8")
                    })
                    .child::<Meta, _>(|m| {
                        m.attr("name", "viewport").attr(
                            "content",
                            "width=device-width, \
                             initial-scale=1.0",
                        )
                    })
                    .child::<Title, _>(|t| t.text(&title_str))
                    .child::<Link, _>(|l| {
                        l.attr("href", "https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css")
                            .attr("rel", "stylesheet")
                    })
                    .child::<Link, _>(|l| {
                        l.attr("href", "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.1/font/bootstrap-icons.css")
                            .attr("rel", "stylesheet")
                    })
                    .child::<Style, _>(|s| s.raw(CSS))
                })
                .child::<Body, _>(|body| {
                    body.child::<Nav, _>(|nav| {
                        render_sidebar(nav, ctx)
                    })
                    .child::<Div, _>(|main| {
                        main.class("main-content")
                            .child::<Nav, _>(|n| {
                                render_topbar(n, ctx)
                            })
                            .child::<Div, _>(|cw| {
                                let cw = cw
                                    .class("content-wrapper");
                                let cw =
                                    render_messages_into(
                                        cw,
                                        &ctx.messages,
                                    );
                                cw.child::<H2, _>(|h| {
                                    h.class("mb-4")
                                        .text(&ctx.page_title)
                                })
                                .raw(&ctx.content)
                            })
                    })
                    .child::<Script, _>(|s| {
                        s.attr("src", "https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/js/bootstrap.bundle.min.js")
                    })
                    .child::<Script, _>(|s| s.raw(JS))
                })
        })
        .build()
}

fn render_sidebar(nav: Element<Nav>, ctx: &AdminContext) -> Element<Nav> {
    nav.class("sidebar")
        .child::<Div, _>(|d| {
            d.class("sidebar-header").child::<H5, _>(|h| {
                h.child::<I, _>(|i| i.class("bi bi-gear-fill me-2"))
                    .text(&ctx.site_header)
            })
        })
        .child::<Ul, _>(|ul| {
            let ul = ul
                .class("nav flex-column")
                .child::<Li, _>(|li| {
                    li.class("nav-item").child::<A, _>(|a| {
                        a.class("nav-link")
                            .attr("href", "/admin/")
                            .child::<I, _>(|i| i.class("bi bi-house-door me-2"))
                            .text("Dashboard")
                    })
                })
                .child::<Li, _>(|li| {
                    li.child::<Hr, _>(|hr| hr.class("my-2 mx-3 border-secondary"))
                });
            ul.children(ctx.models.iter(), |item, li: Element<Li>| {
                let (name, url) = item;
                li.class("nav-item").child::<A, _>(|a| {
                    a.class("nav-link")
                        .attr("href", url.as_str())
                        .child::<I, _>(|i| i.class("bi bi-table me-2"))
                        .text(name.as_str())
                })
            })
        })
}

fn render_topbar(nav: Element<Nav>, ctx: &AdminContext) -> Element<Nav> {
    nav.class(
        "navbar navbar-expand-lg navbar-light \
         bg-white border-bottom",
    )
    .child::<Div, _>(|d| {
        d.class("container-fluid")
            .child::<Button, _>(|b| {
                b.class("btn btn-link d-md-none")
                    .attr("type", "button")
                    .attr(
                        "onclick",
                        "document.querySelector('.sidebar')\
                         .classList.toggle('show')",
                    )
                    .child::<I, _>(|i| i.class("bi bi-list fs-4"))
            })
            .child::<Nav, _>(|n| render_breadcrumbs(n, &ctx.breadcrumbs))
            .child::<Div, _>(|d| render_user_menu(d.class("ms-auto"), &ctx.user_name))
    })
}

fn render_breadcrumbs(nav: Element<Nav>, breadcrumbs: &[(String, Option<String>)]) -> Element<Nav> {
    let last_idx = breadcrumbs.len().saturating_sub(1);
    nav.attr("aria-label", "breadcrumb").child::<Ol, _>(|ol| {
        let mut ol = ol.class("breadcrumb mb-0");
        for (i, (label, url)) in breadcrumbs.iter().enumerate() {
            let is_last = i == last_idx;
            ol = ol.child::<Li, _>(|li| {
                if is_last {
                    li.class("breadcrumb-item active")
                        .attr("aria-current", "page")
                        .text(label.as_str())
                } else if let Some(u) = url {
                    li.class("breadcrumb-item")
                        .child::<A, _>(|a| a.attr("href", u.as_str()).text(label.as_str()))
                } else {
                    li.class("breadcrumb-item").text(label.as_str())
                }
            });
        }
        ol
    })
}

fn render_messages_into(wrapper: Element<Div>, messages: &[(String, String)]) -> Element<Div> {
    let mut w = wrapper;
    for (msg_type, msg) in messages {
        let alert_class = match msg_type.as_str() {
            "success" => "alert-success",
            "error" => "alert-danger",
            "warning" => "alert-warning",
            _ => "alert-info",
        };
        w = w.child::<Div, _>(|d| {
            d.class(format!("alert {} alert-dismissible fade show", alert_class))
                .attr("role", "alert")
                .text(msg.as_str())
                .child::<Button, _>(|b| {
                    b.attr("type", "button")
                        .class("btn-close")
                        .data("bs-dismiss", "alert")
                })
        });
    }
    w
}

fn render_user_menu(wrapper: Element<Div>, user_name: &Option<String>) -> Element<Div> {
    match user_name {
        Some(name) => wrapper.child::<Div, _>(|d| {
            d.class("dropdown")
                .child::<Button, _>(|b| {
                    b.class(
                        "btn btn-link dropdown-toggle \
                         text-decoration-none",
                    )
                    .attr("type", "button")
                    .data("bs-toggle", "dropdown")
                    .child::<I, _>(|i| i.class("bi bi-person-circle me-1"))
                    .text(name.as_str())
                })
                .child::<Ul, _>(|ul| {
                    ul.class("dropdown-menu dropdown-menu-end")
                        .child::<Li, _>(|li| {
                            li.child::<A, _>(|a| {
                                a.class("dropdown-item")
                                    .attr("href", "/admin/password_change/")
                                    .text("Change Password")
                            })
                        })
                        .child::<Li, _>(|li| li.child::<Hr, _>(|hr| hr.class("dropdown-divider")))
                        .child::<Li, _>(|li| {
                            li.child::<A, _>(|a| {
                                a.class("dropdown-item")
                                    .attr("href", "/admin/logout/")
                                    .text("Log out")
                            })
                        })
                })
        }),
        None => wrapper.child::<A, _>(|a| {
            a.class("btn btn-primary")
                .attr("href", "/admin/login/")
                .text("Log in")
        }),
    }
}
