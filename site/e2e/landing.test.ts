import { test, expect } from '@playwright/test';

test.describe('Landing Page', () => {
  test('renders hero section', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('h1')).toBeVisible();
    await expect(page).toHaveTitle(/FreshBlu/i);
  });

  test('has navigation links', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('nav')).toBeVisible();
    await expect(page.locator('a[href="/docs"]')).toBeVisible();
    await expect(page.locator('nav a[href="/playground"]')).toBeVisible();
  });

  test('has footer', async ({ page }) => {
    await page.goto('/');
    const footer = page.locator('footer');
    await expect(footer).toBeVisible();
  });
});
