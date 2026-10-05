const GITHUB_REPO = "predabr/luxmc";
const MODRINTH_API = "https://api.modrinth.com/v2";

let currentCategory = "mod";
let searchDebounce = null;
let searchController = null;
let catalogPage = 1;
let catalogTotal = 0;
const catalogPageSize = 12;
let detectedOS = "linux";
const directDownloadUrls = {
  linux: "/download/linux",
  windows: "/download/windows",
  deb: "/download/deb",
  rpm: "/download/rpm",
  macos: "/download/macos",
};

function startApp() {
  if (document.getElementById("heroPrimaryBtn")) {
    initOSDetection();
    void initGitHubRelease();
  }
  if (document.getElementById("modsGrid")) void initModrinthExplorer();
  initNavbarScroll();
  initKeyboardShortcuts();
  initMobileNav();
  initServerStatusChecker();
  initWebsiteControls();
}
if (document.readyState === "loading")
  document.addEventListener("DOMContentLoaded", startApp);
else startApp();
function getOSDownloadLabel(os) {
  const i18n = window.LuxI18n;
  if (i18n && i18n.t) {
    return i18n.t(`download.btn.${os}`);
  }
  const fallbackLabels = {
    windows: "Baixar para Windows",
    macos: "Conferir pacote macOS",
    deb: "Baixar para Debian / Ubuntu",
    rpm: "Baixar para Fedora",
    linux: "Baixar para Linux",
  };
  return fallbackLabels[os] || fallbackLabels.linux;
}

function initOSDetection() {
  const ua = navigator.userAgent || "";
  const platform = navigator.platform || "";
  if (/Win|Windows/i.test(ua) || /Win/i.test(platform)) {
    detectedOS = "windows";
  } else if (/Macintosh|Mac OS X|MacIntel/i.test(ua) || /Mac/i.test(platform)) {
    detectedOS = "macos";
  } else if (/Ubuntu|Debian/i.test(ua)) {
    detectedOS = "deb";
  } else if (/Fedora/i.test(ua)) {
    detectedOS = "rpm";
  } else {
    detectedOS = "linux";
  }

  function updateDownloadBtn() {
    const button =
      document.getElementById("primaryDownloadBtn") ||
      document.getElementById("heroPrimaryBtn");
    const label = document.getElementById("heroBtnLabel");
    const downloadLabel = getOSDownloadLabel(detectedOS);
    const mobile = /Android|iPhone|iPad/i.test(ua);
    if (button) {
      button.href = mobile
        ? "#download"
        : directDownloadUrls[detectedOS] || "/download/linux";
      if (!label) button.textContent = downloadLabel;
    }
    if (label)
      label.textContent = mobile
        ? catalogText("download.choose", "Escolher plataforma")
        : downloadLabel;

    document
      .querySelectorAll(".download-card")
      .forEach((c) => c.classList.remove("is-detected"));
    const activeCardId =
      detectedOS === "deb" || detectedOS === "rpm"
        ? "card-linux"
        : `card-${detectedOS}`;
    const card = document.getElementById(activeCardId);
    if (card && !mobile) {
      card.classList.add("is-detected");
      let badge = card.querySelector(".detected-system-badge");
      if (!badge) {
        badge = document.createElement("div");
        badge.className = "detected-system-badge";
        card.prepend(badge);
      }
      const recText =
        (window.LuxI18n && window.LuxI18n.t("download.recommended")) ||
        "SEU SISTEMA";
      badge.textContent = recText;
    }
  }

  updateDownloadBtn();
  window.addEventListener("luxmc-language-changed", updateDownloadBtn);
}

