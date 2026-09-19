import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { JSDOM, VirtualConsole } from "jsdom";

const source = readFileSync(new URL("../website/app.js", import.meta.url), "utf8");
function portal(userAgent = "Linux") {
    const dom = new JSDOM('<body><div id="modsGrid"></div></body>', { url: "https://luxmc.example/", runScripts: "outside-only", pretendToBeVisual: true, virtualConsole: new VirtualConsole() });
    const window = dom.window;
    Object.defineProperty(window.navigator, "userAgent", { value: userAgent });
    const register = window.document.addEventListener.bind(window.document);
    window.document.addEventListener = (type, ...args) => { if (type !== "DOMContentLoaded") register(type, ...args); };
    window.HTMLDialogElement.prototype.showModal = function () { this.open = true; };
    window.HTMLDialogElement.prototype.close = function () { this.open = false; };
    const timers = new Map();
    let next = 0;
    window.setTimeout = (callback, delay) => { const id = ++next; timers.set(id, { callback, delay }); return id; };
    window.clearTimeout = id => timers.delete(id);
    window.eval(source);
    return { window, timers, close: () => dom.window.close() };
}

test("protocol fallback waits 1500ms and uses the visitor platform", () => {
    for (const [agent, path] of [["Linux", "/download/linux"], ["Windows NT 10.0", "/download/windows"]]) {
        const app = portal(agent);
        try {
            app.window.openLuxmc("luxmc://install/mod?id=sodium&source=modrinth");
            const timer = [...app.timers.values()][0];
            assert.equal(timer.delay, 1500);
            assert.equal(app.window.document.querySelector("dialog"), null);
            timer.callback();
            assert.equal(app.window.document.querySelector("dialog").open, true);
            assert.equal(app.window.document.querySelector("#launcherFallbackDownload").getAttribute("href"), path);
        } finally { app.close(); }
    }
});

test("switching to the launcher cancels the fallback and retries replace timers", () => {
    const app = portal();
    try {
        app.window.openLuxmc("luxmc://join/server?ip=example.org");
        app.window.openLuxmc("luxmc://join/server?ip=other.example.org");
        assert.equal(app.timers.size, 1);
        app.window.dispatchEvent(new app.window.Event("blur"));
        assert.equal(app.timers.size, 0);
        assert.equal(app.window.document.querySelector("dialog"), null);
        app.window.openLuxmc("javascript:alert(1)");
        assert.equal(app.timers.size, 0);
    } finally { app.close(); }
});

test("catalog keeps provider links and safely encodes launcher actions", () => {
    const app = portal();
    try {
        app.window.renderModCards([{ project_id: "safe-id", slug: "safe-id", title: '<img onerror="bad()">', categories: [] }]);
        const link = app.window.document.querySelector("a[data-luxmc]");
        assert.equal(link.getAttribute("href"), "luxmc://install/mod?id=safe-id&source=modrinth");
        assert.ok(app.window.document.querySelector('a[href="https://modrinth.com/mod/safe-id/versions"]'));
        assert.equal(app.window.document.querySelector(".project-title img"), null);
    } finally { app.close(); }
});
