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

function plansFixture({ plan = 'pro', enabled = false, pending = false, pauseOnCreate = false } = {}) {
  const dom = createDom(read('planos.html'));
  const window = dom.window;
  const calls = [];
  const state = { identity:{id:'test-account',username:'Player',owner:true}, membership:{plan,expiresAt:1900000000,collections:24,presets:20}, checkout:{enabled}, orders:pending ? [{id:'test-order',socialId:'test-account',username:'Player',plan:'plus',amount:990,bank:'Test bank',pixKey:'test-only',days:30,status:'pending',version:0,createdAt:1800000000,updatedAt:1800000000,note:'',receiptHash:null}] : [] };
  window.HTMLDialogElement.prototype.showModal = function () { this.open = true; };
  window.HTMLDialogElement.prototype.close = function () { this.open = false; this.dispatchEvent(new window.Event('close')); };
  window.AbortSignal.timeout = () => new window.AbortController().signal;
  window.fetch = async (url) => {
    const action = url.split('/').pop(); calls.push(action);
    if (action === 'create' && pauseOnCreate) { state.checkout.enabled = false; return {ok:false,status:503,json:async()=>({error:'Os pedidos de planos ainda não estão abertos.'})}; }
    assert.equal(action, 'status');
    return {ok:true,status:200,json:async()=>structuredClone(state)};
  };
  window.eval(read('commerce.js'));
  return { dom, window, calls };
}

test('plans keep the active subscription and block duplicate or paused purchases without page clutter', async () => {
  const {window,calls} = plansFixture();
  try {
    await settle();
    const document = window.document;
    assert.match(document.querySelector('#identity').textContent, /Player · Luxmc Pro/);
    assert.match(document.querySelector('#identity').textContent, /Ativo até/);
    assert.equal(document.querySelectorAll('#orders,#preferences,#profile-customization').length, 0);
    assert.equal(document.querySelector('#identity a[href="/painel-luxmc"]'), null);
    assert.equal(document.querySelector('[data-plan=pro]').textContent, 'Seu plano ativo');
    assert.equal(document.querySelector('[data-plan=plus]').textContent, 'Novos pedidos pausados');
    document.querySelector('[data-plan=pro]').click(); document.querySelector('[data-plan=plus]').click();
    await settle();
    assert.deepEqual(calls, ['status']);
    assert.equal(document.querySelector('#message').textContent, '');
    assert.equal(document.querySelector('#checkout').open, false);
  } finally { window.close(); }
});

test('an existing order remains accessible in a dismissible dialog when new purchases are paused', async () => {
  const {window,calls} = plansFixture({pending:true});
  try {
    await settle();
    const control = window.document.querySelector('[data-plan=plus]');
    assert.equal(control.disabled, false); assert.equal(control.textContent, 'Ver pedido');
    control.click(); await settle();
    assert.equal(window.document.querySelector('#checkout').open, true);
    assert.match(window.document.querySelector('#order-list').textContent, /Esperando verificação/);
    window.document.querySelector('#close-checkout').click();
    assert.equal(window.document.querySelector('#checkout').open, false);
    assert.deepEqual(calls, ['status']);
  } finally { window.close(); }
});

test('a checkout pause after page load refreshes the controls and reports the server reason', async () => {
  const {window,calls} = plansFixture({plan:'free',enabled:true,pauseOnCreate:true});
  try {
    await settle(); window.document.querySelector('[data-plan=pro]').click(); await settle(); await settle();
    assert.deepEqual(calls, ['status','create','status']);
    assert.equal(window.document.querySelector('[data-plan=pro]').disabled, true);
    assert.equal(window.document.querySelector('[data-plan=pro]').textContent, 'Novos pedidos pausados');
    assert.match(window.document.querySelector('#message').textContent, /pedidos de planos ainda não estão abertos/);
    assert.equal(window.document.querySelector('#checkout').open, false);
  } finally { window.close(); }
});

