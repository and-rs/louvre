import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";

const siteName = process.env.SITE_NAME?.trim() || "Louvre";

test("configured branding appears on public pages and Studio", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(siteName);
  await expect(page.getByRole("heading", { name: siteName, exact: true, level: 1 })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Main navigation" }).getByRole("link", { name: siteName, exact: true })).toBeVisible();
  await expect(page.locator("footer").getByRole("link", { name: siteName, exact: true })).toBeVisible();
  await expect(page.locator("footer")).toContainText(`Copyright (c) 2026 ${siteName}. All Rights Reserved.`);

  await page.goto("/studio/login");
  await expect(page).toHaveTitle(`Studio sign in | ${siteName}`);
  await expect(page.getByText(`${siteName} / PRIVATE WORKSPACE`, { exact: true })).toBeVisible();

  await page.goto("/missing-page");
  await expect(page).toHaveTitle(`Not found | ${siteName}`);
});

test("artwork without a description uses the configured brand in metadata", async ({ page }) => {
  await page.request.post("/studio/login", {
    form: { username: "e2e-owner", password: "e2e-password" },
  });
  const draft = await page.request.post("/studio/artworks", {
    multipart: {
      title: "Brand metadata artwork",
      slug: "brand-metadata-artwork",
      alt_text: "A branding test image",
      image: { name: "test.png", mimeType: "image/png", buffer: readFileSync("e2e/fixtures/test.png") },
    },
  });
  expect(draft.ok()).toBeTruthy();
  const published = await page.request.post("/studio/artworks/brand-metadata-artwork/publish");
  expect(published.ok()).toBeTruthy();

  await page.goto("/artwork/brand-metadata-artwork");
  await expect(page).toHaveTitle(`Brand metadata artwork | ${siteName}`);
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    "content",
    `Artwork from the ${siteName} collection.`,
  );
});
