import { test, expect } from '@playwright/test';

test.describe('Playground', () => {
  test('loads playground page', async ({ page }) => {
    await page.goto('/playground');
    await expect(page).toHaveTitle(/Playground/i);
  });

  test('shows register and connect mode tabs', async ({ page }) => {
    await page.goto('/playground');
    await expect(page.locator('.mode-btn:has-text("Register")')).toBeVisible();
    await expect(page.locator('.mode-btn:has-text("Connect")')).toBeVisible();
  });
});
