import { execFileSync } from "node:child_process";

export const testImage = "e2e/fixtures/test.png";

export function artwork(...args: string[]) {
  return execFileSync("target/debug/louvre", ["artwork", ...args], { encoding: "utf8" });
}

export function publishFixture(slug: string, title: string, ...altTexts: string[]) {
  artwork("new", slug, title);
  for (const altText of altTexts) {
    artwork("add-image", slug, testImage, altText);
  }
  artwork("publish", slug);
}