async function initGitHubRelease() {
  try {
    let data = null;
    try {
      const edgeRes = await fetch("/api/latest-release", {
        signal: AbortSignal.timeout(10000),
      });
      if (edgeRes.ok) data = await edgeRes.json();
    } catch {}

    if (!data || !data.tag_name) {
      const ghRes = await fetch(
        `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
        { signal: AbortSignal.timeout(10000) },
      );
      if (ghRes.ok) data = await ghRes.json();
    }

    if (!data) throw new Error("Release indisponível");

    const tag = typeof data.tag_name === "string" ? data.tag_name : "v3.0.0";
    window.LuxLatestVersion = tag;
    document.querySelectorAll(".live-version-tag").forEach((el) => {
      el.textContent = tag;
    });

    const { assetFor } = await import("./lib/releases.js");
    const appImage = assetFor(data.assets, "linux");
    if (appImage) {
      if (appImage.browser_download_url)
        directDownloadUrls.linux = appImage.browser_download_url;
      const link = document.getElementById("downloadAppImageLink");
      if (link) link.href = appImage.browser_download_url;
      if (detectedOS === "linux") {
        const btn =
          document.getElementById("primaryDownloadBtn") ||
          document.getElementById("heroPrimaryBtn");
        if (btn) btn.href = appImage.browser_download_url;
      }
      const meta = document.getElementById("appImageSize");
      if (meta) {
        const sizeMb = (appImage.size / (1024 * 1024)).toFixed(0);
        meta.innerText = `Portátil · ${sizeMb} MB`;
      }
      const codeSnippet = document.getElementById("appImageCodeSnippet");
      if (codeSnippet) {
        codeSnippet.textContent = `chmod +x -- ${shellQuote(appImage.name)} && ${shellQuote("./" + appImage.name)}`;
      }
    }

    const deb = assetFor(data.assets, "deb");
    if (deb) {
      if (deb.browser_download_url)
        directDownloadUrls.deb = deb.browser_download_url;
      const link = document.getElementById("downloadDebLink");
      if (link) link.href = deb.browser_download_url;
      const meta = document.getElementById("debSize");
      if (meta) {
        const sizeMb = (deb.size / (1024 * 1024)).toFixed(0);
        meta.innerText = `Instalador .deb · ${sizeMb} MB`;
      }
    }

    const rpm = assetFor(data.assets, "rpm");
    if (rpm) {
      if (rpm.browser_download_url)
        directDownloadUrls.rpm = rpm.browser_download_url;
      const link = document.getElementById("downloadRpmLink");
      if (link) link.href = rpm.browser_download_url;
    }

    const exe = assetFor(data.assets, "windows");
    if (exe) {
      if (exe.browser_download_url)
        directDownloadUrls.windows = exe.browser_download_url;
      const link = document.getElementById("downloadExeLink");
      if (link) link.href = exe.browser_download_url;
      if (detectedOS === "windows") {
        const btn =
          document.getElementById("primaryDownloadBtn") ||
          document.getElementById("heroPrimaryBtn");
        if (btn) btn.href = exe.browser_download_url;
      }
      const meta = document.getElementById("exeSize");
      if (meta) {
        const sizeMb = (exe.size / (1024 * 1024)).toFixed(0);
        meta.innerText = `Instalador .exe · ${sizeMb} MB`;
      }
    }
    const macos = assetFor(data.assets, "macos");
    const macLink = document.getElementById("downloadMacLink");
    if (macos) {
      directDownloadUrls.macos = macos.browser_download_url;
      if (macLink) macLink.href = macos.browser_download_url;
    } else if (macLink) {
      macLink.href = `https://github.com/${GITHUB_REPO}/releases/latest`;
      macLink.textContent = "Conferir disponibilidade macOS";
    }
    const heroAsset = { windows: exe, macos, deb, rpm, linux: appImage }[
      detectedOS
    ];
    const heroButton =
      document.getElementById("primaryDownloadBtn") ||
      document.getElementById("heroPrimaryBtn");
    if (heroButton && !/Android|iPhone|iPad/i.test(navigator.userAgent))
      heroButton.href =
        heroAsset?.browser_download_url ||
        `https://github.com/${GITHUB_REPO}/releases/latest`;
    const heroSize = document.getElementById("heroReleaseSize");
    if (heroSize && heroAsset && Number.isFinite(heroAsset.size))
      heroSize.textContent = `${(heroAsset.size / (1024 * 1024)).toFixed(1)} MB`;
  } catch (e) {
    document.querySelectorAll(".live-version-tag").forEach((el) => {
      el.textContent = window.LuxLatestVersion || "v3.0.0";
    });
    console.debug("GitHub API fetch fallback:", e);
  }
}

async function initModrinthExplorer() {
  const input = document.getElementById("modSearchInput");
  const pills = document.querySelectorAll(".search-pill");

  pills.forEach((pill) => {
    pill.addEventListener("click", () => {
      pills.forEach((p) => p.classList.remove("active"));
      pill.classList.add("active");
      currentCategory = pill.dataset.type || "mod";
      clearTimeout(searchDebounce);
      catalogPage = 1;
      pills.forEach((p) => p.setAttribute("aria-pressed", String(p === pill)));
      performModSearch(input ? input.value : "");
    });
  });

  if (input) {
    input.addEventListener("input", (e) => {
      clearTimeout(searchDebounce);
      searchController?.abort();
      catalogPage = 1;
      searchDebounce = setTimeout(() => {
        performModSearch(e.target.value.trim());
      }, 350);
    });
  }

  pills.forEach((pill) =>
    pill.setAttribute(
      "aria-pressed",
      String(pill.classList.contains("active")),
    ),
  );
  const search = () => {
    clearTimeout(searchDebounce);
    catalogPage = 1;
    void performModSearch(input?.value.trim() || "");
  };
  ["modLoader", "modGameVersion", "modSort"].forEach((id) =>
    document.getElementById(id)?.addEventListener("change", search),
  );
  document.getElementById("modReset")?.addEventListener("click", () => {
    if (input) input.value = "";
    ["modLoader", "modGameVersion"].forEach((id) => {
      const field = document.getElementById(id);
      if (field) field.value = "";
    });
    const sort = document.getElementById("modSort");
    if (sort) sort.value = "relevance";
    search();
  });
  const navigate = (direction) => {
    clearTimeout(searchDebounce);
    catalogPage = Math.max(1, catalogPage + direction);
    void performModSearch(input?.value.trim() || "");
  };
  document
    .getElementById("modPrevious")
    ?.addEventListener("click", () => navigate(-1));
  document
    .getElementById("modNext")
    ?.addEventListener("click", () => navigate(1));
  const versionSelect = document.getElementById("modGameVersion");
  void performModSearch(input?.value.trim() || "");
  if (versionSelect) {
    try {
      const response = await fetch(`${MODRINTH_API}/tag/game_version`, {
        signal: AbortSignal.timeout(10000),
      });
      if (!response.ok) throw new Error("Versions unavailable");
      const versions = await response.json();
      for (const version of versions.filter(
        (version) => version.version_type === "release",
      )) {
        versionSelect.add(new Option(version.version, version.version));
      }
    } catch {
      versionSelect.disabled = true;
    }
  }

  window.addEventListener("luxmc-language-changed", () => {
    if (lastRenderedMods && lastRenderedMods.length > 0) {
      renderModCards(lastRenderedMods);
    }
    updateCatalogPagination();
  });
}

let lastRenderedMods = [];

function catalogText(key, fallback) {
  const translated = window.LuxI18n?.t(key);
  return translated && translated !== key ? translated : fallback;
}

function updateCatalogPagination(loading = false) {
  const previous = document.getElementById("modPrevious");
  const next = document.getElementById("modNext");
  const number = document.getElementById("modPageNumber");
  if (previous) previous.disabled = loading || catalogPage <= 1;
  if (next)
    next.disabled = loading || catalogPage * catalogPageSize >= catalogTotal;
  if (number)
    number.textContent = `${catalogPage} / ${Math.max(1, Math.ceil(catalogTotal / catalogPageSize))}`;
}

async function performModSearch(query) {
  searchController?.abort();
  const controller = new AbortController();
  searchController = controller;
  const container = document.getElementById("modsGrid");
  if (!container) return;
  const status = document.getElementById("modSearchStatus");
  container.setAttribute("aria-busy", "true");
  updateCatalogPagination(true);

  const loadingText =
    (window.LuxI18n && window.LuxI18n.t("mods.loading")) ||
    "Buscando mods no Modrinth...";
  container.innerHTML =
    '<div class="skeleton" aria-hidden="true"></div>'.repeat(catalogPageSize);
  if (status) status.textContent = loadingText;

  try {
    const facets = [[`project_type:${currentCategory}`]];
    const loader = document.getElementById("modLoader")?.value;
    const version = document.getElementById("modGameVersion")?.value;
    if (loader) facets.push([`categories:${loader}`]);
    if (version) facets.push([`versions:${version}`]);
    const params = new URLSearchParams({
      query,
      limit: String(catalogPageSize),
      offset: String((catalogPage - 1) * catalogPageSize),
      facets: JSON.stringify(facets),
      index: document.getElementById("modSort")?.value || "relevance",
    });
    const url = `${MODRINTH_API}/search?${params}`;
    const res = await fetch(url, {
      signal: AbortSignal.any([controller.signal, AbortSignal.timeout(10000)]),
    });
    if (!res.ok) throw new Error("Modrinth API error");
    const data = await res.json();

    if (controller.signal.aborted) return;
    if (!Array.isArray(data.hits) || !Number.isFinite(data.total_hits))
      throw new Error("Invalid catalog response");
    catalogTotal = data.total_hits;
    renderModCards(Array.isArray(data.hits) ? data.hits : []);
    if (status)
      status.textContent = `${catalogTotal.toLocaleString()} ${catalogText("mods.results", "resultados")}`;
    updateCatalogPagination();
    container.setAttribute("aria-busy", "false");
    return;
  } catch (e) {
    console.debug("Catalog request failed:", e);
  }

  if (controller.signal.aborted) return;
  catalogTotal = 0;
  lastRenderedMods = [];
  container.setAttribute("aria-busy", "false");
  if (status)
    status.textContent = catalogText(
      "mods.unavailable",
      "Não foi possível consultar o Modrinth. Verifique sua conexão e tente novamente.",
    );
  container.replaceChildren();
  const retry = document.createElement("button");
  retry.className = "catalog-retry";
  retry.type = "button";
  retry.textContent = catalogText("mods.retry", "Tentar novamente");
  retry.addEventListener("click", () => void performModSearch(query));
  container.appendChild(retry);
  updateCatalogPagination();
}

function renderModCards(mods) {
  lastRenderedMods = mods;
  const container = document.getElementById("modsGrid");
  if (!container) return;

  const emptyText =
    (window.LuxI18n && window.LuxI18n.t("mods.empty")) ||
    "Nenhum mod encontrado para essa pesquisa.";
  const byText = (window.LuxI18n && window.LuxI18n.t("mods.by")) || "por";
  const installText =
    (window.LuxI18n && window.LuxI18n.t("mods.install")) || "Instalar no Luxmc";
  const versionsText =
    (window.LuxI18n && window.LuxI18n.t("mods.versions")) || "Ver versões";

  if (mods.length === 0) {
    container.innerHTML = `<div style="grid-column: 1/-1; text-align: center; padding: 40px; color: var(--text-muted);">
      ${escapeHtml(emptyText)}
    </div>`;
    return;
  }

  container.innerHTML = mods
    .map((mod) => {
      const slug = mod.slug || mod.project_id || "mod";
      const title = mod.title;
      const author = mod.author || "Community";
      const desc = mod.description || "";
      const icon = mod.icon_url || "assets/logo.png";
      const downloads = formatNumber(mod.downloads || 0);
      const categories = (mod.categories || []).slice(0, 3);
      const versions = mod.versions || [];
      const compatibility = versions.length
        ? document.getElementById("modGameVersion")?.value || versions.at(-1)
        : "";
      const downloadUrl = `https://modrinth.com/${["mod", "modpack", "resourcepack", "shader"].includes(currentCategory) ? currentCategory : "mod"}/${encodeURIComponent(slug)}/versions`;

      return `
      <div class="project-card spotlight-card reveal active">
        <div>
          <div class="project-header">
            <img src="${escapeHtml(icon)}" alt="${escapeHtml(title)}" loading="lazy" decoding="async" class="project-icon" onerror="this.onerror=null;this.src='assets/logo.png'">
            <div class="project-title-area">
              <h3 class="project-title">${escapeHtml(title)}</h3>
              <p class="project-author">${escapeHtml(byText)} <span>${escapeHtml(author)}</span></p>
            </div>
          </div>
          <p class="project-desc">${escapeHtml(desc)}</p>
          <div class="project-tags">
            ${categories.map((c) => `<span class="project-tag">${escapeHtml(c)}</span>`).join("")}
            ${compatibility ? `<span class="project-tag">MC ${escapeHtml(compatibility)}</span>` : ""}
          </div>
        </div>
        <div class="project-footer">
          <div class="project-stats" title="${escapeHtml(catalogText("mods.downloadCount", "Downloads no Modrinth"))}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            <span>${downloads}</span>
          </div>
          ${["mod", "modpack"].includes(currentCategory) ? `<a class="btn-download-mod" data-luxmc href="luxmc://install/${currentCategory}?id=${encodeURIComponent(mod.project_id || slug)}&amp;source=modrinth">${escapeHtml(installText)}</a>` : ""}
          <a href="${downloadUrl}" target="_blank" rel="noopener noreferrer" class="btn-download-mod">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            ${escapeHtml(versionsText)}
          </a>
        </div>
      </div>
    `;
    })
    .join("");
}

function shellQuote(value) {
  return "'" + String(value).replaceAll("'", "'\"'\"'") + "'";
}

function formatNumber(num) {
  if (num >= 1000000) return (num / 1000000).toFixed(1) + "M";
  if (num >= 1000) return (num / 1000).toFixed(1) + "k";
  return num.toString();
}

function escapeHtml(str) {
  return String(str).replace(
    /[&<>"']/g,
    (m) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      })[m],
  );
}

