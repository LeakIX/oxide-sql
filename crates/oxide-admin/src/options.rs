//! ModelAdmin configuration options.

/// Configuration for how a model is displayed in the admin.
#[derive(Debug, Clone)]
pub struct ModelAdmin {
    /// Columns to display in the list view.
    pub list_display: Vec<String>,
    /// Columns that link to the edit page.
    pub list_display_links: Vec<String>,
    /// Columns that can be used for filtering.
    pub list_filter: Vec<String>,
    /// Columns that can be searched.
    pub search_fields: Vec<String>,
    /// Default ordering (prefix with - for descending).
    pub ordering: Vec<String>,
    /// Number of items per page.
    pub list_per_page: usize,
    /// Fields to show in the detail view (None = all).
    pub fields: Option<Vec<String>>,
    /// Fields to exclude from the detail view.
    pub exclude: Vec<String>,
    /// Fields that are read-only.
    pub readonly_fields: Vec<String>,
    /// Field groupings for the detail view.
    pub fieldsets: Vec<Fieldset>,
    /// Date hierarchy field for filtering.
    pub date_hierarchy: Option<String>,
    /// Whether to show actions dropdown.
    pub actions_on_top: bool,
    /// Whether to show actions at bottom.
    pub actions_on_bottom: bool,
}

impl Default for ModelAdmin {
    fn default() -> Self {
        Self {
            list_display: Vec::new(),
            list_display_links: Vec::new(),
            list_filter: Vec::new(),
            search_fields: Vec::new(),
            ordering: Vec::new(),
            list_per_page: 25,
            fields: None,
            exclude: Vec::new(),
            readonly_fields: Vec::new(),
            fieldsets: Vec::new(),
            date_hierarchy: None,
            actions_on_top: true,
            actions_on_bottom: false,
        }
    }
}

