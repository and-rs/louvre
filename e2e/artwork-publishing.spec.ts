import { expect, test } from "@playwright/test";
import { resolve } from "node:path";

test("sign in, create a draft, upload an image, publish it, and view it publicly", async ({ page }) => {
  const title = "E2E Artwork";
  const slug = "e2e-artwork";
  const altText = "A small test artwork";

  await page.goto("/studio/login");
  await page.getByLabel("Username").fill("e2e-owner");
  await page.getByLabel("Password").fill("e2e-password");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page).toHaveURL(/\/studio$/);

  await page.getByRole("link", { name: "New artwork" }).click();
  await page.getByLabel("Title").fill(title);
  await page.getByLabel("URL slug").fill(slug);
  await page.getByLabel("Description (optional)").fill("Created by the browser E2E test.");
  await page.getByLabel("First image (optional)").setInputFiles(resolve("e2e/fixtures/test.png"));
  await page.getByLabel("Alternative text (required with an image)").fill(altText);
  await page.getByRole("button", { name: "Create draft" }).click();
  await expect(page).toHaveURL(new RegExp(`/studio/artworks/${slug}$`));
  await expect(page.getByText(altText)).toBeVisible();

  const secondAltText = "A second test artwork image";
  await page.getByLabel("Image file").setInputFiles(resolve("e2e/fixtures/test.png"));
  await page.getByLabel("Alternative text").fill(secondAltText);
  await page.getByRole("button", { name: "Upload image" }).click();
  await expect(page.getByText(secondAltText)).toBeVisible();

  await page.goto(`/artwork/${slug}`);
  await expect(page.getByRole("heading", { name: "Page not found" })).toBeVisible();

  await page.goto(`/studio/artworks/${slug}`);
  await page.getByRole("button", { name: "Publish artwork" }).click();
  await expect(page.getByText("This artwork is public.")).toBeVisible();
  await page.getByRole("link", { name: "View it on the site." }).click();

  await expect(page.getByRole("heading", { name: title, level: 1 })).toBeVisible();
  const image = page.getByRole("img", { name: altText });
  await expect(image).toBeVisible();
  await expect(image).toHaveJSProperty("naturalWidth", 1);
  await expect(page.getByRole("img", { name: secondAltText })).toBeVisible();

  await page.goto("/");
  await expect(page.getByRole("heading", { name: title, level: 2 })).toBeVisible();

  await page.goto("/studio");
  await page.getByRole("link", { name: "New artwork" }).click();
  await page.getByLabel("Title").fill("E2E Draft Without Image");
  await page.getByLabel("URL slug").fill("e2e-draft-without-image");
  await page.getByRole("button", { name: "Create draft" }).click();
  await expect(page).toHaveURL(/\/studio\/artworks\/e2e-draft-without-image$/);
  await expect(page.getByLabel("Image file")).toBeVisible();
  await expect(page.getByRole("button", { name: "Publish artwork" })).toBeDisabled();
});
