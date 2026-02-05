//! List view filters for the admin interface.

/// A filter that can be applied to list views.
pub trait Filter: Send + Sync {
    /// Returns the filter's field name.
    fn field(&self) -> &str;

    /// Returns the display label for this filter.
    fn label(&self) -> &str;

    /// Returns the available filter options as (value, label) pairs.
    fn choices(&self) -> Vec<(String, String)>;

    /// Generates a SQL WHERE clause fragment for the given value.
    ///
    /// Returns None if the value is not valid for this filter.
    fn to_sql(&self, value: &str) -> Option<(String, Vec<String>)>;
}

/// A filter for boolean fields.
pub struct BooleanFilter {
    field: String,
    label: String,
    true_label: String,
    false_label: String,
}

impl BooleanFilter {
    /// Creates a new boolean filter.
    pub fn new(field: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            label: label.into(),
            true_label: "Yes".to_string(),
            false_label: "No".to_string(),
        }
    }

    /// Sets custom labels for true/false values.
    #[must_use]
    pub fn labels(mut self, true_label: impl Into<String>, false_label: impl Into<String>) -> Self {
        self.true_label = true_label.into();
        self.false_label = false_label.into();
        self
    }
}

impl Filter for BooleanFilter {
    fn field(&self) -> &str {
        &self.field
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn choices(&self) -> Vec<(String, String)> {
        vec![
            ("1".to_string(), self.true_label.clone()),
            ("0".to_string(), self.false_label.clone()),
        ]
    }

    fn to_sql(&self, value: &str) -> Option<(String, Vec<String>)> {
        match value {
            "1" | "true" => Some((format!("{} = ?", self.field), vec!["1".to_string()])),
            "0" | "false" => Some((format!("{} = ?", self.field), vec!["0".to_string()])),
            _ => None,
        }
    }
}

/// A filter with predefined choices.
pub struct ChoicesFilter {
    field: String,
    label: String,
    choices: Vec<(String, String)>,
}

impl ChoicesFilter {
    /// Creates a new choices filter.
    pub fn new(field: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            label: label.into(),
            choices: Vec::new(),
        }
    }

    /// Adds a choice to the filter.
    #[must_use]
    pub fn choice(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        self.choices.push((value.into(), label.into()));
        self
    }

    /// Sets all choices at once.
    #[must_use]
    pub fn choices(mut self, choices: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        self.choices = choices
            .into_iter()
            .map(|(v, l)| (v.into(), l.into()))
            .collect();
        self
    }
}

impl Filter for ChoicesFilter {
    fn field(&self) -> &str {
        &self.field
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn choices(&self) -> Vec<(String, String)> {
        self.choices.clone()
    }

    fn to_sql(&self, value: &str) -> Option<(String, Vec<String>)> {
        if self.choices.iter().any(|(v, _)| v == value) {
            Some((format!("{} = ?", self.field), vec![value.to_string()]))
        } else {
            None
        }
    }
}

/// A filter for date ranges.
pub struct DateRangeFilter {
    field: String,
    label: String,
}

impl DateRangeFilter {
    /// Creates a new date range filter.
    pub fn new(field: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            label: label.into(),
        }
    }
}

impl Filter for DateRangeFilter {
    fn field(&self) -> &str {
        &self.field
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn choices(&self) -> Vec<(String, String)> {
        vec![
            ("today".to_string(), "Today".to_string()),
            ("past_7_days".to_string(), "Past 7 days".to_string()),
            ("this_month".to_string(), "This month".to_string()),
            ("this_year".to_string(), "This year".to_string()),
        ]
    }

    fn to_sql(&self, value: &str) -> Option<(String, Vec<String>)> {
        // Using SQLite date functions
        match value {
            "today" => Some((format!("date({}) = date('now')", self.field), Vec::new())),
            "past_7_days" => Some((
                format!("{} >= date('now', '-7 days')", self.field),
                Vec::new(),
            )),
            "this_month" => Some((
                format!(
                    "strftime('%Y-%m', {}) = strftime('%Y-%m', 'now')",
                    self.field
                ),
                Vec::new(),
            )),
            "this_year" => Some((
                format!("strftime('%Y', {}) = strftime('%Y', 'now')", self.field),
                Vec::new(),
            )),
            _ => None,
        }
    }
}

/// A filter for numeric ranges.
pub struct RangeFilter {
    field: String,
    label: String,
    ranges: Vec<(String, String, Option<i64>, Option<i64>)>,
}

impl RangeFilter {
    /// Creates a new range filter.
    pub fn new(field: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            label: label.into(),
            ranges: Vec::new(),
        }
    }

