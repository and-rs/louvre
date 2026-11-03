import { expect, test } from "@playwright/test";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

test("sign in, create a draft, upload an image, publish it, and view it publicly", async ({ page }) => {
  const title = "E2E Artwork";
  const originalSlug = "e2e-artwork";
  let slug = originalSlug;
  const altText = "A small test artwork";

  await page.goto("/studio/login");
  await page.getByLabel("Username").fill("e2e-owner");
  await page.getByLabel("Password").fill("e2e-password");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page).toHaveURL(/\/studio$/);

  await page.getByRole("link", { name: "New artwork" }).click();
  await page.getByLabel("Title").fill(title);
  await page.getByLabel("URL slug").fill(slug);
  await page.getByLabel("Collection", { exact: true }).fill("E2E Collection");
  await page.getByLabel("Medium", { exact: true }).fill("Acrylic on canvas");
  await page.getByLabel("Collection number").fill("42");
  await page.getByLabel("Description", { exact: true }).fill("Created by the browser E2E test.");
  await page.getByLabel("Image file").setInputFiles(resolve("e2e/fixtures/test.png"));
  await page.getByLabel("Alternative text", { exact: true }).fill(altText);
  await page.getByRole("button", { name: "Create draft" }).click();
  await expect(page).toHaveURL(new RegExp(`/studio/artworks/${slug}$`));
  await expect(page.locator("[data-image-position='0'] input[name='alt_text']")).toHaveValue(altText);

  const secondAltText = "A second test artwork image";
  const thirdAltText = "A third test artwork image";
  const png = await readFile(resolve("e2e/fixtures/test.png"));
  await page.locator("[data-batch-files]").setInputFiles([
    { name: "batch-two.png", mimeType: "image/png", buffer: png },
    { name: "batch-three.png", mimeType: "image/png", buffer: png },
  ]);
  await page.getByLabel("Alternative text for batch-two.png").fill(secondAltText);
  await page.getByLabel("Alternative text for batch-three.png").fill(thirdAltText);
  const retryUploadId = await page.locator('[data-upload-name="batch-two.png"]').getAttribute("data-upload-id");
  let failFirstBatchUpload = true;
  await page.route("**/studio/artworks/e2e-artwork/images", async (route) => {
    if (failFirstBatchUpload && route.request().postData()?.includes("batch-two.png")) {
      failFirstBatchUpload = false;
      await route.fulfill({ status: 502, contentType: "text/html", body: '<p role="alert">Temporary upload failure.</p>' });
    } else {
      await route.continue();
    }
  });
  await page.getByRole("button", { name: "Upload selected images" }).click();
  const failedUpload = page.locator('[data-upload-name="batch-two.png"]');
  await expect(failedUpload.getByRole("alert")).toHaveText("Temporary upload failure.");
  await expect(page.locator('[data-upload-name="batch-three.png"]')).toContainText("Ready to upload");
  await failedUpload.getByRole("button", { name: "Retry" }).click();
  await expect(page.locator("[data-image-position]")).toHaveCount(3);
  await expect(page.locator('[data-image-position="1"] input[name="alt_text"]')).toHaveValue(secondAltText);
  expect(retryUploadId).toBeTruthy();
  const duplicateUpload = await page.request.post(`/studio/artworks/${slug}/images`, {
    multipart: {
      image: { name: "batch-two.png", mimeType: "image/png", buffer: png },
      alt_text: secondAltText,
      upload_id: retryUploadId!,
    },
  });
  expect(duplicateUpload.ok()).toBeTruthy();
  await page.reload();
  await expect(page.locator("[data-image-position]")).toHaveCount(3);
  await expect(page.locator("[data-image-position='0'] input[name='alt_text']")).toHaveValue(altText);
  await expect(page.getByText("1 × 1").first()).toBeVisible();

  await page.getByRole("button", { name: "Move image 2 earlier" }).click();
  await expect(page.locator("[data-image-position='0'] input[name='alt_text']")).toHaveValue(secondAltText);
  const coverAltText = "Updated cover image description";
  const coverCard = page.locator("[data-image-position='0']");
  await coverCard.locator("input[name='alt_text']").fill(coverAltText);
  await coverCard.getByRole("button", { name: "Save alt text" }).click();
  await expect(page.locator("[data-image-position='0'] input[name='alt_text']")).toHaveValue(coverAltText);
  page.once("dialog", (dialog) => dialog.accept());
  await page.getByRole("button", { name: "Remove", exact: true }).last().click();
  await expect(page.locator("[data-image-position]")).toHaveCount(2);

  await page.getByLabel("Collection", { exact: true }).fill("E2E Collection Revised");
  await page.getByLabel("Medium", { exact: true }).fill("Oil and pigment on linen");
  await page.getByLabel("Collection number").fill("43");
  await page.getByLabel("URL slug").fill("e2e-artwork-renamed");
  await page.getByRole("button", { name: "Save artwork details" }).click();
  slug = "e2e-artwork-renamed";
  await expect(page).toHaveURL(new RegExp(`/studio/artworks/${slug}$`));
  await expect(page.getByLabel("URL slug")).toHaveValue(slug);
  await expect(page.getByLabel("Collection", { exact: true })).toHaveValue("E2E Collection Revised");
  await expect(page.getByLabel("Medium", { exact: true })).toHaveValue("Oil and pigment on linen");
  await expect(page.getByLabel("Collection number")).toHaveValue("43");

  await page.goto("/studio");
  const artworkRow = page.getByRole("listitem").filter({ hasText: title });
  const thumbnail = artworkRow.locator(`img[src="/studio/artworks/${slug}/thumbnail"]`);
  await expect(thumbnail).toBeVisible();
  await expect(thumbnail).toHaveJSProperty("naturalWidth", 1);

  await page.goto(`/artwork/${originalSlug}`);
  await expect(page.getByRole("heading", { name: "Page not found" })).toBeVisible();
  await page.goto(`/artwork/${slug}`);
  await expect(page.getByRole("heading", { name: "Page not found" })).toBeVisible();

  await page.goto(`/studio/artworks/${slug}`);
  await page.getByRole("button", { name: "Publish artwork" }).click();
  await expect(page.getByText("This artwork is public.")).toBeVisible();
  await page.getByRole("link", { name: "View it on the site." }).click();

  await expect(page.getByRole("heading", { name: title, level: 1 })).toBeVisible();
  await expect(page.getByText("E2E Collection Revised")).toBeVisible();
  await expect(page.getByText("Oil and pigment on linen")).toBeVisible();
  await expect(page.getByText("No. 43")).toBeVisible();
  const image = page.getByRole("img", { name: coverAltText });
  await expect(image).toBeVisible();
  await expect(image).toHaveJSProperty("naturalWidth", 1);
  await expect(page.getByRole("img", { name: altText })).toBeVisible();
  await page.getByRole("button", { name: "Next image" }).click();
  await expect(page.getByRole("status")).toHaveText("Image 2 of 2");
  await page.locator("[data-gallery-track]").focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.getByRole("status")).toHaveText("Image 1 of 2");

  await page.goto("/");
  await expect(page.getByRole("heading", { name: title, level: 2 })).toBeVisible();
  await expect(page.getByRole("img", { name: coverAltText })).toHaveCSS("aspect-ratio", "1 / 1");
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator("body")).toHaveJSProperty("scrollWidth", 390);

  await page.goto("/studio");
  await page.getByRole("link", { name: "New artwork" }).click();
  await page.getByLabel("Title").fill("E2E Draft Without Image");
  await page.getByLabel("URL slug").fill("e2e-draft-without-image");
  await page.getByRole("button", { name: "Create draft" }).click();
  await expect(page).toHaveURL(/\/studio\/artworks\/e2e-draft-without-image$/);
  await expect(page.locator("[data-batch-files]")).toBeVisible();
  await expect(page.getByRole("button", { name: "Publish artwork" })).toBeDisabled();
  await page.getByLabel("URL slug").fill(slug);
  await page.getByRole("button", { name: "Save artwork details" }).click();
  await expect(page.getByRole("alert")).toHaveText("That URL slug is already in use.");
  await expect(page.getByLabel("URL slug")).toHaveValue(slug);
  await page.getByLabel("URL slug").fill("e2e-draft-without-image-renamed");
  await page.getByRole("button", { name: "Save artwork details" }).click();
  await expect(page).toHaveURL(/\/studio\/artworks\/e2e-draft-without-image-renamed$/);
  await page.goto("/studio");
  const emptyArtworkRow = page.getByRole("listitem").filter({ hasText: "E2E Draft Without Image" });
  await expect(emptyArtworkRow.getByRole("img", { name: "No image uploaded" })).toBeVisible();

  await page.getByRole("button", { name: "Sign out" }).click();
  await page.goto(`/studio/artworks/${slug}/thumbnail`);
  await expect(page).toHaveURL(/\/studio\/login/);
});