function initNavbarScroll() {
  const navbar = document.querySelector(".navbar");
  if (!navbar) return;
  const update = () => {
    if (window.scrollY > 20) {
      navbar.classList.add("scrolled");
    } else {
      navbar.classList.remove("scrolled");
    }
  };
  window.addEventListener("scroll", update, { passive: true });
  update();
}

function initKeyboardShortcuts() {
  window.addEventListener("keydown", (e) => {
    if (
      e.key === "/" &&
      !e.ctrlKey &&
      !e.metaKey &&
      !e.altKey &&
      !document.activeElement?.matches(
        "input,textarea,select,[contenteditable=true]",
      )
    ) {
      e.preventDefault();
      const input = document.getElementById("modSearchInput");
      if (input) {
        input.focus();
        input.scrollIntoView({
          behavior: matchMedia("(prefers-reduced-motion: reduce)").matches
            ? "auto"
            : "smooth",
          block: "center",
        });
      }
    }
  });
}

let launcherAttemptCleanup;
function openLuxmc(value) {
  let url;
  try {
    url = new URL(value);
    if (
      url.protocol !== "luxmc:" ||
      !["install", "skin", "join"].includes(url.hostname) ||
      value.length > 8192
    )
      throw new Error("Link inválido");
  } catch {
    return;
  }
  launcherAttemptCleanup?.();
  document.getElementById("launcherFallback")?.close();
  let timer;
  const cleanup = () => {
    clearTimeout(timer);
    document.removeEventListener("visibilitychange", hidden);
    window.removeEventListener("pagehide", cleanup);
    window.removeEventListener("blur", cleanup);
  };
  const hidden = () => {
    if (document.hidden) cleanup();
  };
  launcherAttemptCleanup = cleanup;
  document.addEventListener("visibilitychange", hidden);
  window.addEventListener("pagehide", cleanup, { once: true });
  window.addEventListener("blur", cleanup, { once: true });
  timer = setTimeout(() => {
    cleanup();
    if (!document.hidden) showLauncherFallback();
  }, 1500);
  window.location.href = url.href;
}

