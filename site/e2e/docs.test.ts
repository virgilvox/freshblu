import { test, expect } from '@playwright/test';

test.describe('Documentation', () => {
  test('docs index loads with category cards', async ({ page }) => {
    await page.goto('/docs');
    await expect(page).toHaveTitle(/Documentation/i);
    const main = page.locator('main');
    await expect(main.locator('text=Tutorials').first()).toBeVisible();
    await expect(main.locator('text=Reference').first()).toBeVisible();
  });

  test('sidebar navigation works', async ({ page }) => {
    await page.goto('/docs');
    const sidebar = page.locator('nav.sidebar, aside, [class*="sidebar"]');
    await expect(sidebar.first()).toBeVisible();
  });

  test('tutorial page loads', async ({ page }) => {
    await page.goto('/docs/tutorials/getting-started');
    await expect(page.locator('h1')).toBeVisible();
  });

  test('reference page loads', async ({ page }) => {
    await page.goto('/docs/reference/http-api');
    await expect(page.locator('h1')).toContainText(/HTTP API/i);
  });

  test('how-to page loads', async ({ page }) => {
    await page.goto('/docs/how-to/register-device');
    await expect(page.locator('h1')).toBeVisible();
  });

  test('explanation page loads', async ({ page }) => {
    await page.goto('/docs/explanation/architecture');
    await expect(page.locator('h1')).toContainText(/Architecture/i);
  });
});