test('site navigation shows the authenticated photo and binds the plan link to the connected account', async () => {
  const dom=createDom(read('index.html')),window=dom.window;
  let signedIn=true;const calls=[];
  window.AbortSignal.timeout=()=>new window.AbortController().signal;
  window.fetch=async(url,options={})=>{calls.push({url,options});if(options.method==='POST'){assert.equal(options.headers['X-Luxmc-CSRF'],'test-csrf');signedIn=false;return{ok:true};}return{ok:true,json:async()=>({user:signedIn?{id:'profile-a',username:'Player',displayName:'<script>Creator</script>',avatarUrl:'/assets/avatar-steve.png',skinUrl:null,canManageAccount:true,membership:{plan:'pro',expiresAt:1900000000}}:null,csrfToken:'test-csrf'})};};
  window.eval(read('session.js'));await settle();
  try {
    const document=window.document;
    assert.equal(document.querySelector('[data-site-account]').dataset.accountId,'profile-a');
    assert.equal(document.querySelector('.site-account-name').textContent,'<script>Creator</script>');
    assert.equal(document.querySelectorAll('[data-site-account] script').length,0);
    assert.equal(document.querySelector('.site-account-menu img').getAttribute('src'),'/assets/avatar-steve.png');
    assert.equal(document.querySelector('.site-account-dropdown a').getAttribute('href'),'/planos');
    document.querySelector('.site-account-dropdown button').click();await settle();await settle();
    assert.equal(document.querySelector('.site-account-menu'),null);
    assert.equal(document.querySelector('[data-site-account] a').textContent,'Entrar');
    assert.equal(calls.filter(call=>call.options.method==='POST').length,1);
  } finally {window.close();}
});

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
    "conta.html",
    "privacidade.html",
    "termos.html",
    "assinatura.html",
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

test("Skin Studio is removed from the website while old links redirect home", () => {
  assert.equal(existsSync(resolve(root, "skins.html")), false);
  assert.equal(existsSync(resolve(root, "studio.js")), false);
  for (const name of ["index.html", "conta.html", "termos.html", "privacidade.html", "404.html"]) {
    const dom = createDom(read(name));
    assert.equal(dom.window.document.querySelectorAll('a[href^="/skins"],.identity-section').length, 0);
    dom.window.close();
  }
  assert.doesNotMatch(read("sitemap.xml"), /\/skins/);
  assert.match(read("_redirects"), /\/skins\s+\/\s+301/);
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
      json: async () => ({ error: "SessÃ£o expirada" }),
    };
  };
  window.LuxI18n = {getLanguage: () => "pt"};
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
  window.LuxI18n = {getLanguage: () => "pt"};
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
    await settle();
    assert.equal(requests.length, 1, "a successfully decoded screen is reused on the next selection");
    assert.match(window.document.querySelector("#productTourImage").src, /two\.png$/);
    assert.equal(buttons[1].getAttribute("aria-pressed"), "true");
  } finally { window.close(); }
});

test("a failed tour image can be retried without changing the selected screen early", async () => {
  const dom = createDom('<body><button data-product-screen="/one.png" aria-pressed="true"></button><button data-product-screen="/two.png" aria-pressed="false"></button><img id="productTourImage" src="/one.png"><span id="productScreenLabel"></span></body>');
  const window = dom.window;
  const requests = [];
  window.matchMedia = () => ({ matches: true, addEventListener() {} });
  window.IntersectionObserver = class { observe() {} unobserve() {} disconnect() {} };
  window.Image = class { decode() { return new Promise((resolve, reject) => requests.push({resolve, reject})); } };
  try {
    window.eval(read("visuals/motion.js"));
    const buttons = window.document.querySelectorAll("[data-product-screen]");
    buttons[1].click();
    requests[0].reject(new Error("Connection interrupted"));
    await settle();
    assert.match(window.document.querySelector("#productTourImage").src, /one\.png$/);
    assert.equal(buttons[0].getAttribute("aria-pressed"), "true");
    buttons[1].click();
    assert.equal(requests.length, 2);
    requests[1].resolve();
    await settle();
    assert.match(window.document.querySelector("#productTourImage").src, /two\.png$/);
    assert.equal(buttons[1].getAttribute("aria-pressed"), "true");
  } finally { window.close(); }
});

test('account page follows English and Spanish without clearing entered credentials', async () => {
  const dom = createDom(read("conta.html")), window = dom.window;
  window.localStorage.setItem('luxmc_lang', 'en');
  window.fetch = async () => ({ok:false,status:401,json:async()=>({error:'Session expired'})});
  window.eval(read('i18n.js'));
  window.eval(read('account.js'));
  await settle();
  try {
    assert.match(window.document.querySelector('#authTitle').textContent, /Sign in/);
    assert.match(window.document.querySelector('#microsoftLogin').textContent, /Sign in with Microsoft/);
    window.document.querySelector('#nickname').value = 'ValidNick';
    window.document.querySelector('#password').value = 'keep-private-value';
    window.LuxI18n.setLanguage('es');
    assert.match(window.document.querySelector('#authTitle').textContent, /Entra en tu cuenta/);
    assert.match(window.document.querySelector('#microsoftLogin').textContent, /Entrar con Microsoft/);
    assert.equal(window.document.querySelector('#nickname').value, 'ValidNick');
    assert.equal(window.document.querySelector('#password').value, 'keep-private-value');
  } finally { window.close(); }
});