function showLauncherFallback() {
  let modal = document.getElementById("launcherFallback");
  if (!modal) {
    modal = document.createElement("dialog");
    modal.id = "launcherFallback";
    modal.className = "launcher-fallback";
    modal.setAttribute("aria-labelledby", "launcherFallbackTitle");
    modal.innerHTML = `<form method="dialog"><button class="btn-secondary" aria-label="Fechar" autofocus>Fechar</button></form>
      <h2 id="launcherFallbackTitle">Continue no Luxmc Launcher</h2>
      <p>O Luxmc não respondeu. Se ainda não estiver instalado, baixe agora para usar a instalação em 1 clique. Se já estiver, permita a abertura do aplicativo no navegador.</p>
      <a class="btn-primary" id="launcherFallbackDownload">Baixar Luxmc</a>`;
    document.body.appendChild(modal);
    modal.addEventListener("click", (event) => {
      if (event.target === modal) {
        const box = modal.getBoundingClientRect();
        if (
          event.clientX < box.left ||
          event.clientX > box.right ||
          event.clientY < box.top ||
          event.clientY > box.bottom
        )
          modal.close();
      }
    });
  }
  const windows = /windows/i.test(navigator.userAgent);
  const link = modal.querySelector("#launcherFallbackDownload");
  link.href = windows ? "/download/windows" : "/download/linux";
  link.textContent = windows
    ? "Baixar para Windows"
    : "Baixar AppImage para Linux";
  if (!modal.open) modal.showModal();
}