impl ModelAdmin {
    /// Creates a new ModelAdmin with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the columns to display in the list view.
    #[must_use]
    pub fn list_display(mut self, cols: &[&str]) -> Self {
        self.list_display = cols.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Sets the columns that link to the edit page.
    #[must_use]
    pub fn list_display_links(mut self, cols: &[&str]) -> Self {
        self.list_display_links = cols.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Sets the columns for filtering.
    #[must_use]
    pub fn list_filter(mut self, cols: &[&str]) -> Self {
        self.list_filter = cols.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Sets the columns for searching.
    #[must_use]
    pub fn search_fields(mut self, cols: &[&str]) -> Self {
        self.search_fields = cols.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Sets the default ordering.
    #[must_use]
    pub fn ordering(mut self, cols: &[&str]) -> Self {
        self.ordering = cols.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Sets the number of items per page.
    #[must_use]
    pub fn list_per_page(mut self, n: usize) -> Self {
        self.list_per_page = n;
        self
    }

    /// Sets the fields to show in detail view.
    #[must_use]
    pub fn fields(mut self, cols: &[&str]) -> Self {
        self.fields = Some(cols.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Sets the fields to exclude.
    #[must_use]
    pub fn exclude(mut self, cols: &[&str]) -> Self {
        self.exclude = cols.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Sets the read-only fields.
    #[must_use]
    pub fn readonly_fields(mut self, cols: &[&str]) -> Self {
        self.readonly_fields = cols.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Adds a fieldset.
    #[must_use]
    pub fn fieldset(mut self, fieldset: Fieldset) -> Self {
        self.fieldsets.push(fieldset);
        self
    }

    /// Sets the date hierarchy field.
    #[must_use]
    pub fn date_hierarchy(mut self, field: &str) -> Self {
        self.date_hierarchy = Some(field.to_string());
        self
    }
}

/// A fieldset groups related fields together in the detail view.
#[derive(Debug, Clone)]
pub struct Fieldset {
    /// Optional name/title.
    pub name: Option<String>,
    /// Fields in this set.
    pub fields: Vec<String>,
    /// CSS classes (e.g., "collapse" to make collapsible).
    pub classes: Vec<String>,
    /// Optional description.
    pub description: Option<String>,
}

impl Fieldset {
    /// Creates a new fieldset with the given fields.
    pub fn new(fields: &[&str]) -> Self {
        Self {
            name: None,
            fields: fields.iter().map(|s| (*s).to_string()).collect(),
            classes: Vec::new(),
            description: None,
        }
    }

    /// Creates a named fieldset.
    pub fn named(name: &str, fields: &[&str]) -> Self {
        Self {
            name: Some(name.to_string()),
            fields: fields.iter().map(|s| (*s).to_string()).collect(),
            classes: Vec::new(),
            description: None,
        }
    }

    /// Adds CSS classes.
    #[must_use]
    pub fn classes(mut self, classes: &[&str]) -> Self {
        self.classes = classes.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Sets the description.
    #[must_use]
    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Makes the fieldset collapsible.
    #[must_use]
    pub fn collapse(mut self) -> Self {
        self.classes.push("collapse".to_string());
        self
    }
}

/// Configuration for inline editing of related models.
#[derive(Debug, Clone)]
pub struct InlineAdmin {
    /// The foreign key field name.
    pub fk_field: String,
    /// Fields to display.
    pub fields: Vec<String>,
    /// Number of extra empty forms.
    pub extra: usize,
    /// Maximum number of forms.
    pub max_num: Option<usize>,
    /// Minimum number of forms.
    pub min_num: usize,
    /// Whether to show delete checkbox.
    pub can_delete: bool,
    /// Verbose name.
    pub verbose_name: Option<String>,
    /// Verbose name plural.
    pub verbose_name_plural: Option<String>,
}

impl InlineAdmin {
    /// Creates a new inline admin.
    pub fn new(fk_field: &str) -> Self {
        Self {
            fk_field: fk_field.to_string(),
            fields: Vec::new(),
            extra: 3,
            max_num: None,
            min_num: 0,
            can_delete: true,
            verbose_name: None,
            verbose_name_plural: None,
        }
    }

    /// Sets the fields to display.
    #[must_use]
    pub fn fields(mut self, fields: &[&str]) -> Self {
        self.fields = fields.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Sets the number of extra forms.
    #[must_use]
    pub fn extra(mut self, n: usize) -> Self {
        self.extra = n;
        self
    }

    /// Sets the maximum number of forms.
    #[must_use]
    pub fn max_num(mut self, n: usize) -> Self {
        self.max_num = Some(n);
        self
    }

    /// Sets the minimum number of forms.
    #[must_use]
    pub fn min_num(mut self, n: usize) -> Self {
        self.min_num = n;
        self
    }

    /// Sets whether items can be deleted.
    #[must_use]
    pub fn can_delete(mut self, v: bool) -> Self {
        self.can_delete = v;
        self
    }

    /// Sets the verbose name.
    #[must_use]
    pub fn verbose_name(mut self, name: &str) -> Self {
        self.verbose_name = Some(name.to_string());
        self
    }

    /// Sets the verbose name plural.
    #[must_use]
    pub fn verbose_name_plural(mut self, name: &str) -> Self {
        self.verbose_name_plural = Some(name.to_string());
        self
    }
}

/// A bulk action that can be performed on selected items.
pub trait Action: Send + Sync {
    /// Returns the action name (used as value).
    fn name(&self) -> &str;

    /// Returns the action description (shown in dropdown).
    fn description(&self) -> &str;

    /// Executes the action on the selected items.
    fn execute(
        &self,
        selected_pks: &[String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ActionResult> + Send + '_>>;
}

/// Result of executing an action.
#[derive(Debug)]
pub struct ActionResult {
    /// Success message.
    pub message: Option<String>,
    /// Error message.
    pub error: Option<String>,
    /// Number of items affected.
    pub affected_count: usize,
}

impl ActionResult {
    /// Creates a success result.
    pub fn success(message: impl Into<String>, affected: usize) -> Self {
        Self {
            message: Some(message.into()),
            error: None,
            affected_count: affected,
        }
    }

    /// Creates an error result.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: None,
            error: Some(message.into()),
            affected_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_admin_builder() {
        let admin = ModelAdmin::new()
            .list_display(&["id", "title", "created_at"])
            .list_filter(&["status"])
            .search_fields(&["title", "content"])
            .ordering(&["-created_at"])
            .list_per_page(50);

        assert_eq!(admin.list_display, vec!["id", "title", "created_at"]);
        assert_eq!(admin.list_filter, vec!["status"]);
        assert_eq!(admin.search_fields, vec!["title", "content"]);
        assert_eq!(admin.ordering, vec!["-created_at"]);
        assert_eq!(admin.list_per_page, 50);
    }

    #[test]
    fn test_fieldset() {
        let fieldset = Fieldset::named("Basic Info", &["title", "slug"])
            .description("Enter the basic information")
            .collapse();

        assert_eq!(fieldset.name, Some("Basic Info".to_string()));
        assert_eq!(fieldset.fields, vec!["title", "slug"]);
        assert!(fieldset.classes.contains(&"collapse".to_string()));
    }

    #[test]
    fn test_inline_admin() {
        let inline = InlineAdmin::new("post_id")
            .fields(&["content", "author"])
            .extra(2)
            .max_num(10);

        assert_eq!(inline.fk_field, "post_id");
        assert_eq!(inline.fields, vec!["content", "author"]);
        assert_eq!(inline.extra, 2);
        assert_eq!(inline.max_num, Some(10));
    }
}
