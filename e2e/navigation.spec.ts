import { expect, test } from "@playwright/test";
import { publishFixture } from "./artwork-cli";

const siteName = process.env.SITE_NAME?.trim() || "Louvre";

for (const width of [1280, 390]) {
  test(`boosted navigation and restored gallery work at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 844 });
    const slug = `navigation-artwork-${width}`;
    publishFixture(slug, slug, "First navigation image", "Second navigation image");

    await page.goto("/");
    await expect(page.locator('script[src*="/static/js/htmx-4.0.1.min.js"]')).toHaveCount(1);
    expect(await page.evaluate(() => window.htmx.version)).toBe("4.0.1");
    await page.evaluate(() => { window.navigationMarker = "same document"; });
    const request = page.waitForRequest((request) => request.url().endsWith(`/artwork/${slug}`));
    await page.locator(`a[href="/artwork/${slug}"]`).first().click();
    expect((await request).headers()["hx-boosted"]).toBe("true");
    await expect(page).toHaveURL(new RegExp(`/artwork/${slug}$`));
    await expect(page).toHaveTitle(`${slug} | ${siteName}`);
    expect(await page.evaluate(() => window.navigationMarker)).toBe("same document");
    await page.getByRole("button", { name: "Next image" }).click();
    await expect(page.getByRole("status")).toHaveText("Image 2 of 2");

    await page.getByRole("navigation").getByRole("link", { name: siteName, exact: true }).click();
    await expect(page).toHaveURL(/\/$/);
    const historyRequest = page.waitForRequest((request) =>
      request.url().endsWith(`/artwork/${slug}`) && request.headers()["hx-history-restore-request"] === "true"
    );
    await page.goBack();
    await historyRequest;
    await expect(page).toHaveTitle(`${slug} | ${siteName}`);
    // Refetched history content must get fresh gallery event listeners.
    await expect(page.getByRole("status")).toHaveText("Image 1 of 2");
    await page.getByRole("button", { name: "Next image" }).click();
    await expect(page.getByRole("status")).toHaveText("Image 2 of 2");
    await page.goForward();
    await expect(page).toHaveTitle(siteName);
    expect(await page.evaluate(() => window.navigationMarker)).toBe("same document");
    await expect(page.locator("main")).toHaveCount(1);
    await expect(page.getByRole("navigation", { name: "Main navigation", exact: true })).toHaveCount(1);

    // Exercise the shared navigation policy for error pages and raw assets.
    await page.evaluate(() => {
      const link = document.createElement("a");
      link.href = "/missing-navigation-page";
      link.textContent = "Missing page";
      document.querySelector("main").append(link);
      window.htmx.process(link);
    });
    await page.getByRole("link", { name: "Missing page", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Page not found" })).toBeVisible();
    await expect(page).toHaveTitle(`Not found | ${siteName}`);
    await page.evaluate(() => {
      const link = document.createElement("a");
      link.href = "/static/favicon/favicon-32x32.png";
      link.textContent = "Raw asset";
      link.setAttribute("hx-boost", "false");
      document.querySelector("main").append(link);
      window.htmx.process(link);
    });
    const assetRequest = page.waitForRequest((request) => request.isNavigationRequest());
    await page.getByRole("link", { name: "Raw asset" }).click();
    expect((await assetRequest).headers()["hx-request"]).toBeUndefined();
    await expect(page).toHaveURL(/favicon-32x32\.png$/);
    const dimensions = await page.evaluate(async () => {
      const icon = new Image();
      icon.src = location.href;
      await icon.decode();
      return [icon.naturalWidth, icon.naturalHeight];
    });
    expect(dimensions).toEqual([32, 32]);
  });
}

test("failed boosted requests preserve the current page and URL", async ({ page }) => {
  await page.goto("/");
  for (const status of [403, 500]) {
    const path = `/navigation-error-${status}`;
    await page.route(`**${path}`, (route) => route.fulfill({
      status,
      contentType: "text/html",
      body: "<title>Failed request</title><main><h1>Failed request</h1></main>",
    }));
    await page.evaluate((path) => {
      const link = document.createElement("a");
      link.href = path;
      link.textContent = "Failed navigation";
      document.querySelector("main").append(link);
      window.htmx.process(link);
    }, path);
    await Promise.all([
      page.evaluate(() => new Promise<void>((resolve) => {
        document.addEventListener("htmx:finally:request", () => resolve(), { once: true });
      })),
      page.getByRole("link", { name: "Failed navigation", exact: true }).click(),
    ]);
    await expect(page).toHaveURL(/\/$/);
    await expect(page).toHaveTitle(siteName);
    await expect(page.getByRole("heading", { name: "Failed request", exact: true })).toHaveCount(0);
    await page.getByRole("link", { name: "Failed navigation", exact: true }).evaluate((link) => link.remove());
  }
});

test("public navigation works without JavaScript", async ({ browser }) => {
  const slug = "no-javascript-artwork";
  publishFixture(slug, slug, "An artwork image");
  const context = await browser.newContext({ javaScriptEnabled: false, baseURL: test.info().project.use.baseURL });
  try {
    const page = await context.newPage();
    await page.goto("/");
    await page.locator(`a[href="/artwork/${slug}"]`).click();
    await expect(page).toHaveTitle(`${slug} | ${siteName}`);
    await expect(page.getByRole("img", { name: "An artwork image" })).toBeVisible();
    await page.getByRole("navigation").getByRole("link", { name: siteName, exact: true }).click();
    await expect(page).toHaveTitle(siteName);
  } finally {
    await context.close();
  }
});