document.addEventListener("click", (event) => {
  const link =
    event.target instanceof Element
      ? event.target.closest("a[data-luxmc]")
      : null;
  if (
    !link ||
    event.defaultPrevented ||
    event.ctrlKey ||
    event.metaKey ||
    event.shiftKey ||
    event.altKey
  )
    return;
  event.preventDefault();
  openLuxmc(link.href);
});

function initMobileNav() {
  const toggle = document.getElementById("navMobileToggle");
  const drawer = document.getElementById("mobileNavDrawer");
  if (!toggle || !drawer) return;
  drawer.inert = true;

  const openMenu = () => {
    toggle.classList.add("open");
    toggle.setAttribute("aria-expanded", "true");
    drawer.classList.add("open");
    drawer.setAttribute("aria-hidden", "false");
    drawer.inert = false;
    document.querySelector("main").inert = true;
    document.querySelector("footer").inert = true;
    document.body.style.overflow = "hidden";
    drawer.querySelector("a")?.focus();
  };

  const closeMenu = () => {
    toggle.classList.remove("open");
    toggle.setAttribute("aria-expanded", "false");
    drawer.classList.remove("open");
    drawer.setAttribute("aria-hidden", "true");
    drawer.inert = true;
    document.querySelector("main").inert = false;
    document.querySelector("footer").inert = false;
    document.body.style.overflow = "";
    toggle.focus();
  };

  toggle.addEventListener("click", () => {
    if (drawer.classList.contains("open")) {
      closeMenu();
    } else {
      openMenu();
    }
  });

  drawer.querySelectorAll("a").forEach((link) => {
    link.addEventListener("click", () => {
      closeMenu();
    });
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Tab" && drawer.classList.contains("open")) {
      const elements = [toggle, ...drawer.querySelectorAll("a, button")].filter(
        (element) => element.getClientRects().length > 0,
      );
      const first = elements[0];
      const last = elements.at(-1);
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last?.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first?.focus();
      }
    }
    if (e.key === "Escape" && drawer.classList.contains("open")) {
      closeMenu();
    }
  });

  window.addEventListener(
    "resize",
    () => {
      if (window.innerWidth > 1000 && drawer.classList.contains("open")) {
        closeMenu();
      }
    },
    { passive: true },
  );
}

