import { defineConfig } from "@playwright/test";

const baseURL = process.env.MATRIXCLAW_BASE_URL ?? "http://127.0.0.1:38495";

export default defineConfig({
    testDir: "./tests",
    timeout: 30_000,
    use: {
        baseURL,
        browserName: "firefox",
        headless: true,
        screenshot: "only-on-failure"
    },
    reporter: [["list"]]
});
