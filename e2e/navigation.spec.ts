import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";

const siteName = process.env.SITE_NAME?.trim() || "Louvre";

for (const width of [1280, 390]) {
  test(`boosted navigation and restored gallery work at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 844 });
    await page.request.post("/studio/login", {
      form: { username: "e2e-owner", password: "e2e-password" },
    });
    const slug = `navigation-artwork-${width}`;
    const image = { name: "test.png", mimeType: "image/png", buffer: readFileSync("e2e/fixtures/test.png") };
    const draft = await page.request.post("/studio/artworks", {
      multipart: { title: slug, slug, alt_text: "First navigation image", image },
    });
    expect(draft.ok()).toBeTruthy();
    const upload = await page.request.post(`/studio/artworks/${slug}/images`, {
      multipart: { alt_text: "Second navigation image", image },
    });
    expect(upload.ok()).toBeTruthy();
    expect((await page.request.post(`/studio/artworks/${slug}/publish`)).ok()).toBeTruthy();

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

test("Studio keeps auth forms native and rechecks auth on history navigation", async ({ page, context }) => {
  await page.goto("/studio/login");
  await page.getByLabel("Username").fill("e2e-owner");
  await page.getByLabel("Password").fill("wrong-password");
  const loginRequest = page.waitForRequest((request) =>
    request.url().endsWith("/studio/login") && request.method() === "POST"
  );
  await page.getByRole("button", { name: "Sign in" }).click();
  expect((await loginRequest).isNavigationRequest()).toBeTruthy();
  expect((await loginRequest).headers()["hx-request"]).toBeUndefined();
  await expect(page.getByRole("alert")).toHaveText("Those details did not match.");
  await page.getByLabel("Username").fill("e2e-owner");
  await page.getByLabel("Password").fill("e2e-password");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page).toHaveURL(/\/studio$/);
  await page.getByRole("link", { name: "New artwork" }).click();
  await expect(page).toHaveURL(/\/studio\/artworks\/new$/);
  await page.getByRole("navigation").getByRole("link", { name: siteName, exact: true }).click();
  await expect(page).toHaveURL(/\/$/);
  await page.goBack();
  await expect(page.getByRole("button", { name: "Create draft" })).toBeVisible();
  await page.goBack();
  await expect(page.getByRole("heading", { name: "Studio", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Sign out" }).click();
  await expect(page).toHaveURL(/\/studio\/login$/);

  await page.getByLabel("Username").fill("e2e-owner");
  await page.getByLabel("Password").fill("e2e-password");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page).toHaveURL(/\/studio$/);
  await context.clearCookies();
  await page.getByRole("link", { name: "New artwork" }).click();
  await expect(page).toHaveURL(/\/studio\/login\?next=%2Fstudio%2Fartworks%2Fnew$/);
  await expect(page.getByRole("button", { name: "Sign in" })).toBeVisible();

  await page.getByLabel("Username").fill("e2e-owner");
  await page.getByLabel("Password").fill("e2e-password");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page).toHaveURL(/\/studio$/);
  await page.getByRole("navigation").getByRole("link", { name: siteName, exact: true }).click();
  await expect(page).toHaveURL(/\/$/);
  await context.clearCookies();
  await page.goBack();
  await expect(page).toHaveURL(/\/studio\/login\?next=%2Fstudio$/);
  await expect(page.getByRole("button", { name: "Sign in" })).toBeVisible();
});

test("navigation and sign in work without JavaScript", async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false, baseURL: test.info().project.use.baseURL });
  try {
    const page = await context.newPage();
    await page.goto("/studio/login");
    await page.getByLabel("Username").fill("e2e-owner");
    await page.getByLabel("Password").fill("e2e-password");
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveURL(/\/studio$/);
    await page.getByRole("link", { name: "New artwork" }).click();
    await expect(page.getByRole("button", { name: "Create draft" })).toBeVisible();
    await page.getByRole("navigation").getByRole("link", { name: siteName, exact: true }).click();
    await expect(page).toHaveTitle(siteName);
  } finally {
    await context.close();
  }
});