function showToast(message, icon = "✓") {
  let container = document.getElementById("toastContainer");
  if (!container) {
    container = document.createElement("div");
    container.id = "toastContainer";
    container.className = "toast-container";
    container.setAttribute("aria-live", "polite");
    document.body.appendChild(container);
  }

  const toast = document.createElement("div");
  toast.className = "toast-item";
  toast.innerHTML = `<span class="toast-icon">${icon}</span><span class="toast-message">${escapeHtml(message)}</span>`;
  container.appendChild(toast);

  requestAnimationFrame(() => {
    toast.classList.add("show");
  });

  setTimeout(() => {
    toast.classList.remove("show");
    setTimeout(() => {
      toast.remove();
    }, 300);
  }, 2800);
}
window.showToast = showToast;

function initServerStatusChecker() {
  const input = document.getElementById("serverIpInput");
  const btn = document.getElementById("btnCheckServer");
  const resultBox = document.getElementById("serverResultBox");
  const quickPills = document.querySelectorAll(".quick-server-pill");

  if (!input || !btn || !resultBox) return;

  quickPills.forEach((pill) => {
    pill.addEventListener("click", () => {
      quickPills.forEach((p) => p.classList.remove("active"));
      pill.classList.add("active");
      input.value = pill.dataset.ip || "";
      checkServer(input.value.trim());
    });
  });

  btn.addEventListener("click", () => {
    checkServer(input.value.trim());
  });

  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      checkServer(input.value.trim());
    }
  });

  let lastServerCheck = null;
  let serverGeneration = 0;

  function renderServerResult(address, data, pingMs) {
    if (!resultBox) return;
    const noResponseText =
      (window.LuxI18n && window.LuxI18n.t("servers.noResponse")) ||
      "Servidor não respondeu ao ping";
    const offlineText =
      (window.LuxI18n && window.LuxI18n.t("servers.offline")) ||
      "Offline / Inacessível";
    const offlineDescText =
      (window.LuxI18n && window.LuxI18n.t("servers.offlineDesc")) ||
      "O servidor está desligado, em manutenção ou com proteção contra pings diretos. Verifique se o endereço foi digitado corretamente.";
    const onlineText =
      (window.LuxI18n && window.LuxI18n.t("servers.online")) ||
      "Servidor Online";
    const connectedPlayersText =
      (window.LuxI18n && window.LuxI18n.t("servers.connectedPlayers")) ||
      "Jogadores Conectados";
    const latencyText =
      (window.LuxI18n && window.LuxI18n.t("servers.latency")) ||
      "Tempo de consulta";
    const versionText =
      (window.LuxI18n && window.LuxI18n.t("servers.version")) ||
      "Versão do Servidor";
    const btnCopyIpText =
      (window.LuxI18n && window.LuxI18n.t("servers.btnCopyIp")) || "Copiar IP";
    const btnConnectText =
      (window.LuxI18n && window.LuxI18n.t("servers.btnConnect")) ||
      "Conectar via Luxmc";

    if (!data || !data.online) {
      resultBox.innerHTML = `
        <div class="server-card-content">
          <div class="server-card-top">
            <div class="server-identity">
              <div class="server-favicon" style="display:flex;align-items:center;justify-content:center;color:rgb(var(--danger));font-size:1.4rem;">✕</div>
              <div class="server-name-wrap">
                <div class="server-hostname">${escapeHtml(address)}</div>
                <div class="server-ip-copy-tag">${escapeHtml(noResponseText)}</div>
              </div>
            </div>
            <div class="server-status-pill offline">
              <span class="pill-dot" style="background:rgb(var(--danger));"></span>
              <span>${escapeHtml(offlineText)}</span>
            </div>
          </div>
          <div class="server-motd-container" style="color:rgb(var(--fg-muted));">
            ${escapeHtml(offlineDescText)}
          </div>
        </div>
      `;
      return;
    }

    const onlinePlayers = data.players?.online ?? 0;
    const maxPlayers = data.players?.max ?? 0;
    const versionStr =
      data.version?.name_clean ||
      (typeof data.version === "string"
        ? data.version
        : catalogText("servers.unknown", "Não informado"));
    const motdHtml = escapeHtml(
      Array.isArray(data.motd?.clean)
        ? data.motd.clean.join("\n")
        : data.motd?.clean || address,
    ).replaceAll("\n", "<br>");
    const iconSrc =
      typeof data.icon === "string" &&
      /^(https:\/\/|data:image\/png;base64,)/.test(data.icon)
        ? data.icon
        : "assets/favicon.png";

    resultBox.innerHTML = `
      <div class="server-card-content">
        <div class="server-card-top">
          <div class="server-identity">
            <img src="${escapeHtml(iconSrc)}" alt="Ícone de ${escapeHtml(address)}" class="server-favicon" onerror="this.onerror=null;this.src='assets/favicon.png'">
            <div class="server-name-wrap">
              <div class="server-hostname">${escapeHtml(address)}</div>
              <div class="server-ip-copy-tag">${escapeHtml(address)}</div>
            </div>
          </div>
          <div class="server-status-pill online">
            <span class="pill-dot" style="background:rgb(var(--success));"></span>
            <span>${escapeHtml(onlineText)}</span>
          </div>
        </div>

        <div class="server-metrics-grid">
          <div class="server-metric-item">
            <span class="metric-label">${escapeHtml(connectedPlayersText)}</span>
            <span class="metric-val">
              ${formatNumber(onlinePlayers)} <span style="font-size:0.75rem;color:rgb(var(--fg-muted));font-weight:600;">/ ${formatNumber(maxPlayers)}</span>
            </span>
          </div>
          <div class="server-metric-item">
            <span class="metric-label">${escapeHtml(latencyText)}</span>
            <span class="metric-val" style="color:rgb(var(--success));">
              ${pingMs} <span style="font-size:0.75rem;color:rgb(var(--fg-muted));font-weight:600;">ms</span>
            </span>
          </div>
          <div class="server-metric-item">
            <span class="metric-label">${escapeHtml(versionText)}</span>
            <span class="metric-val" style="font-size:0.88rem;">
              ${escapeHtml(versionStr)}
            </span>
          </div>
        </div>

        <div class="server-motd-container">${motdHtml}</div>

        <div class="server-actions-bar">
          <button type="button" class="btn-server-copy" data-copy-ip="${escapeHtml(address)}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
            <span>${escapeHtml(btnCopyIpText)}</span>
          </button>
          <a href="luxmc://join/server?ip=${encodeURIComponent(address)}" data-luxmc="luxmc://join/server?ip=${encodeURIComponent(address)}" class="btn-server-play">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
            <span>${escapeHtml(btnConnectText)}</span>
          </a>
        </div>
      </div>
    `;

    const copyBtn = resultBox.querySelector("[data-copy-ip]");
    if (copyBtn) {
      copyBtn.addEventListener("click", () => {
        navigator.clipboard.writeText(address).then(() => {
          showToast(`IP "${address}" copiado com sucesso!`);
          const label = copyBtn.querySelector("span");
          if (label) {
            const old = label.textContent;
            label.textContent = "Copiado! ✓";
            setTimeout(() => {
              label.textContent = old;
            }, 2000);
          }
        });
      });
    }
  }

  window.addEventListener("luxmc-language-changed", () => {
    if (lastServerCheck) {
      renderServerResult(
        lastServerCheck.address,
        lastServerCheck.data,
        lastServerCheck.pingMs,
      );
    }
  });

  async function checkServer(address) {
    if (!address) return;
    const requestGeneration = ++serverGeneration;

    const checkingText =
      (window.LuxI18n && window.LuxI18n.t("servers.checking")) ||
      "Consultando status de";
    resultBox.innerHTML = `
      <div style="text-align: center; padding: 24px; color: var(--text-muted);">
        <div style="font-size: 1.6rem; margin-bottom: 8px;">⏳</div>
        <div>${escapeHtml(checkingText)} <strong>${escapeHtml(address)}</strong>...</div>
      </div>
    `;

    const startTs = performance.now();
    let data = null;
    let pingMs = 0;

    try {
      const res = await fetch(
        `https://eu.mc-api.net/v3/server/ping/${encodeURIComponent(address)}`,
        {
          signal: AbortSignal.timeout(6000),
        },
      );
      if (res.ok) {
        const d = await res.json();
        if (d && (d.online === true || d.status === true)) {
          data = {
            online: true,
            icon: d.favicon || d.favicon_base64 || null,
            players: d.players || { online: 0, max: 0 },
            version: { name_clean: d.version?.name || "" },
            motd: {
              clean:
                typeof d.description === "string"
                  ? d.description.replace(/§./g, "")
                  : d.description?.text || "",
            },
          };
          pingMs =
            typeof d.took === "number"
              ? Math.max(1, Math.round(d.took))
              : Math.round(performance.now() - startTs);
        }
      }
    } catch {}

    if (!data || !data.online) {
      try {
        const res1 = await fetch(
          `https://api.mcstatus.io/v2/status/java/${encodeURIComponent(address)}`,
          {
            signal: AbortSignal.timeout(5000),
          },
        );
        if (res1.ok) {
          const d1 = await res1.json();
          if (d1 && d1.online) {
            data = d1;
            pingMs = Math.round(performance.now() - startTs);
          }
        }
      } catch {}
    }

    if (!data || !data.online) {
      try {
        const res2 = await fetch(
          `https://api.mcsrvstat.us/3/${encodeURIComponent(address)}`,
          {
            signal: AbortSignal.timeout(5000),
          },
        );
        if (res2.ok) {
          const d2 = await res2.json();
          if (d2 && d2.online) {
            data = {
              online: true,
              icon: d2.icon || null,
              players: d2.players || { online: 0, max: 0 },
              version: { name_clean: d2.version || "" },
              motd: {
                clean: d2.motd?.clean?.join("\n") || "",
              },
            };
            pingMs = Math.round(performance.now() - startTs);
          }
        }
      } catch {}
    }

    if (requestGeneration !== serverGeneration) return;
    lastServerCheck = { address, data, pingMs };
    renderServerResult(address, data, pingMs);
  }

  const details = document.getElementById("servidores");
  details?.addEventListener("toggle", () => {
    if (details.open && !lastServerCheck) void checkServer(input.value.trim());
  });
}

