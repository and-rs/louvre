import { expect, test } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { artwork } from "./artwork-cli";

test("dev reads existing S3 images without writing or deleting them", async ({ page, request }) => {
  const key = "artworks/s3-reference/test.png";
  const images = JSON.stringify([
    {
      alt_text: "Existing S3 artwork",
      position: 0,
      source: { s3_key: key },
      web_versions: [{ s3_key: key, content_type: "image/png", width: 1, height: 1 }],
    },
  ]);
  execFileSync("psql", [
    process.env.DATABASE_URL!,
    "-X",
    "-v",
    "ON_ERROR_STOP=1",
    "-c",
    `INSERT INTO artworks (slug, title, status, images) VALUES
      ('s3-published', 'Existing S3 artwork', 'published', '${images}'::jsonb),
      ('s3-draft', 'Existing S3 draft', 'draft', '${images}'::jsonb)
      ON CONFLICT (slug) DO UPDATE SET images = EXCLUDED.images`,
  ]);

  await page.goto("/artwork/s3-published");
  await expect(page.getByRole("img", { name: "Existing S3 artwork" })).toHaveJSProperty("naturalWidth", 1);

  expect(artwork("show", "s3-draft")).toContain(key);
  artwork("remove-image", "s3-draft", "1");
  expect(artwork("show", "s3-draft")).not.toContain(key);

  const remoteImage = await request.get(`${process.env.S3_ENDPOINT_URL}/louvre-e2e/${key}`);
  expect(remoteImage.status()).toBe(200);
  await page.goto("/artwork/s3-published");
  await expect(page.getByRole("img", { name: "Existing S3 artwork" })).toHaveJSProperty("naturalWidth", 1);
  const health = await request.get(`${process.env.S3_ENDPOINT_URL}/health`);
  expect(await health.json()).toEqual({ mutations: 0 });
});
