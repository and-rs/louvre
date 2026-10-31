import { defineConfig, devices } from "@playwright/test";

const baseURL = `http://127.0.0.1:${process.env.PORT ?? "3300"}`;
const s3MockPort = process.env.S3_MOCK_PORT ?? "19000";

export default defineConfig({
  testDir: "./e2e",
  retries: process.env.CI ? 1 : 0,
  reporter: [["list"], ["html", { outputFolder: "playwright-report", open: "never" }]],
  outputDir: "test-results",
  use: {
    baseURL,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: [
    {
      command: "node e2e/s3-mock.mjs",
      url: `http://127.0.0.1:${s3MockPort}/health`,
      reuseExistingServer: false,
      timeout: 30_000,
    },
    {
      command: "cargo run -p louvre-site --bin louvre-server --features dev",
      url: `${baseURL}/health`,
      reuseExistingServer: false,
      timeout: 120_000,
      gracefulShutdown: { signal: "SIGTERM", timeout: 5_000 },
    },
  ],
});