function initWebsiteControls() {
  const image = document.getElementById("productTourImage");
  const modal = document.getElementById("showcaseModal");
  document.getElementById("btnZoomShowcase")?.addEventListener("click", () => {
    document.getElementById("showcaseModalImg").src = image.src;
    document.getElementById("showcaseModalImg").alt = image.alt;
    modal.showModal();
  });
  const donate = document.getElementById("donateModal");
  document
    .getElementById("openDonate")
    ?.addEventListener("click", () => donate.showModal());
  document
    .getElementById("btnCopyInlinePix")
    ?.addEventListener(
      "click",
      (event) =>
        void copyText(
          document.getElementById("inlinePixKey").value,
          event.currentTarget,
        ),
    );
  document.querySelectorAll(".install-tab").forEach((button) =>
    button.addEventListener("click", () => {
      document.getElementById("terminal-cmd-text").textContent =
        button.dataset.cmd;
      document.querySelectorAll(".install-tab").forEach((tab) => {
        tab.classList.toggle("active", tab === button);
        tab.setAttribute("aria-pressed", String(tab === button));
      });
    }),
  );
  document
    .getElementById("btnCopyTerminalCmd")
    ?.addEventListener(
      "click",
      (event) =>
        void copyText(
          document.getElementById("terminal-cmd-text").textContent,
          event.currentTarget,
        ),
    );
  document.querySelectorAll("dialog").forEach((dialog) =>
    dialog.addEventListener("click", (event) => {
      if (event.target !== dialog) return;
      const rect = dialog.getBoundingClientRect();
      if (
        event.clientX < rect.left ||
        event.clientX > rect.right ||
        event.clientY < rect.top ||
        event.clientY > rect.bottom
      )
        dialog.close();
    }),
  );
}

async function copyText(value, button) {
  try {
    await navigator.clipboard.writeText(value);
    const previous = button.textContent;
    button.textContent = catalogText("copy.done", "Copiado ✓");
    setTimeout(() => {
      button.textContent = previous;
    }, 1800);
  } catch {
    showToast(
      catalogText(
        "copy.failed",
        "Não foi possível copiar. Selecione o texto e copie manualmente.",
      ),
      "!",
    );
  }
}
