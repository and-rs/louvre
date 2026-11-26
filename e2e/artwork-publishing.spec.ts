import { expect, test } from "@playwright/test";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { artwork, testImage } from "./artwork-cli";

test("create, edit, and publish artwork from the CLI", async ({ page, request }) => {
  const slug = "quiet-study";
  artwork("new", slug, "Untitled", "--collection", "Studies");
  artwork("edit", slug, "--title", "Quiet Study", "--description", "A charcoal study.");
  expect(artwork("list")).toContain(`${slug}\tdraft\tQuiet Study`);
  expect(artwork("show", slug)).toContain("Collection: Studies");
  expect(run(["publish", slug]).stderr).toContain("at least one image");
  expect(run(["add-image", slug, "-", "Broken image"], { input: "not an image" }).status).toBe(1);

  artwork("add-image", slug, testImage, "First image");
  artwork("add-image", slug, testImage, "Second image");
  artwork("alt-text", slug, "2", "Updated second image");
  artwork("move-image", slug, "2", "1");
  const key = artwork("show", slug).match(/^1\tUpdated second image\tartworks\/quiet-study\/(.+)$/m)?.[1];
  expect(key).toBeTruthy();
  expect((await request.get(`/artwork/${slug}`)).status()).toBe(404);
  expect((await request.get(`/artwork/${slug}/image/${key}`)).status()).toBe(404);
  await page.goto("/");
  await expect(page.getByRole("link", { name: /Quiet Study/ })).toHaveCount(0);

  expect(artwork("publish", slug)).toContain("[published]");
  expect(run(["edit", slug, "--title", "Cannot change"]).status).toBe(1);
  await page.reload();
  await page.getByRole("link", { name: /Quiet Study/ }).click();
  await expect(page.getByRole("heading", { name: "Quiet Study" })).toBeVisible();
  await expect(page.getByRole("img", { name: "Updated second image" })).toHaveJSProperty("naturalWidth", 1);
  expect((await request.get("/studio")).status()).toBe(404);
  expect((await request.post("/studio/artworks")).status()).toBe(404);
});

test("Railway mode is explicit and uploads stdin bytes to S3", async ({ request }) => {
  expect(run(["--railway", "list"]).stderr).toContain("inside the intended Railway app service");
  expect(run(["list"], { env: { DATABASE_URL: "postgres://production.example/louvre" } }).stderr)
    .toContain("require loopback PostgreSQL");
  const env = { RAILWAY_ENVIRONMENT_ID: "e2e", S3_BUCKET: "louvre-e2e-railway" };
  expect(run(["list"], { env }).stderr).toContain("explicitly pass artwork --railway");
  const remote = (args: string[], input?: Buffer) => {
    const result = run(["--railway", ...args], { env, input });
    expect(result.status, result.stderr).toBe(0);
    return result.stdout;
  };
  const slug = "railway-stream";
  remote(["new", slug, "Railway stream"]);
  const bytes = readFileSync(testImage);
  remote(["add-image", slug, "-", "Streamed image"], bytes);
  const key = remote(["show", slug]).match(/^1\tStreamed image\t(.+)$/m)?.[1];
  expect(key).toBeTruthy();
  const object = await request.get(`${process.env.S3_ENDPOINT_URL}/${env.S3_BUCKET}/${key}`);
  expect(object.ok()).toBeTruthy();
  expect(await object.body()).toEqual(bytes);
});

function run(args: string[], options: { env?: NodeJS.ProcessEnv; input?: string | Buffer } = {}) {
  return spawnSync("target/debug/louvre", ["artwork", ...args], {
    encoding: "utf8",
    env: { ...process.env, ...options.env },
    input: options.input,
  });
}
