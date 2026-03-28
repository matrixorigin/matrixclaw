import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const tauriConfigPath = resolve(import.meta.dirname, "../src-tauri/tauri.conf.json");

test("tauri bundle configuration includes the built UI assets", () => {
    const tauriConfig = JSON.parse(readFileSync(tauriConfigPath, "utf8"));
    const resources = tauriConfig.bundle?.resources ?? [];
    const targets = tauriConfig.bundle?.targets ?? [];

    assert.ok(Array.isArray(resources), "bundle.resources must remain an array");
    assert.ok(
        resources.includes("../../../ui/build/**/*"),
        "desktop bundle must include the built Svelte UI assets",
    );
    assert.ok(
        Array.isArray(targets) && targets.includes("deb"),
        "desktop bundle targets must include the Debian package",
    );
});
