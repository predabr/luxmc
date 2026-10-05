import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { JSDOM, VirtualConsole } from "jsdom";

const source = readFileSync(
  new URL("../website/app.js", import.meta.url),
  "utf8",
);
function catalog(fetch) {
  const dom = new JSDOM(
    `<input id="modSearchInput"><button class="search-pill active" data-type="mod"></button><select id="modLoader"><option value="fabric">Fabric</option></select><select id="modGameVersion"><option value="1.21.1">1.21.1</option></select><select id="modSort"><option value="updated">Updated</option></select><p id="modSearchStatus"></p><div id="modsGrid"></div><button id="modPrevious"></button><button id="modNext"></button><span id="modPageNumber"></span>`,
    {
      url: "https://luxmc.example/",
      runScripts: "outside-only",
      virtualConsole: new VirtualConsole(),
    },
  );
  const window = dom.window;
  const register = window.document.addEventListener.bind(window.document);
  window.document.addEventListener = (type, ...args) => {
    if (type !== "DOMContentLoaded") register(type, ...args);
  };
  window.AbortSignal = AbortSignal;
  window.AbortController = AbortController;
  window.fetch = fetch;
  window.eval(
    source + "\nwindow.setCatalogPage = value => catalogPage = value;",
  );
  return window;
}

const hit = (title) => ({
  project_id: title,
  slug: title,
  title,
  categories: [],
  downloads: 10,
});
const response = (hits, total_hits = hits.length) => ({
  ok: true,
  json: async () => ({ hits, total_hits }),
});

test("catalog sends version, loader, sort and page offset to the provider", async () => {
  const requests = [];
  const window = catalog(async (url) => {
    requests.push(new URL(url));
    return response([hit("Sodium")], 30);
  });
  try {
    await window.performModSearch("render");
    assert.deepEqual(JSON.parse(requests[0].searchParams.get("facets")), [
      ["project_type:mod"],
      ["categories:fabric"],
      ["versions:1.21.1"],
    ]);
    assert.equal(requests[0].searchParams.get("index"), "updated");
    assert.equal(window.document.querySelector("#modPrevious").disabled, true);
    assert.equal(window.document.querySelector("#modNext").disabled, false);
    window.setCatalogPage(2);
    await window.performModSearch("render");
    assert.equal(requests[1].searchParams.get("offset"), "12");
    assert.equal(
      window.document.querySelector("#modPageNumber").textContent,
      "2 / 3",
    );
  } finally {
    window.close();
  }
});

test("a late search response cannot replace the newest results", async () => {
  let release;
  const window = catalog(async (url) =>
    new URL(url).searchParams.get("query") === "slow"
      ? new Promise((resolve) => {
          release = resolve;
        })
      : response([hit("Newest")]),
  );
  try {
    const pending = window.performModSearch("slow");
    await window.performModSearch("fast");
    release(response([hit("Stale")]));
    await pending;
    assert.equal(
      window.document.querySelector(".project-title").textContent,
      "Newest",
    );
    assert.equal(
      window.document.querySelector("#modsGrid").getAttribute("aria-busy"),
      "false",
    );
  } finally {
    window.close();
  }
});

test("provider failures show a retry instead of fictitious search results", async () => {
  let fails = true;
  const window = catalog(async () => {
    if (fails) throw new Error("Offline");
    return response([hit("Recovered")]);
  });
  try {
    await window.performModSearch("test");
    assert.equal(window.document.querySelector(".project-card"), null);
    assert.ok(window.document.querySelector(".catalog-retry"));
    assert.equal(window.document.querySelector("#modNext").disabled, true);
    fails = false;
    await window.performModSearch("test");
    assert.equal(
      window.document.querySelector(".project-title").textContent,
      "Recovered",
    );
    assert.equal(window.document.querySelector(".catalog-retry"), null);
  } finally {
    window.close();
  }
});

test("blocked browser storage does not break the language switcher", () => {
  const window = catalog(async () => response([]));
  try {
    Object.defineProperty(window, "localStorage", {
      get() {
        throw new Error("Storage blocked");
      },
    });
    window.eval(
      readFileSync(new URL("../website/i18n.js", import.meta.url), "utf8"),
    );
    window.LuxI18n.setLanguage("en");
    assert.equal(window.document.documentElement.lang, "en");
  } finally {
    window.close();
  }
});