    /// Adds a range option.
    #[must_use]
    pub fn range(
        mut self,
        value: impl Into<String>,
        label: impl Into<String>,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Self {
        self.ranges.push((value.into(), label.into(), min, max));
        self
    }
}

impl Filter for RangeFilter {
    fn field(&self) -> &str {
        &self.field
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn choices(&self) -> Vec<(String, String)> {
        self.ranges
            .iter()
            .map(|(v, l, _, _)| (v.clone(), l.clone()))
            .collect()
    }

    fn to_sql(&self, value: &str) -> Option<(String, Vec<String>)> {
        self.ranges
            .iter()
            .find(|(v, _, _, _)| v == value)
            .map(|(_, _, min, max)| {
                let mut conditions = Vec::new();
                let mut params = Vec::new();

                if let Some(min_val) = min {
                    conditions.push(format!("{} >= ?", self.field));
                    params.push(min_val.to_string());
                }
                if let Some(max_val) = max {
                    conditions.push(format!("{} <= ?", self.field));
                    params.push(max_val.to_string());
                }

                if conditions.is_empty() {
                    ("1 = 1".to_string(), params)
                } else {
                    (conditions.join(" AND "), params)
                }
            })
    }
}

/// A filter for null/not null checks.
pub struct NullFilter {
    field: String,
    label: String,
}

impl NullFilter {
    /// Creates a new null filter.
    pub fn new(field: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            label: label.into(),
        }
    }
}

impl Filter for NullFilter {
    fn field(&self) -> &str {
        &self.field
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn choices(&self) -> Vec<(String, String)> {
        vec![
            ("null".to_string(), "Empty".to_string()),
            ("notnull".to_string(), "Not empty".to_string()),
        ]
    }

    fn to_sql(&self, value: &str) -> Option<(String, Vec<String>)> {
        match value {
            "null" => Some((format!("{} IS NULL", self.field), Vec::new())),
            "notnull" => Some((format!("{} IS NOT NULL", self.field), Vec::new())),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_filter() {
        let filter = BooleanFilter::new("is_active", "Active").labels("Active", "Inactive");

        assert_eq!(filter.field(), "is_active");
        assert_eq!(filter.label(), "Active");

        let choices = filter.choices();
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0], ("1".to_string(), "Active".to_string()));

        let (sql, params) = filter.to_sql("1").unwrap();
        assert_eq!(sql, "is_active = ?");
        assert_eq!(params, vec!["1"]);
    }

    #[test]
    fn test_choices_filter() {
        let filter = ChoicesFilter::new("status", "Status")
            .choice("draft", "Draft")
            .choice("published", "Published")
            .choice("archived", "Archived");

        // Use Filter trait method to get choices
        let choices = Filter::choices(&filter);
        assert_eq!(choices.len(), 3);

        let (sql, params) = filter.to_sql("published").unwrap();
        assert_eq!(sql, "status = ?");
        assert_eq!(params, vec!["published"]);

        assert!(filter.to_sql("invalid").is_none());
    }

    #[test]
    fn test_date_range_filter() {
        let filter = DateRangeFilter::new("created_at", "Created");

        // Use Filter trait method to get choices
        let choices = Filter::choices(&filter);
        assert_eq!(choices.len(), 4);

        let (sql, _) = filter.to_sql("today").unwrap();
        assert!(sql.contains("date('now')"));
    }

    #[test]
    fn test_null_filter() {
        let filter = NullFilter::new("deleted_at", "Deleted");

        let (sql, _) = filter.to_sql("null").unwrap();
        assert_eq!(sql, "deleted_at IS NULL");

        let (sql, _) = filter.to_sql("notnull").unwrap();
        assert_eq!(sql, "deleted_at IS NOT NULL");
    }
}
