import { test, expect } from '@playwright/test';

test.describe('Admin Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/admin/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/admin/');
  });

  test('should display dashboard with registered models', async ({ page }) => {
    await expect(page.locator('h1, .dashboard-title')).toContainText(/Dashboard|Admin/i);
    await expect(page.locator('a[href*="posts"]')).toBeVisible();
    await expect(page.locator('a[href*="comments"]')).toBeVisible();
    await expect(page.locator('a[href*="tags"]')).toBeVisible();
  });

  test('should navigate to model list view', async ({ page }) => {
    await page.click('a[href*="posts"]');
    await expect(page).toHaveURL(/.*posts/);
    await expect(page.locator('h1, .page-title')).toContainText(/Posts/i);
  });

  test('should display model counts on dashboard', async ({ page }) => {
    // Dashboard should show counts for each model
    const postCount = page.locator('[data-model="posts"] .count, a[href*="posts"] + .count');
    await expect(postCount).toBeVisible();
  });
});

test.describe('Admin List View', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/admin/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.goto('/admin/posts/');
  });

  test('should display list of items', async ({ page }) => {
    await expect(page.locator('table, .list-view')).toBeVisible();
    await expect(page.locator('th, .column-header')).toHaveCount.greaterThan(0);
  });

  test('should have pagination controls', async ({ page }) => {
    const pagination = page.locator('.pagination, nav[aria-label="pagination"]');
    await expect(pagination).toBeVisible();
  });

  test('should search items', async ({ page }) => {
    const searchInput = page.locator('input[name="q"], input[type="search"]');
    await expect(searchInput).toBeVisible();

    await searchInput.fill('test');
    await page.press('input[name="q"], input[type="search"]', 'Enter');

    await expect(page).toHaveURL(/.*q=test/);
  });

  test('should filter items', async ({ page }) => {
    const filterSelect = page.locator('select[name="status"], .filter-select').first();
    if (await filterSelect.isVisible()) {
      await filterSelect.selectOption({ index: 1 });
      await expect(page).toHaveURL(/.*status=/);
    }
  });

  test('should sort by column', async ({ page }) => {
    const sortableHeader = page.locator('th[data-sortable] a, th a[href*="order"]').first();
    if (await sortableHeader.isVisible()) {
      await sortableHeader.click();
      await expect(page).toHaveURL(/.*order=/);
    }
  });

  test('should select items with checkboxes', async ({ page }) => {
    const checkbox = page.locator('input[type="checkbox"][name="selected"]').first();
    if (await checkbox.isVisible()) {
      await checkbox.check();
      await expect(checkbox).toBeChecked();
    }
  });

  test('should have add new button', async ({ page }) => {
    const addButton = page.locator('a[href*="add"], button:has-text("Add")');
    await expect(addButton).toBeVisible();
  });
});

test.describe('Admin Bulk Actions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/admin/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.goto('/admin/posts/');
  });

  test('should have action dropdown', async ({ page }) => {
    const actionSelect = page.locator('select[name="action"], .action-select');
    await expect(actionSelect).toBeVisible();
  });

  test('should execute bulk action on selected items', async ({ page }) => {
    // Select items
    const checkboxes = page.locator('input[type="checkbox"][name="selected"]');
    const count = await checkboxes.count();

    if (count > 0) {
      await checkboxes.first().check();

      // Select action
      const actionSelect = page.locator('select[name="action"]');
      await actionSelect.selectOption('delete_selected');

      // Submit
      await page.click('button[name="apply_action"], button:has-text("Apply")');

      // Should show confirmation or success message
      const confirmation = page.locator('.confirmation, .alert');
      await expect(confirmation).toBeVisible();
    }
  });
});
