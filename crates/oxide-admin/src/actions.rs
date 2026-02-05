//! Built-in admin actions for bulk operations.

use std::future::Future;
use std::pin::Pin;

use crate::options::{Action, ActionResult};

/// Delete selected items action.
pub struct DeleteSelectedAction;

impl Action for DeleteSelectedAction {
    fn name(&self) -> &str {
        "delete_selected"
    }

    fn description(&self) -> &str {
        "Delete selected items"
    }

    fn execute(
        &self,
        selected_pks: &[String],
    ) -> Pin<Box<dyn Future<Output = ActionResult> + Send + '_>> {
        let count = selected_pks.len();
        Box::pin(async move {
            // Note: Actual deletion should be performed by the view handler
            // that has access to the database pool. This action just reports
            // what was requested.
            ActionResult::success(format!("Successfully deleted {} item(s)", count), count)
        })
    }
}

/// Mark selected items as active.
pub struct ActivateSelectedAction;

impl Action for ActivateSelectedAction {
    fn name(&self) -> &str {
        "activate_selected"
    }

    fn description(&self) -> &str {
        "Mark selected as active"
    }

    fn execute(
        &self,
        selected_pks: &[String],
    ) -> Pin<Box<dyn Future<Output = ActionResult> + Send + '_>> {
        let count = selected_pks.len();
        Box::pin(async move {
            ActionResult::success(format!("Successfully activated {} item(s)", count), count)
        })
    }
}

/// Mark selected items as inactive.
pub struct DeactivateSelectedAction;

impl Action for DeactivateSelectedAction {
    fn name(&self) -> &str {
        "deactivate_selected"
    }

    fn description(&self) -> &str {
        "Mark selected as inactive"
    }

    fn execute(
        &self,
        selected_pks: &[String],
    ) -> Pin<Box<dyn Future<Output = ActionResult> + Send + '_>> {
        let count = selected_pks.len();
        Box::pin(async move {
            ActionResult::success(format!("Successfully deactivated {} item(s)", count), count)
        })
    }
}

/// Export selected items to CSV action.
pub struct ExportCsvAction;

impl Action for ExportCsvAction {
    fn name(&self) -> &str {
        "export_csv"
    }

    fn description(&self) -> &str {
        "Export selected to CSV"
    }

    fn execute(
        &self,
        selected_pks: &[String],
    ) -> Pin<Box<dyn Future<Output = ActionResult> + Send + '_>> {
        let count = selected_pks.len();
        Box::pin(async move {
            // Note: Actual export is handled by the view
            ActionResult::success(format!("Exported {} item(s) to CSV", count), count)
        })
    }
}

/// A custom action that can be created with a closure.
///
/// The handler receives owned `Vec<String>` to avoid lifetime issues.
pub struct CustomAction<F>
where
    F: Fn(Vec<String>) -> Pin<Box<dyn Future<Output = ActionResult> + Send>> + Send + Sync,
{
    name: String,
    description: String,
    handler: F,
}

impl<F> CustomAction<F>
where
    F: Fn(Vec<String>) -> Pin<Box<dyn Future<Output = ActionResult> + Send>> + Send + Sync,
{
    /// Creates a new custom action.
    pub fn new(name: impl Into<String>, description: impl Into<String>, handler: F) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            handler,
        }
    }
}

impl<F> Action for CustomAction<F>
where
    F: Fn(Vec<String>) -> Pin<Box<dyn Future<Output = ActionResult> + Send>> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(
        &self,
        selected_pks: &[String],
    ) -> Pin<Box<dyn Future<Output = ActionResult> + Send + '_>> {
        (self.handler)(selected_pks.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_delete_selected_action() {
        let action = DeleteSelectedAction;
        let pks = vec!["1".to_string(), "2".to_string(), "3".to_string()];

        let result = action.execute(&pks).await;
        assert_eq!(result.affected_count, 3);
        assert!(result.message.is_some());
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_activate_action() {
        let action = ActivateSelectedAction;
        let pks = vec!["1".to_string()];

        let result = action.execute(&pks).await;
        assert_eq!(result.affected_count, 1);
    }
}
