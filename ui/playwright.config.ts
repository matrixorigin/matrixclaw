import { defineConfig } from "@playwright/test";

const baseURL = process.env.MATRIXCLAW_BASE_URL ?? "http://127.0.0.1:38495";
const webServer = process.env.MATRIXCLAW_BASE_URL
    ? undefined
    : {
          command:
              'bash -lc \'TMP_HOME=/tmp/matrixclaw-playwright-home; rm -rf "$TMP_HOME"; mkdir -p "$TMP_HOME"; HOME="$TMP_HOME" MATRIXCLAW_LLM_MODEL=moonshotai/kimi-k2.5 ../target/debug/matrixclaw serve --fixture demo\'',
          url: `${baseURL}/healthz`,
          reuseExistingServer: true,
          timeout: 30_000
      };

export default defineConfig({
    testDir: "./tests",
    timeout: 30_000,
    webServer,
    use: {
        baseURL,
        browserName: "firefox",
        headless: true,
        screenshot: "only-on-failure"
    },
    reporter: [["list"]]
});
