import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { JSDOM, VirtualConsole } from "jsdom";

const root = fileURLToPath(new URL("../website/", import.meta.url));
const read = (name) => readFileSync(resolve(root, name), "utf8");
const createDom = (html) =>
  new JSDOM(html, {
    url: "https://luxmc.test/",
    runScripts: "outside-only",
    pretendToBeVisual: true,
    virtualConsole: new VirtualConsole(),
  });
const settle = async () => {
  for (let index = 0; index < 10; index++) await Promise.resolve();
};

test("the home page exposes the actual launcher and download without WebGL", () => {
  const dom = createDom(read("index.html"));
  try {
    const document = dom.window.document;
    assert.match(document.querySelector("h1").textContent, /Seu mundo/);
    assert.ok(document.querySelector("#heroPrimaryBtn").href);
    assert.equal(
      document.querySelector(".hero-capture img").getAttribute("src"),
      "assets/captures-3.6/home-window.svg",
    );
    assert.equal(
      document.querySelectorAll("#worldStage,[data-skin-scene],.voxel-fallback")
        .length,
      0,
    );
    assert.doesNotMatch(read("visuals/motion.js"), /heroWorld|skinScene/);
  } finally {
    dom.window.close();
  }
});

test("every redesigned page has working local assets, metadata and a main landmark", () => {
  for (const name of [
    "index.html",
    "skins.html",
    "conta.html",
    "privacidade.html",
    "termos.html",
    "404.html",
  ]) {
    const dom = createDom(read(name)),
      document = dom.window.document;
    try {
      assert.equal(document.querySelectorAll("main").length, 1, name);
      assert.ok(document.querySelector("h1"), name);
      assert.ok(
        document.querySelector('meta[name="description"]')?.content,
        name,
      );
      assert.ok(document.querySelector('link[rel="canonical"]')?.href, name);
      assert.equal(
        document.querySelectorAll(
          'script[src*="profitablerate"],script[src*="highrevenue"],img[src*="complementary"]',
        ).length,
        0,
        name,
      );
      for (const element of document.querySelectorAll(
        'script[src],img[src],source[src],link[rel="stylesheet"],link[rel="preload"]',
      )) {
        const src = element.getAttribute("src") || element.getAttribute("href");
        if (!src || /^(https?:|data:)/.test(src)) continue;
        assert.ok(
          existsSync(
            resolve(
              dirname(resolve(root, name)),
              src.replace(/^\//, "").split("?")[0],
            ),
          ),
          `${name}: ${src}`,
        );
      }
    } finally {
      dom.window.close();
    }
  }
});

test("account validation keeps invalid credentials local and focuses the failing field", async () => {
  const dom = createDom(read("conta.html")),
    window = dom.window;
  let writes = 0;
  window.fetch = async () => {
    writes++;
    return {
      ok: false,
      status: 401,
      json: async () => ({ error: "Sessão expirada" }),
    };
  };
  window.eval(read("account.js"));
  await settle();
  const baseline = writes,
    form = window.document.querySelector("#authForm");
  const submit = () =>
    form.dispatchEvent(
      new window.Event("submit", { bubbles: true, cancelable: true }),
    );
  try {
    submit();
    assert.equal(writes, baseline);
    assert.equal(window.document.activeElement.id, "nickname");
    assert.equal(
      window.document.querySelector("#nickname").getAttribute("aria-invalid"),
      "true",
    );
    window.document.querySelector('[data-mode="register"]').click();
    window.document.querySelector("#nickname").value = "ValidNick";
    window.document.querySelector("#password").value = "Valid-Password-123!";
    window.document.querySelector("#confirmPassword").value =
      "Different-Password-123!";
    submit();
    assert.equal(writes, baseline);
    assert.equal(window.document.activeElement.id, "confirmPassword");
    assert.match(
      window.document.querySelector("#authError").textContent,
      /senhas/i,
    );
    window.document.querySelector("#revealPassword").click();
    assert.equal(window.document.querySelector("#password").type, "text");
    window.document.querySelector("#revealPassword").click();
    assert.equal(window.document.querySelector("#password").type, "password");
    window.document.querySelector('[data-mode="recover"]').click();
    assert.equal(
      window.document.querySelector("#recoveryCode").disabled,
      false,
    );
    window.document.querySelector('[data-mode="login"]').click();
    assert.equal(window.document.querySelector("#recoveryCode").disabled, true);
    assert.equal(
      window.document.querySelector("#confirmPassword").disabled,
      true,
    );
  } finally {
    window.close();
  }
});

test("account service failure restores the form and does not expose deployment details", async () => {
  const dom = createDom(read("conta.html")),
    window = dom.window;
  window.fetch = async () => ({
    ok: false,
    status: 503,
    json: async () => ({
      error: "Internal deployment secret configuration required",
    }),
  });
  window.eval(read("account.js"));
  await settle();
  try {
    window.document.querySelector("#nickname").value = "ValidNick";
    window.document.querySelector("#password").value = "Valid-Password-123!";
    window.document
      .querySelector("#authForm")
      .dispatchEvent(
        new window.Event("submit", { bubbles: true, cancelable: true }),
      );
    assert.equal(window.document.querySelector("#authSubmit").disabled, true);
    await settle();
    assert.equal(window.document.querySelector("#authSubmit").disabled, false);
    assert.match(
      window.document.querySelector("#authError").textContent,
      /temporariamente indisponível/,
    );
    assert.doesNotMatch(
      window.document.querySelector("#authError").textContent,
      /secret/,
    );
  } finally {
    window.close();
  }
});

test("motion preferences and pause reveal terminal content and cancel pending animation work", () => {
  for (const reduced of [false, true]) {
    const dom = createDom(
      '<body><button id="motionToggle"></button><div id="diagnosticBody"><p class="diagnostic-line">Example one</p><p class="diagnostic-line">Example two</p></div><button id="replayDiagnostic"></button></body>',
    );
    const window = dom.window,
      observers = [],
      timers = new Map();
    let next = 0;
    window.matchMedia = () => ({ matches: reduced, addEventListener() {} });
    window.requestIdleCallback = () => 1;
    window.cancelIdleCallback = () => {};
    window.setTimeout = (callback) => {
      const id = ++next;
      timers.set(id, callback);
      return id;
    };
    window.clearTimeout = (id) => timers.delete(id);
    window.IntersectionObserver = class {
      constructor(callback) {
        this.callback = callback;
        observers.push(this);
      }
      observe() {}
      unobserve() {}
      disconnect() {
        this.disconnected = true;
      }
    };
    try {
      window.eval(read("visuals/motion.js"));
      assert.equal(
        window.document.querySelector("#motionToggle").disabled,
        reduced,
      );
      window.document.querySelector("#replayDiagnostic").click();
      if (!reduced) {
        assert.equal(timers.size, 2);
        window.document.querySelector("#motionToggle").click();
      }
      assert.equal(timers.size, 0);
      assert.ok(window.document.body.classList.contains("motion-paused"));
      for (const line of window.document.querySelectorAll(".diagnostic-line"))
        assert.equal(line.style.opacity, "1");
      window.dispatchEvent(new window.Event("pagehide"));
      assert.ok(observers.every((observer) => observer.disconnected));
    } finally {
      window.close();
    }
  }
});

test("a pending tour image cannot replace the screen selected afterward", async () => {
  const dom = createDom('<body><button data-product-screen="/one.png" aria-pressed="true"></button><button data-product-screen="/two.png" aria-pressed="false"></button><img id="productTourImage" src="/one.png"><span id="productScreenLabel"></span></body>');
  const window = dom.window;
  const requests = [];
  window.matchMedia = () => ({ matches: true, addEventListener() {} });
  window.IntersectionObserver = class { observe() {} unobserve() {} disconnect() {} };
  window.Image = class { decode() { return new Promise((resolve) => requests.push(resolve)); } };
  try {
    window.eval(read("visuals/motion.js"));
    const buttons = window.document.querySelectorAll("[data-product-screen]");
    buttons[1].click();
    buttons[0].click();
    requests[0]();
    await settle();
    assert.match(window.document.querySelector("#productTourImage").src, /one\.png$/);
    assert.equal(buttons[0].getAttribute("aria-pressed"), "true");
    buttons[1].click();
    requests[1]();
    await settle();
    assert.match(window.document.querySelector("#productTourImage").src, /two\.png$/);
    assert.equal(buttons[1].getAttribute("aria-pressed"), "true");
  } finally { window.close(); }
});
