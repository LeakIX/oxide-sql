import { test, expect } from '@playwright/test';

test.describe('CRUD Operations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/admin/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
  });

  test.describe('Create', () => {
    test('should navigate to add form', async ({ page }) => {
      await page.goto('/admin/posts/');
      await page.click('a[href*="add"], button:has-text("Add")');

      await expect(page).toHaveURL(/.*add/);
      await expect(page.locator('form')).toBeVisible();
    });

    test('should create a new post', async ({ page }) => {
      await page.goto('/admin/posts/add/');

      await page.fill('input[name="title"]', 'Test Post Title');
      await page.fill('input[name="slug"]', 'test-post-title');
      await page.fill('textarea[name="content"]', 'This is the test content.');

      // Select status if available
      const statusSelect = page.locator('select[name="status"]');
      if (await statusSelect.isVisible()) {
        await statusSelect.selectOption('published');
      }

      await page.click('button[type="submit"]');

      // Should redirect to list or detail page
      await expect(page).toHaveURL(/.*posts\/?(?:$|\d)/);

      // Should show success message
      const successMessage = page.locator('.alert-success, .message-success');
      await expect(successMessage).toBeVisible();
    });

    test('should show validation errors for required fields', async ({ page }) => {
      await page.goto('/admin/posts/add/');

      // Submit without filling required fields
      await page.click('button[type="submit"]');

      // Should show validation errors
      const errorMessage = page.locator(
        '.is-invalid, .field-error, .alert-danger, [aria-invalid="true"]',
      );
      await expect(errorMessage.first()).toBeVisible();
    });
  });

  test.describe('Read', () => {
    test('should display item details', async ({ page }) => {
      await page.goto('/admin/posts/');

      // Click on first item to view details
      const firstItem = page.locator('table tbody tr a, .list-item a').first();
      if (await firstItem.isVisible()) {
        await firstItem.click();
        await expect(page.locator('form, .detail-view')).toBeVisible();
      }
    });

    test('should display readonly fields correctly', async ({ page }) => {
      await page.goto('/admin/posts/1/');

      // Created at and updated at should typically be readonly
      const readonlyField = page.locator(
        'input[readonly], .readonly-field, [data-readonly]',
      );
      // May or may not exist depending on model config
      if ((await readonlyField.count()) > 0) {
        await expect(readonlyField.first()).toBeVisible();
      }
    });
  });

  test.describe('Update', () => {
    test('should update an existing post', async ({ page }) => {
      await page.goto('/admin/posts/1/');

      const titleInput = page.locator('input[name="title"]');
      await titleInput.clear();
      await titleInput.fill('Updated Post Title');

      await page.click('button[type="submit"]');

      // Should show success message
      const successMessage = page.locator('.alert-success, .message-success');
      await expect(successMessage).toBeVisible();
    });

    test('should preserve other field values on update', async ({ page }) => {
      await page.goto('/admin/posts/1/');

      // Get original content
      const contentField = page.locator('textarea[name="content"]');
      const originalContent = await contentField.inputValue();

      // Update only title
      const titleInput = page.locator('input[name="title"]');
      await titleInput.clear();
      await titleInput.fill('Another Update');

      await page.click('button[type="submit"]');

      // Reload and check content is preserved
      await page.goto('/admin/posts/1/');
      await expect(contentField).toHaveValue(originalContent);
    });
  });

  test.describe('Delete', () => {
    test('should show delete confirmation', async ({ page }) => {
      await page.goto('/admin/posts/1/');

      const deleteButton = page.locator(
        'a[href*="delete"], button:has-text("Delete")',
      );
      if (await deleteButton.isVisible()) {
        await deleteButton.click();

        // Should show confirmation page or modal
        const confirmation = page.locator(
          '.delete-confirmation, .modal, text=/confirm|sure/i',
        );
        await expect(confirmation).toBeVisible();
      }
    });

    test('should cancel delete and return to detail', async ({ page }) => {
      await page.goto('/admin/posts/1/delete/');

      const cancelButton = page.locator(
        'a:has-text("Cancel"), button:has-text("Cancel")',
      );
      if (await cancelButton.isVisible()) {
        await cancelButton.click();
        await expect(page).not.toHaveURL(/.*delete/);
      }
    });

    test('should delete item after confirmation', async ({ page }) => {
      // First create a post to delete
      await page.goto('/admin/posts/add/');
      await page.fill('input[name="title"]', 'Post to Delete');
      await page.fill('input[name="slug"]', 'post-to-delete');
      await page.fill('textarea[name="content"]', 'This will be deleted.');
      await page.click('button[type="submit"]');

      // Get the URL of the created post
      const url = page.url();
      const idMatch = url.match(/\/(\d+)\/?$/);

      if (idMatch) {
        const postId = idMatch[1];

        // Navigate to delete page
        await page.goto(`/admin/posts/${postId}/delete/`);

        // Confirm deletion
        const confirmButton = page.locator(
          'button[type="submit"]:has-text("Delete"), button:has-text("Confirm")',
        );
        await confirmButton.click();

        // Should redirect to list
        await expect(page).toHaveURL(/.*posts\/?$/);

        // Should show success message
        const successMessage = page.locator('.alert-success, .message-success');
        await expect(successMessage).toBeVisible();
      }
    });
  });
});

test.describe('Form Fieldsets', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/admin/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
  });

  test('should display fieldsets with proper grouping', async ({ page }) => {
    await page.goto('/admin/posts/add/');

    // Check for fieldset elements
    const fieldsets = page.locator('fieldset, .fieldset, .card');
    const count = await fieldsets.count();

    if (count > 1) {
      // Verify fieldsets have legends/titles
      const legends = page.locator(
        'fieldset legend, .fieldset-title, .card-header',
      );
      await expect(legends.first()).toBeVisible();
    }
  });

  test('should toggle collapsible fieldsets', async ({ page }) => {
    await page.goto('/admin/posts/add/');

    const collapsibleToggle = page.locator(
      '[data-bs-toggle="collapse"], .collapse-toggle',
    );

    if ((await collapsibleToggle.count()) > 0) {
      const content = page.locator('.collapse').first();
      const isVisible = await content.isVisible();

      await collapsibleToggle.first().click();

      // State should have toggled
      if (isVisible) {
        await expect(content).not.toBeVisible();
      } else {
        await expect(content).toBeVisible();
      }
    }
  });
});
