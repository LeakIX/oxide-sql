//! Aggregate functions for QuerySet operations.
//!
//! Provides Django-like aggregate functions such as Count, Sum, Avg, Max, Min.

/// An aggregate function that can be applied to a QuerySet.
#[derive(Debug, Clone)]
pub enum Aggregate {
    /// COUNT aggregate
    Count {
        /// Column to count, or "*" for all rows
        column: String,
        /// Whether to count only distinct values
        distinct: bool,
    },
    /// SUM aggregate
    Sum {
        /// Column to sum
        column: String,
    },
    /// AVG aggregate
    Avg {
        /// Column to average
        column: String,
    },
    /// MAX aggregate
    Max {
        /// Column to find maximum
        column: String,
    },
    /// MIN aggregate
    Min {
        /// Column to find minimum
        column: String,
    },
}

impl Aggregate {
    /// Creates a COUNT(*) aggregate.
    pub fn count_all() -> Self {
        Self::Count {
            column: "*".to_string(),
            distinct: false,
        }
    }

    /// Creates a COUNT(column) aggregate.
    pub fn count(column: &str) -> Self {
        Self::Count {
            column: column.to_string(),
            distinct: false,
        }
    }

    /// Creates a COUNT(DISTINCT column) aggregate.
    pub fn count_distinct(column: &str) -> Self {
        Self::Count {
            column: column.to_string(),
            distinct: true,
        }
    }

    /// Creates a SUM(column) aggregate.
    pub fn sum(column: &str) -> Self {
        Self::Sum {
            column: column.to_string(),
        }
    }

    /// Creates an AVG(column) aggregate.
    pub fn avg(column: &str) -> Self {
        Self::Avg {
            column: column.to_string(),
        }
    }

    /// Creates a MAX(column) aggregate.
    pub fn max(column: &str) -> Self {
        Self::Max {
            column: column.to_string(),
        }
    }

    /// Creates a MIN(column) aggregate.
    pub fn min(column: &str) -> Self {
        Self::Min {
            column: column.to_string(),
        }
    }

    /// Returns the SQL representation of this aggregate.
    pub fn to_sql(&self) -> String {
        match self {
            Self::Count { column, distinct } => {
                if *distinct {
                    format!("COUNT(DISTINCT {column})")
                } else {
                    format!("COUNT({column})")
                }
            }
            Self::Sum { column } => format!("SUM({column})"),
            Self::Avg { column } => format!("AVG({column})"),
            Self::Max { column } => format!("MAX({column})"),
            Self::Min { column } => format!("MIN({column})"),
        }
    }
}

/// Convenience function to create a COUNT(*) aggregate.
pub fn count_all() -> Aggregate {
    Aggregate::count_all()
}

/// Convenience function to create a COUNT(column) aggregate.
pub fn count(column: &str) -> Aggregate {
    Aggregate::count(column)
}

/// Convenience function to create a COUNT(DISTINCT column) aggregate.
pub fn count_distinct(column: &str) -> Aggregate {
    Aggregate::count_distinct(column)
}

/// Convenience function to create a SUM(column) aggregate.
pub fn sum(column: &str) -> Aggregate {
    Aggregate::sum(column)
}

/// Convenience function to create an AVG(column) aggregate.
pub fn avg(column: &str) -> Aggregate {
    Aggregate::avg(column)
}

/// Convenience function to create a MAX(column) aggregate.
pub fn max(column: &str) -> Aggregate {
    Aggregate::max(column)
}

/// Convenience function to create a MIN(column) aggregate.
pub fn min(column: &str) -> Aggregate {
    Aggregate::min(column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_all() {
        let agg = count_all();
        assert_eq!(agg.to_sql(), "COUNT(*)");
    }

    #[test]
    fn test_count_column() {
        let agg = count("id");
        assert_eq!(agg.to_sql(), "COUNT(id)");
    }

    #[test]
    fn test_count_distinct() {
        let agg = count_distinct("user_id");
        assert_eq!(agg.to_sql(), "COUNT(DISTINCT user_id)");
    }

    #[test]
    fn test_sum() {
        let agg = sum("amount");
        assert_eq!(agg.to_sql(), "SUM(amount)");
    }

    #[test]
    fn test_avg() {
        let agg = avg("price");
        assert_eq!(agg.to_sql(), "AVG(price)");
    }

    #[test]
    fn test_max() {
        let agg = max("created_at");
        assert_eq!(agg.to_sql(), "MAX(created_at)");
    }

    #[test]
    fn test_min() {
        let agg = min("id");
        assert_eq!(agg.to_sql(), "MIN(id)");
    }
}
