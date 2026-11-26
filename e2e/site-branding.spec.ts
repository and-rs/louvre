import { expect, test } from "@playwright/test";
import { publishFixture } from "./artwork-cli";

const siteName = process.env.SITE_NAME?.trim() || "Louvre";

test("configured branding appears on public pages", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(siteName);
  await expect(page.getByRole("heading", { name: siteName, exact: true, level: 1 })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Main navigation" }).getByRole("link", { name: siteName, exact: true })).toBeVisible();
  await expect(page.locator("footer").getByRole("link", { name: siteName, exact: true })).toBeVisible();
  await expect(page.locator("footer")).toContainText(`Copyright (c) 2026 ${siteName}. All Rights Reserved.`);

  await page.goto("/missing-page");
  await expect(page).toHaveTitle(`Not found | ${siteName}`);
});

test("artwork without a description uses the configured brand in metadata", async ({ page }) => {
  publishFixture("brand-metadata-artwork", "Brand metadata artwork", "A branding test image");

  await page.goto("/artwork/brand-metadata-artwork");
  await expect(page).toHaveTitle(`Brand metadata artwork | ${siteName}`);
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    "content",
    `Artwork from the ${siteName} collection.`,
  );
});
