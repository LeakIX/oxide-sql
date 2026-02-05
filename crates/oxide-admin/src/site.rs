//! Admin site registration and configuration.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use oxide_orm::Model;
use oxide_router::RouteGroup;

use crate::options::ModelAdmin;

/// Type-erased model registration info.
#[derive(Clone)]
pub struct ModelRegistration {
    /// Model name (e.g., "User", "Post").
    pub name: String,
    /// Verbose name for display.
    pub verbose_name: String,
    /// Verbose name plural.
    pub verbose_name_plural: String,
    /// URL slug (e.g., "user", "post").
    pub slug: String,
    /// Admin configuration.
    pub admin: ModelAdmin,
    /// Type ID for the model.
    pub type_id: TypeId,
}

/// Admin site that manages model registrations and routing.
#[derive(Clone)]
pub struct AdminSite {
    /// Site name/title.
    pub name: String,
    /// URL prefix (default: "/admin").
    pub url_prefix: String,
    /// Registered models.
    registrations: Arc<HashMap<TypeId, ModelRegistration>>,
    /// Model order for display.
    model_order: Arc<Vec<TypeId>>,
}

impl Default for AdminSite {
    fn default() -> Self {
        Self::new("Administration")
    }
}

impl AdminSite {
    /// Creates a new admin site with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url_prefix: "/admin".to_string(),
            registrations: Arc::new(HashMap::new()),
            model_order: Arc::new(Vec::new()),
        }
    }

    /// Sets the URL prefix for the admin site.
    #[must_use]
    pub fn url_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.url_prefix = prefix.into();
        self
    }

    /// Registers a model with custom admin options.
    #[must_use]
    pub fn register<M: Model + 'static>(mut self, admin: ModelAdmin) -> Self {
        let type_id = TypeId::of::<M>();
        let name = std::any::type_name::<M>()
            .rsplit("::")
            .next()
            .unwrap_or("Model")
            .to_string();

        let slug = name.to_lowercase();
        let verbose_name = humanize_model_name(&name);
        let verbose_name_plural = pluralize(&verbose_name);

        let registration = ModelRegistration {
            name: name.clone(),
            verbose_name,
            verbose_name_plural,
            slug,
            admin,
            type_id,
        };

        // We need to clone and modify the Arc
        let mut registrations = (*self.registrations).clone();
        registrations.insert(type_id, registration);
        self.registrations = Arc::new(registrations);

        let mut order = (*self.model_order).clone();
        order.push(type_id);
        self.model_order = Arc::new(order);

        self
    }

    /// Registers a model with default admin options.
    #[must_use]
    pub fn register_default<M: Model + 'static>(self) -> Self {
        self.register::<M>(ModelAdmin::default())
    }

    /// Returns all registered models in order.
    pub fn registered_models(&self) -> Vec<&ModelRegistration> {
        self.model_order
            .iter()
            .filter_map(|type_id| self.registrations.get(type_id))
            .collect()
    }

    /// Gets registration for a model by type.
    pub fn get_registration<M: 'static>(&self) -> Option<&ModelRegistration> {
        let type_id = TypeId::of::<M>();
        self.registrations.get(&type_id)
    }

    /// Gets registration by slug.
    pub fn get_registration_by_slug(&self, slug: &str) -> Option<&ModelRegistration> {
        self.registrations.values().find(|reg| reg.slug == slug)
    }

    /// Returns the list URL for a model.
    pub fn list_url(&self, slug: &str) -> String {
        format!("{}/{}/", self.url_prefix, slug)
    }

    /// Returns the add URL for a model.
    pub fn add_url(&self, slug: &str) -> String {
        format!("{}/{}/add/", self.url_prefix, slug)
    }

    /// Returns the change URL for a model object.
    pub fn change_url(&self, slug: &str, pk: &str) -> String {
        format!("{}/{}/{}/change/", self.url_prefix, slug, pk)
    }

    /// Returns the delete URL for a model object.
    pub fn delete_url(&self, slug: &str, pk: &str) -> String {
        format!("{}/{}/{}/delete/", self.url_prefix, slug, pk)
    }

    /// Returns the model list for the sidebar navigation.
    pub fn model_list(&self) -> Vec<(String, String)> {
        self.registered_models()
            .iter()
            .map(|reg| (reg.verbose_name_plural.clone(), self.list_url(&reg.slug)))
            .collect()
    }

    /// Builds a RouteGroup with all admin routes.
    ///
    /// This generates routes for:
    /// - GET {prefix}/ - Dashboard
    /// - GET {prefix}/{model}/ - List view
    /// - GET {prefix}/{model}/add/ - Add form
    /// - POST {prefix}/{model}/add/ - Create object
    /// - GET {prefix}/{model}/{pk}/change/ - Edit form
    /// - POST {prefix}/{model}/{pk}/change/ - Update object
    /// - GET {prefix}/{model}/{pk}/delete/ - Delete confirmation
    /// - POST {prefix}/{model}/{pk}/delete/ - Delete object
    pub fn routes(&self) -> RouteGroup {
        RouteGroup::new(&self.url_prefix)
    }
}

/// Converts a CamelCase model name to a human-readable name.
fn humanize_model_name(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push(' ');
        }
        if i == 0 {
            result.push(c.to_ascii_uppercase());
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }
    result
}

/// Simple pluralization (adds 's' or 'es').
fn pluralize(name: &str) -> String {
    if name.ends_with('s') || name.ends_with('x') || name.ends_with("ch") || name.ends_with("sh") {
        format!("{name}es")
    } else if name.ends_with('y') {
        let mut s = name.to_string();
        s.pop();
        format!("{s}ies")
    } else {
        format!("{name}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_model_name() {
        assert_eq!(humanize_model_name("User"), "User");
        assert_eq!(humanize_model_name("BlogPost"), "Blog post");
        assert_eq!(humanize_model_name("UserProfile"), "User profile");
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize("User"), "Users");
        assert_eq!(pluralize("Post"), "Posts");
        assert_eq!(pluralize("Category"), "Categories");
        assert_eq!(pluralize("Box"), "Boxes");
        assert_eq!(pluralize("Class"), "Classes");
    }

    #[test]
    fn test_admin_site_urls() {
        let site = AdminSite::new("Test Admin");

        assert_eq!(site.list_url("user"), "/admin/user/");
        assert_eq!(site.add_url("post"), "/admin/post/add/");
        assert_eq!(
            site.change_url("comment", "123"),
            "/admin/comment/123/change/"
        );
        assert_eq!(site.delete_url("tag", "456"), "/admin/tag/456/delete/");
    }

    #[test]
    fn test_custom_prefix() {
        let site = AdminSite::new("Custom").url_prefix("/manage");

        assert_eq!(site.list_url("user"), "/manage/user/");
    }
}
