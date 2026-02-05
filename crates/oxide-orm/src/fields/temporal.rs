//! Temporal (date/time) field types.

use super::{Field, FieldOptions};

/// A date field.
#[derive(Debug, Clone)]
pub struct DateField {
    /// Whether to auto-set to now on creation.
    pub auto_now_add: bool,
    /// Whether to auto-set to now on every save.
    pub auto_now: bool,
    /// Field options.
    pub options: FieldOptions,
}

impl DateField {
    /// Creates a new DateField.
    pub fn new() -> Self {
        Self {
            auto_now_add: false,
            auto_now: false,
            options: FieldOptions::new(),
        }
    }

    /// Sets auto_now_add (set to now on creation).
    #[must_use]
    pub fn auto_now_add(mut self) -> Self {
        self.auto_now_add = true;
        self
    }

    /// Sets auto_now (set to now on every save).
    #[must_use]
    pub fn auto_now(mut self) -> Self {
        self.auto_now = true;
        self
    }

    /// Sets field options.
    #[must_use]
    pub fn options(mut self, options: FieldOptions) -> Self {
        self.options = options;
        self
    }
}

impl Default for DateField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for DateField {
    fn sql_type(&self) -> &'static str {
        "DATE"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        // Basic ISO date format validation (YYYY-MM-DD)
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() != 3 {
            return Err("Date must be in YYYY-MM-DD format".to_string());
        }

        let year: i32 = parts[0].parse().map_err(|_| "Invalid year".to_string())?;
        let month: u32 = parts[1].parse().map_err(|_| "Invalid month".to_string())?;
        let day: u32 = parts[2].parse().map_err(|_| "Invalid day".to_string())?;

        if !(1..=12).contains(&month) {
            return Err("Month must be between 1 and 12".to_string());
        }

        if !(1..=31).contains(&day) {
            return Err("Day must be between 1 and 31".to_string());
        }

        // Basic year range check
        if !(1..=9999).contains(&year) {
            return Err("Year must be between 1 and 9999".to_string());
        }

        Ok(())
    }
}

/// A datetime field with timezone support.
#[derive(Debug, Clone)]
pub struct DateTimeField {
    /// Whether to auto-set to now on creation.
    pub auto_now_add: bool,
    /// Whether to auto-set to now on every save.
    pub auto_now: bool,
    /// Field options.
    pub options: FieldOptions,
}

impl DateTimeField {
    /// Creates a new DateTimeField.
    pub fn new() -> Self {
        Self {
            auto_now_add: false,
            auto_now: false,
            options: FieldOptions::new(),
        }
    }

    /// Sets auto_now_add (set to now on creation).
    #[must_use]
    pub fn auto_now_add(mut self) -> Self {
        self.auto_now_add = true;
        self
    }

    /// Sets auto_now (set to now on every save).
    #[must_use]
    pub fn auto_now(mut self) -> Self {
        self.auto_now = true;
        self
    }

    /// Sets field options.
    #[must_use]
    pub fn options(mut self, options: FieldOptions) -> Self {
        self.options = options;
        self
    }
}

impl Default for DateTimeField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for DateTimeField {
    fn sql_type(&self) -> &'static str {
        "TIMESTAMP"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        // Accept ISO 8601 format: YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM:SS
        let normalized = value.replace('T', " ");
        let parts: Vec<&str> = normalized.split(' ').collect();

        if parts.len() < 2 {
            return Err("DateTime must include date and time".to_string());
        }

        // Validate date part
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        if date_parts.len() != 3 {
            return Err("Date must be in YYYY-MM-DD format".to_string());
        }

        // Validate time part (allow optional timezone)
        let time_str = parts[1].split('+').next().unwrap_or(parts[1]);
        let time_str = time_str.split('Z').next().unwrap_or(time_str);
        let time_parts: Vec<&str> = time_str.split(':').collect();

        if time_parts.len() < 2 {
            return Err("Time must include at least hours and minutes".to_string());
        }

        let hour: u32 = time_parts[0]
            .parse()
            .map_err(|_| "Invalid hour".to_string())?;
        let minute: u32 = time_parts[1]
            .parse()
            .map_err(|_| "Invalid minute".to_string())?;

        if hour > 23 {
            return Err("Hour must be between 0 and 23".to_string());
        }
        if minute > 59 {
            return Err("Minute must be between 0 and 59".to_string());
        }

        Ok(())
    }
}

/// A time field.
#[derive(Debug, Clone)]
pub struct TimeField {
    /// Field options.
    pub options: FieldOptions,
}

impl TimeField {
    /// Creates a new TimeField.
    pub fn new() -> Self {
        Self {
            options: FieldOptions::new(),
        }
    }

    /// Sets field options.
    #[must_use]
    pub fn options(mut self, options: FieldOptions) -> Self {
        self.options = options;
        self
    }
}

impl Default for TimeField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for TimeField {
    fn sql_type(&self) -> &'static str {
        "TIME"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        let parts: Vec<&str> = value.split(':').collect();

        if parts.len() < 2 {
            return Err("Time must include at least hours and minutes".to_string());
        }

        let hour: u32 = parts[0].parse().map_err(|_| "Invalid hour".to_string())?;
        let minute: u32 = parts[1].parse().map_err(|_| "Invalid minute".to_string())?;

        if hour > 23 {
            return Err("Hour must be between 0 and 23".to_string());
        }
        if minute > 59 {
            return Err("Minute must be between 0 and 59".to_string());
        }

        if parts.len() > 2 {
            // Remove fractional seconds for validation
            let sec_str = parts[2].split('.').next().unwrap_or(parts[2]);
            let second: u32 = sec_str.parse().map_err(|_| "Invalid second".to_string())?;
            if second > 59 {
                return Err("Second must be between 0 and 59".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_field_validation() {
        let field = DateField::new();
        assert!(field.validate("2024-01-15").is_ok());
        assert!(field.validate("2024-12-31").is_ok());
        assert!(field.validate("2024-13-01").is_err()); // Invalid month
        assert!(field.validate("2024-01-32").is_err()); // Invalid day
        assert!(field.validate("invalid").is_err());
    }

    #[test]
    fn test_datetime_field_validation() {
        let field = DateTimeField::new();
        assert!(field.validate("2024-01-15 10:30:00").is_ok());
        assert!(field.validate("2024-01-15T10:30:00").is_ok());
        assert!(field.validate("2024-01-15T10:30:00Z").is_ok());
        assert!(field.validate("2024-01-15T10:30:00+00:00").is_ok());
        assert!(field.validate("invalid").is_err());
    }

    #[test]
    fn test_time_field_validation() {
        let field = TimeField::new();
        assert!(field.validate("10:30").is_ok());
        assert!(field.validate("10:30:45").is_ok());
        assert!(field.validate("10:30:45.123").is_ok());
        assert!(field.validate("25:00").is_err()); // Invalid hour
        assert!(field.validate("10:60").is_err()); // Invalid minute
    }
}
