//! Base layout template.

use super::html_escape;

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

/// Renders the base admin layout.
pub fn render_base(ctx: &AdminContext) -> String {
    let nav_items = render_nav_items(&ctx.models);
    let breadcrumbs = render_breadcrumbs(&ctx.breadcrumbs);
    let messages = render_messages(&ctx.messages);
    let user_menu = render_user_menu(&ctx.user_name);

    format!(
        r##"<!DOCTYPE html>
<html lang="en" data-bs-theme="light">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{page_title} | {site_title}</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" rel="stylesheet">
    <link href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.1/font/bootstrap-icons.css" rel="stylesheet">
    <style>
        :root {{
            --sidebar-width: 280px;
        }}
        body {{
            min-height: 100vh;
        }}
        .sidebar {{
            position: fixed;
            top: 0;
            left: 0;
            bottom: 0;
            width: var(--sidebar-width);
            background-color: #212529;
            padding-top: 0;
            z-index: 1000;
            overflow-y: auto;
        }}
        .sidebar .nav-link {{
            color: rgba(255, 255, 255, 0.75);
            padding: 0.75rem 1rem;
            border-radius: 0;
        }}
        .sidebar .nav-link:hover {{
            color: rgba(255, 255, 255, 1);
            background-color: rgba(255, 255, 255, 0.1);
        }}
        .sidebar .nav-link.active {{
            color: #fff;
            background-color: #0d6efd;
        }}
        .sidebar-header {{
            padding: 1rem;
            background-color: #0d6efd;
            color: white;
        }}
        .sidebar-header h5 {{
            margin: 0;
            font-weight: 600;
        }}
        .main-content {{
            margin-left: var(--sidebar-width);
            min-height: 100vh;
        }}
        .navbar {{
            padding-left: 1.5rem;
            padding-right: 1.5rem;
        }}
        .content-wrapper {{
            padding: 1.5rem;
        }}
        .table-actions {{
            white-space: nowrap;
        }}
        .bulk-actions {{
            display: flex;
            gap: 0.5rem;
            align-items: center;
        }}
        @media (max-width: 768px) {{
            .sidebar {{
                transform: translateX(-100%);
                transition: transform 0.3s ease;
            }}
            .sidebar.show {{
                transform: translateX(0);
            }}
            .main-content {{
                margin-left: 0;
            }}
        }}
    </style>
</head>
<body>
    <!-- Sidebar -->
    <nav class="sidebar">
        <div class="sidebar-header">
            <h5><i class="bi bi-gear-fill me-2"></i>{site_header}</h5>
        </div>
        <ul class="nav flex-column">
            <li class="nav-item">
                <a class="nav-link" href="/admin/">
                    <i class="bi bi-house-door me-2"></i>Dashboard
                </a>
            </li>
            <hr class="my-2 mx-3 border-secondary">
            {nav_items}
        </ul>
    </nav>

    <!-- Main content -->
    <div class="main-content">
        <!-- Top navbar -->
        <nav class="navbar navbar-expand-lg navbar-light bg-white border-bottom">
            <div class="container-fluid">
                <button class="btn btn-link d-md-none" type="button" onclick="document.querySelector('.sidebar').classList.toggle('show')">
                    <i class="bi bi-list fs-4"></i>
                </button>
                {breadcrumbs}
                <div class="ms-auto">
                    {user_menu}
                </div>
            </div>
        </nav>

        <!-- Page content -->
        <div class="content-wrapper">
            {messages}
            <h2 class="mb-4">{page_title}</h2>
            {content}
        </div>
    </div>

    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/js/bootstrap.bundle.min.js"></script>
    <script>
        // Select all checkboxes
        document.querySelectorAll('.select-all').forEach(checkbox => {{
            checkbox.addEventListener('change', function() {{
                const table = this.closest('table');
                table.querySelectorAll('.row-select').forEach(cb => {{
                    cb.checked = this.checked;
                }});
            }});
        }});

        // Confirm delete
        document.querySelectorAll('.delete-confirm').forEach(form => {{
            form.addEventListener('submit', function(e) {{
                if (!confirm('Are you sure you want to delete this item?')) {{
                    e.preventDefault();
                }}
            }});
        }});
    </script>
</body>
</html>"##,
        page_title = html_escape(&ctx.page_title),
        site_title = html_escape(&ctx.site_title),
        site_header = html_escape(&ctx.site_header),
        nav_items = nav_items,
        breadcrumbs = breadcrumbs,
        user_menu = user_menu,
        messages = messages,
        content = ctx.content,
    )
}

fn render_nav_items(models: &[(String, String)]) -> String {
    models
        .iter()
        .map(|(name, url)| {
            format!(
                r#"<li class="nav-item">
                    <a class="nav-link" href="{}">
                        <i class="bi bi-table me-2"></i>{}
                    </a>
                </li>"#,
                html_escape(url),
                html_escape(name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_breadcrumbs(breadcrumbs: &[(String, Option<String>)]) -> String {
    let items: Vec<String> = breadcrumbs
        .iter()
        .enumerate()
        .map(|(i, (label, url))| {
            let is_last = i == breadcrumbs.len() - 1;
            if is_last {
                format!(
                    r#"<li class="breadcrumb-item active" aria-current="page">{}</li>"#,
                    html_escape(label)
                )
            } else if let Some(url) = url {
                format!(
                    r#"<li class="breadcrumb-item"><a href="{}">{}</a></li>"#,
                    html_escape(url),
                    html_escape(label)
                )
            } else {
                format!(r#"<li class="breadcrumb-item">{}</li>"#, html_escape(label))
            }
        })
        .collect();

    format!(
        r#"<nav aria-label="breadcrumb">
            <ol class="breadcrumb mb-0">{}</ol>
        </nav>"#,
        items.join("\n")
    )
}

fn render_messages(messages: &[(String, String)]) -> String {
    if messages.is_empty() {
        return String::new();
    }

    messages
        .iter()
        .map(|(msg_type, msg)| {
            let alert_class = match msg_type.as_str() {
                "success" => "alert-success",
                "error" => "alert-danger",
                "warning" => "alert-warning",
                _ => "alert-info",
            };
            format!(
                r#"<div class="alert {} alert-dismissible fade show" role="alert">
                    {}
                    <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
                </div>"#,
                alert_class,
                html_escape(msg)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_user_menu(user_name: &Option<String>) -> String {
    match user_name {
        Some(name) => format!(
            r#"<div class="dropdown">
                <button class="btn btn-link dropdown-toggle text-decoration-none" type="button" data-bs-toggle="dropdown">
                    <i class="bi bi-person-circle me-1"></i>{}
                </button>
                <ul class="dropdown-menu dropdown-menu-end">
                    <li><a class="dropdown-item" href="/admin/password_change/">Change Password</a></li>
                    <li><hr class="dropdown-divider"></li>
                    <li><a class="dropdown-item" href="/admin/logout/">Log out</a></li>
                </ul>
            </div>"#,
            html_escape(name)
        ),
        None => r#"<a class="btn btn-primary" href="/admin/login/">Log in</a>"#.to_string(),
    }
}
