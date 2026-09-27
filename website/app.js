
const GITHUB_REPO = "predabr/luxmc";
const MODRINTH_API = "https://api.modrinth.com/v2";

let currentCategory = "mod";
let searchDebounce = null;
let searchController = null;
let detectedOS = "linux";
const directDownloadUrls = {
  linux: "/download/linux",
  windows: "/download/windows",
  deb: "/download/deb",
  rpm: "/download/rpm",
  macos: "/download/macos"
};

let currentGatePlatform = "linux";

const FALLBACK_MODS = [
  {
    slug: "sodium",
    title: "Sodium",
    author: "jellysquid3",
    description: "Motor de renderização moderno e otimizador de gráficos para Fabric que multiplica seus FPS.",
    icon_url: "https://cdn.modrinth.com/data/AANobbMI/icon.png",
    downloads: 32400000,
    categories: ["fabric", "optimization"]
  },
  {
    slug: "iris",
    title: "Iris Shaders",
    author: "coderbot",
    description: "Suporte moderno e ultrarrápido para shaders compatível com o Sodium e alto desempenho.",
    icon_url: "https://cdn.modrinth.com/data/YL57xq9U/icon.png",
    downloads: 25100000,
    categories: ["fabric", "shaders", "optimization"]
  },
  {
    slug: "fabric-api",
    title: "Fabric API",
    author: "modmuss50",
    description: "Biblioteca essencial para a maioria esmagadora de mods desenvolvidos para a plataforma Fabric.",
    icon_url: "https://cdn.modrinth.com/data/P7dR8mSH/icon.png",
    downloads: 48900000,
    categories: ["fabric", "library"]
  },
  {
    slug: "lithium",
    title: "Lithium",
    author: "jellysquid3",
    description: "Otimização geral sem perdas na física do jogo, inteligência artificial dos mobs e chunks.",
    icon_url: "https://cdn.modrinth.com/data/gvQqBUqZ/icon.png",
    downloads: 21500000,
    categories: ["fabric", "optimization"]
  },
  {
    slug: "ferrite-core",
    title: "FerriteCore",
    author: "malte0811",
    description: "Reduz o consumo de memória RAM do Minecraft em até 50% sem perda de performance.",
    icon_url: "https://cdn.modrinth.com/data/u6dRKJwZ/icon.png",
    downloads: 18200000,
    categories: ["fabric", "forge", "optimization"]
  },
  {
    slug: "entityculling",
    title: "Entity Culling",
    author: "tr9zw",
    description: "Usa raytracing assíncrono para deixar de renderizar entidades e blocos escondidos atrás de paredes.",
    icon_url: "https://cdn.modrinth.com/data/NNAgCjsB/icon.png",
    downloads: 14700000,
    categories: ["fabric", "forge", "optimization"]
  }
];

function startApp() {
  initBackgroundParticles();
  initSpotlightCards();
  initMockupTabs();
  initOSDetection();
  initGitHubRelease();
  initModrinthExplorer();
  initShowcaseTabs();
  initFAQ();
  initScrollAnimations();
  initNavbarScroll();
  initKeyboardShortcuts();
  initCookieConsent();
  initDownloadGate();
  initMobileNav();
  initBackToTop();
  initAnimatedCounters();
  initStabilityVisualizer();
  initServerStatusChecker();
  initShowcaseModal();
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", startApp);
} else {
  startApp();
}

function initBackgroundParticles() {
  const canvas = document.getElementById("bgCanvas");
  if (!canvas || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  let width = (canvas.width = window.innerWidth);
  let height = (canvas.height = window.innerHeight);

  window.addEventListener("resize", () => {
    width = canvas.width = window.innerWidth;
    height = canvas.height = window.innerHeight;
  }, { passive: true });

  const count = Math.min(55, Math.floor(window.innerWidth / 24));
  const particles = [];
  const mouse = { x: -1000, y: -1000, active: false };

  window.addEventListener("mousemove", (e) => {
    mouse.x = e.clientX;
    mouse.y = e.clientY;
    mouse.active = true;
  }, { passive: true });

  window.addEventListener("mouseleave", () => {
    mouse.active = false;
  }, { passive: true });

  for (let i = 0; i < count; i++) {
    particles.push({
      x: Math.random() * width,
      y: Math.random() * height,
      vx: (Math.random() - 0.5) * 0.4,
      vy: (Math.random() - 0.5) * 0.4,
      radius: Math.random() * 1.5 + 0.8,
      alpha: Math.random() * 0.35 + 0.15
    });
  }

  function frame() {
    ctx.clearRect(0, 0, width, height);

    for (let i = 0; i < particles.length; i++) {
      const p = particles[i];

      if (mouse.active) {
        const dx = mouse.x - p.x;
        const dy = mouse.y - p.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist > 0 && dist < 130) {
          const force = (130 - dist) / 130;
          p.x -= (dx / dist) * force * 1.6;
          p.y -= (dy / dist) * force * 1.6;
        }
      }

      p.x += p.vx;
      p.y += p.vy;

      if (p.x < 0) p.x = width;
      if (p.x > width) p.x = 0;
      if (p.y < 0) p.y = height;
      if (p.y > height) p.y = 0;

      ctx.beginPath();
      ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(255, 255, 255, ${p.alpha})`;
      ctx.fill();


    }

    requestAnimationFrame(frame);
  }

  requestAnimationFrame(frame);
}

function initSpotlightCards() {
  const cards = document.querySelectorAll(".spotlight-card");
  cards.forEach(card => {
    if (card._hasSpotlight) return;
    card._hasSpotlight = true;
    card.addEventListener("mousemove", (e) => {
      const rect = card.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      card.style.setProperty("--mouse-x", `${x}px`);
      card.style.setProperty("--mouse-y", `${y}px`);
    });
  });
}

function getOSDownloadLabel(os) {
  const i18n = window.LuxI18n;
  if (i18n && i18n.t) {
    if (os === "windows") return i18n.t("download.btn.windows").toUpperCase() + " ↗";
    if (os === "macos") return i18n.t("download.btn.macos").toUpperCase() + " ↗";
    if (os === "deb") return i18n.t("download.btn.deb").toUpperCase() + " ↗";
    if (os === "rpm") return i18n.t("download.btn.rpm").toUpperCase() + " ↗";
    return i18n.t("download.btn.linux").toUpperCase() + " ↗";
  }
  const fallbackLabels = {
    windows: "BAIXAR PARA WINDOWS (.EXE) ↗",
    macos: "BAIXAR PARA MACOS (.DMG) ↗",
    deb: "BAIXAR PARA UBUNTU / DEBIAN (.DEB) ↗",
    rpm: "BAIXAR PARA FEDORA (.RPM) ↗",
    linux: "BAIXAR PARA LINUX (.APPIMAGE) ↗"
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
    const button = document.getElementById("primaryDownloadBtn") || document.getElementById("heroPrimaryBtn");
    const label = document.getElementById("heroBtnLabel");
    const downloadLabel = getOSDownloadLabel(detectedOS);
    if (button) {
      button.href = directDownloadUrls[detectedOS] || "/download/linux";
      if (!label) button.textContent = downloadLabel;
    }
    if (label) label.textContent = downloadLabel;

    document.querySelectorAll(".download-card").forEach(c => c.classList.remove("is-detected"));
    const activeCardId = detectedOS === "deb" || detectedOS === "rpm" ? "card-linux" : `card-${detectedOS}`;
    const card = document.getElementById(activeCardId);
    if (card) {
      card.classList.add("is-detected");
      let badge = card.querySelector(".detected-system-badge");
      if (!badge) {
        badge = document.createElement("div");
        badge.className = "detected-system-badge";
        card.prepend(badge);
      }
      const recText = (window.LuxI18n && window.LuxI18n.t("download.recommended")) || "★ RECOMENDADO PARA VOCÊ";
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
      const edgeRes = await fetch("/api/latest-release", { signal: AbortSignal.timeout(10000) });
      if (edgeRes.ok) data = await edgeRes.json();
    } catch {}

    if (!data || !data.tag_name) {
      const ghRes = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`, { signal: AbortSignal.timeout(10000) });
      if (ghRes.ok) data = await ghRes.json();
    }

    if (!data) throw new Error("Release indisponível");
    
    const tag = typeof data.tag_name === "string" ? data.tag_name : "Versão indisponível";
    document.querySelectorAll(".live-version-tag").forEach(el => {
      el.textContent = tag;
    });

    const { assetFor } = await import("./lib/releases.js");
    const appImage = assetFor(data.assets, "linux");
    if (appImage) {
      if (appImage.browser_download_url) directDownloadUrls.linux = appImage.browser_download_url;
      const link = document.getElementById("downloadAppImageLink");
      if (link) link.href = appImage.browser_download_url;
      if (detectedOS === "linux") {
        const btn = document.getElementById("primaryDownloadBtn") || document.getElementById("heroPrimaryBtn");
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
      if (deb.browser_download_url) directDownloadUrls.deb = deb.browser_download_url;
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
      if (rpm.browser_download_url) directDownloadUrls.rpm = rpm.browser_download_url;
      const link = document.getElementById("downloadRpmLink");
      if (link) link.href = rpm.browser_download_url;
    }

    const exe = assetFor(data.assets, "windows");
    if (exe) {
      if (exe.browser_download_url) directDownloadUrls.windows = exe.browser_download_url;
      const link = document.getElementById("downloadExeLink");
      if (link) link.href = exe.browser_download_url;
      if (detectedOS === "windows") {
        const btn = document.getElementById("primaryDownloadBtn") || document.getElementById("heroPrimaryBtn");
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
    const heroAsset = ({ windows: exe, macos, deb, rpm, linux: appImage })[detectedOS];
    const heroButton = document.getElementById("primaryDownloadBtn") || document.getElementById("heroPrimaryBtn");
    if (heroButton) heroButton.href = heroAsset?.browser_download_url || `https://github.com/${GITHUB_REPO}/releases/latest`;
    const heroSize = document.getElementById("heroReleaseSize");
    if (heroSize && heroAsset) heroSize.textContent = `${(heroAsset.size / (1024 * 1024)).toFixed(1)} MB · Download direto`;
  } catch (e) {
    document.querySelectorAll(".live-version-tag").forEach(el => { el.textContent = "Releases no GitHub"; });
    console.debug("GitHub API fetch fallback:", e);
  }
}

async function initModrinthExplorer() {
  const input = document.getElementById("modSearchInput");
  const pills = document.querySelectorAll(".search-pill");

  pills.forEach(pill => {
    pill.addEventListener("click", () => {
      pills.forEach(p => p.classList.remove("active"));
      pill.classList.add("active");
      currentCategory = pill.dataset.type || "mod";
      performModSearch(input ? input.value : "");
    });
  });

  if (input) {
    input.addEventListener("input", (e) => {
      clearTimeout(searchDebounce);
      searchDebounce = setTimeout(() => {
        performModSearch(e.target.value.trim());
      }, 350);
    });
  }

  window.addEventListener("luxmc-language-changed", () => {
    if (lastRenderedMods && lastRenderedMods.length > 0) {
      renderModCards(lastRenderedMods);
    }
  });

  performModSearch("");
}

let lastRenderedMods = [];

async function performModSearch(query) {
  searchController?.abort();
  const controller = new AbortController();
  searchController = controller;
  const container = document.getElementById("modsGrid");
  if (!container) return;

  const loadingText = (window.LuxI18n && window.LuxI18n.t("mods.loading")) || "Buscando mods no Modrinth...";
  container.innerHTML = `<div style="grid-column: 1/-1; text-align: center; padding: 40px; color: var(--text-muted);">
    <div style="font-size: 1.5rem; margin-bottom: 8px;">⏳</div>
    ${escapeHtml(loadingText)}
  </div>`;

  try {
    let facets = `[["project_type:${currentCategory}"]]`;
    const url = `${MODRINTH_API}/search?query=${encodeURIComponent(query)}&limit=6&facets=${encodeURIComponent(facets)}`;
    const res = await fetch(url, { signal: AbortSignal.any([controller.signal, AbortSignal.timeout(10000)]) });
    if (!res.ok) throw new Error("Modrinth API error");
    const data = await res.json();

    if (controller.signal.aborted) return;
    renderModCards(Array.isArray(data.hits) ? data.hits : []);
    return;
  } catch (e) {
    console.debug("Using fallback mods data:", e);
  }

  if (controller.signal.aborted) return;
  if (currentCategory !== "mod") { renderModCards([]); return; }
  let filtered = FALLBACK_MODS;
  if (query) {
    filtered = FALLBACK_MODS.filter(m => 
      m.title.toLowerCase().includes(query.toLowerCase()) || 
      m.description.toLowerCase().includes(query.toLowerCase())
    );
  }
  renderModCards(filtered);
}

function renderModCards(mods) {
  lastRenderedMods = mods;
  const container = document.getElementById("modsGrid");
  if (!container) return;

  const emptyText = (window.LuxI18n && window.LuxI18n.t("mods.empty")) || "Nenhum mod encontrado para essa pesquisa.";
  const byText = (window.LuxI18n && window.LuxI18n.t("mods.by")) || "por";
  const installText = (window.LuxI18n && window.LuxI18n.t("mods.install")) || "Instalar no Luxmc";
  const versionsText = (window.LuxI18n && window.LuxI18n.t("mods.versions")) || "Ver versões";

  if (mods.length === 0) {
    container.innerHTML = `<div style="grid-column: 1/-1; text-align: center; padding: 40px; color: var(--text-muted);">
      ${escapeHtml(emptyText)}
    </div>`;
    return;
  }

  container.innerHTML = mods.map(mod => {
    const slug = mod.slug || mod.project_id || "mod";
    const title = mod.title;
    const author = mod.author || "Community";
    const desc = mod.description || "";
    const icon = mod.icon_url || "assets/logo.png";
    const downloads = formatNumber(mod.downloads || 0);
    const categories = (mod.categories || []).slice(0, 3);
    const downloadUrl = `https://modrinth.com/${["mod", "modpack", "resourcepack", "shader"].includes(currentCategory) ? currentCategory : "mod"}/${encodeURIComponent(slug)}/versions`;

    return `
      <div class="project-card spotlight-card reveal active">
        <div>
          <div class="project-header">
            <img src="${escapeHtml(icon)}" alt="${escapeHtml(title)}" class="project-icon" onerror="this.src='assets/logo.png'">
            <div class="project-title-area">
              <h4 class="project-title">${escapeHtml(title)}</h4>
              <p class="project-author">${escapeHtml(byText)} <span>${escapeHtml(author)}</span></p>
            </div>
          </div>
          <p class="project-desc">${escapeHtml(desc)}</p>
          <div class="project-tags">
            ${categories.map(c => `<span class="project-tag">${escapeHtml(c)}</span>`).join("")}
          </div>
        </div>
        <div class="project-footer">
          <div class="project-stats">
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
  }).join("");

  initSpotlightCards();
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
  return String(str).replace(/[&<>"']/g, m => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;"
  }[m]));
}

function initShowcaseTabs() {
  const tabs = document.querySelectorAll(".showcase-tab");
  const panels = document.querySelectorAll(".showcase-panel");

  tabs.forEach(tab => {
    tab.addEventListener("click", () => {
      const targetId = tab.dataset.panel;
      tabs.forEach(t => t.classList.remove("active"));
      panels.forEach(p => p.classList.remove("active"));

      tab.classList.add("active");
      const targetPanel = document.getElementById(targetId);
      if (targetPanel) {
        targetPanel.classList.add("active");
      }
    });
  });
}

function initMockupTabs() {
  const btns = document.querySelectorAll(".mockup-nav-btn, .tab-btn");
  const img = document.getElementById("mockupDisplayImg") || document.getElementById("showcaseImage");
  const badge = document.getElementById("mockupTitleBadge") || document.getElementById("showcaseBadgeText");
  if (!btns.length || !img) return;

  let currentIndex = Math.max(0, Array.from(btns).findIndex(button => button.classList.contains("active")));
  let autoTimer = null;
  let userInteracted = false;

  btns.forEach(btn => {
    if (btn.dataset.mockup && btn.dataset.mockup !== "friends" && typeof Image !== "undefined") {
      const preload = new Image();
      preload.src = btn.dataset.mockup;
    }
  });

  function setMockup(index, manual = false) {
    if (manual) userInteracted = true;
    btns.forEach(b => { b.classList.remove("active"); b.setAttribute("aria-pressed", "false"); });
    const btn = btns[index];
    if (!btn) return;
    btn.classList.add("active");
    btn.setAttribute("aria-pressed", "true");

    const newSrc = btn.dataset.mockup;
    const titleKey = btn.dataset.titleKey;
    const title = (titleKey && window.LuxI18n ? window.LuxI18n.t(titleKey) : null) || btn.dataset.title || "Menu Principal";

    img.style.opacity = "0.2";
    img.style.transform = "scale(0.995)";
    setTimeout(() => {
      const social = document.getElementById("socialMockup");
      img.hidden = newSrc === "friends";
      if (social) social.hidden = newSrc !== "friends";
      if (newSrc !== "friends") img.src = newSrc;
      img.style.opacity = "1";
      img.style.transform = "scale(1)";
      if (badge) badge.innerText = title;
    }, 120);

    currentIndex = index;
  }

  btns.forEach((btn, idx) => {
    btn.addEventListener("click", () => {
      clearInterval(autoTimer);
      setMockup(idx, true);
    });
  });

  window.addEventListener("luxmc-language-changed", () => {
    const activeBtn = Array.from(btns).find(b => b.classList.contains("active"));
    if (activeBtn && badge) {
      const titleKey = activeBtn.dataset.titleKey;
      const title = (titleKey && window.LuxI18n ? window.LuxI18n.t(titleKey) : null) || activeBtn.dataset.title || "Menu Principal";
      badge.innerText = title;
    }
  });

  // Auto rotate every 6s if user hasn't clicked
  autoTimer = setInterval(() => {
    if (!userInteracted && !document.hidden && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      const next = (currentIndex + 1) % btns.length;
      setMockup(next);
    }
  }, 6000);

  const mockupContainer = document.querySelector(".window-mockup, .window-showcase");
  if (mockupContainer) {
    mockupContainer.addEventListener("mouseenter", () => clearInterval(autoTimer));
  }
}

function initFAQ() {
  const items = document.querySelectorAll(".faq-item");
  items.forEach(item => {
    const question = item.querySelector(".faq-question");
    if (!question) return;

    question.addEventListener("click", () => {
      const isOpen = item.classList.contains("open");
      items.forEach(other => {
        if (other !== item) other.classList.remove("open");
      });
      item.classList.toggle("open", !isOpen);
    });
  });
}

function initScrollAnimations() {
  const reveals = document.querySelectorAll(".reveal");
  if (!("IntersectionObserver" in window)) {
    reveals.forEach(el => el.classList.add("active"));
    return;
  }

  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        entry.target.classList.add("active");
        observer.unobserve(entry.target);
      }
    });
  }, {
    threshold: 0.08,
    rootMargin: "0px 0px -30px 0px"
  });

  reveals.forEach(el => observer.observe(el));
}

function initNavbarScroll() {
  const navbar = document.querySelector(".navbar");
  if (!navbar) return;
  window.addEventListener("scroll", () => {
    if (window.scrollY > 20) {
      navbar.classList.add("scrolled");
    } else {
      navbar.classList.remove("scrolled");
    }
  }, { passive: true });
}

function initKeyboardShortcuts() {
  window.addEventListener("keydown", (e) => {
    if (e.key === "/" && document.activeElement?.tagName !== "INPUT") {
      e.preventDefault();
      const input = document.getElementById("modSearchInput");
      if (input) {
        input.focus();
        input.scrollIntoView({ behavior: "smooth", block: "center" });
      }
    }
  });
}

function copyCode(text, btnElement) {
  navigator.clipboard.writeText(text).then(() => {
    const orig = btnElement.innerText;
    btnElement.innerText = "Copiado! ✓";
    btnElement.style.background = "#ffffff";
    btnElement.style.color = "#000000";
    showToast("Comando copiado para a área de transferência!");
    setTimeout(() => {
      btnElement.innerText = orig;
      btnElement.style.background = "";
      btnElement.style.color = "";
    }, 2000);
  });
}


let launcherAttemptCleanup;
function openLuxmc(value) {
  let url;
  try {
    url = new URL(value);
    if (url.protocol !== "luxmc:" || !["install", "skin", "join"].includes(url.hostname) || value.length > 8192) throw new Error("Link inválido");
  } catch { return; }
  launcherAttemptCleanup?.();
  document.getElementById("launcherFallback")?.close();
  let timer;
  const cleanup = () => {
    clearTimeout(timer);
    document.removeEventListener("visibilitychange", hidden);
    window.removeEventListener("pagehide", cleanup);
    window.removeEventListener("blur", cleanup);
  };
  const hidden = () => { if (document.hidden) cleanup(); };
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
    modal.addEventListener("click", event => { if (event.target === modal) { const box = modal.getBoundingClientRect(); if (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom) modal.close(); } });
  }
  const windows = /windows/i.test(navigator.userAgent);
  const link = modal.querySelector("#launcherFallbackDownload");
  link.href = windows ? "/download/windows" : "/download/linux";
  link.textContent = windows ? "Baixar para Windows" : "Baixar AppImage para Linux";
  if (!modal.open) modal.showModal();
}

document.addEventListener("click", event => {
  const link = event.target instanceof Element ? event.target.closest("a[data-luxmc]") : null;
  if (!link || event.defaultPrevented || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
  event.preventDefault();
  openLuxmc(link.href);
});

function initCookieConsent() {
  if (typeof localStorage === "undefined" || localStorage.getItem("luxmc_cookie_consent")) return;
  const banner = document.createElement("div");
  banner.id = "cookieConsent";
  banner.className = "cookie-banner";
  banner.setAttribute("role", "dialog");
  banner.setAttribute("aria-label", "Consentimento de cookies e privacidade");
  banner.innerHTML = `
    <p>Utilizamos cookies e tecnologias essenciais para viabilizar e aprimorar sua experiência. Ao continuar navegando, você concorda com a nossa <a href="privacidade.html">Política de Privacidade</a>.</p>
    <div class="cookie-actions">
      <button type="button" class="btn-accept" id="btnAcceptCookie">Concordar e Fechar</button>
    </div>
  `;
  document.body.appendChild(banner);
  document.getElementById("btnAcceptCookie")?.addEventListener("click", () => {
    localStorage.setItem("luxmc_cookie_consent", "true");
    banner.style.opacity = "0";
    banner.style.transform = "translateY(12px)";
    banner.style.transition = "opacity 0.25s ease, transform 0.25s ease";
    setTimeout(() => banner.remove(), 250);
  });
}

let gateProgressTimer = null;
let gateTipTimer = null;

const GATE_TIPS = [
  "O Luxmc consome menos de 100MB de RAM em repouso, garantindo mais FPS para o seu jogo.",
  "Você pode pesquisar e instalar mods do Modrinth com apenas 1 clique direto pelo launcher.",
  "No Linux, o Luxmc ativa suporte nativo a Wayland e MangoHud sem congelamentos.",
  "Sincronize suas skins em 3D e seus amigos através da sua Conta Luxmc.",
  "Suporte total a modpacks pesados no Windows com detecção automática de drivers e runtimes."
];

async function checkAdBlock() {
  const bait = document.createElement("div");
  bait.className = "adsbox pub_300x250 pub_300x250m pub_728x90 text-ad textAd text_ad text_ads text-ads text-ad-links ad-banner ads-banner ad-placement";
  bait.style.cssText = "position:absolute;left:-9999px;top:-9999px;width:10px;height:10px;pointer-events:none;";
  bait.innerHTML = "&nbsp;";
  document.body.appendChild(bait);

  let isBlocked = false;
  await new Promise(r => setTimeout(r, 40));

  if (
    !bait.parentElement ||
    bait.offsetParent === null ||
    bait.clientHeight === 0 ||
    bait.offsetHeight === 0 ||
    window.getComputedStyle(bait).display === "none" ||
    window.getComputedStyle(bait).visibility === "hidden"
  ) {
    isBlocked = true;
  }
  bait.remove();

  if (!isBlocked) {
    try {
      await fetch("https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js", {
        method: "HEAD",
        mode: "no-cors",
        cache: "no-store"
      });
    } catch {
      isBlocked = true;
    }
  }

  return isBlocked;
}

function triggerBrowserDownload(url) {
  if (!url) return;
  const link = document.createElement("a");
  link.href = url;
  link.setAttribute("download", "");
  link.rel = "noopener noreferrer";
  link.style.display = "none";
  document.body.appendChild(link);
  link.click();
  setTimeout(() => link.remove(), 1000);
}

function updateGatePlatformUI(platform) {
  currentGatePlatform = platform === "windows" ? "windows" : "linux";
  const isWindows = currentGatePlatform === "windows";

  const platformName = document.getElementById("gatePlatformName");
  const platformIcon = document.getElementById("gatePlatformIcon");
  const toggleBtn = document.getElementById("gateTogglePlatformBtn");
  const finalBtn = document.getElementById("gateFinalDownloadBtn");

  if (platformName) {
    platformName.textContent = isWindows ? "Windows 10 / 11 (.exe Oficial)" : "Linux AppImage (.AppImage x86_64)";
  }
  if (platformIcon) {
    platformIcon.innerHTML = isWindows
      ? `<svg width="18" height="18" viewBox="0 0 88 88" fill="currentColor"><path d="M0 12.56L35.73 7.69V42.66H0V12.56ZM0 45.34H35.73V80.31L0 75.44V45.34ZM39.06 7.23L88 0V42.66H39.06V7.23ZM39.06 45.34H88V88L39.06 80.77V45.34Z"/></svg>`
      : `<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139z"/></svg>`;
  }
  if (toggleBtn) {
    toggleBtn.textContent = isWindows ? "Mudar para Linux" : "Mudar para Windows";
  }
  const downloadUrl = directDownloadUrls[currentGatePlatform] || (isWindows ? "/download/windows" : "/download/linux");
  if (finalBtn) {
    finalBtn.href = downloadUrl;
  }
}

function initDownloadGate() {
  const modal = document.getElementById("downloadGateModal");
  if (!modal) return;

  const closeBtn = document.getElementById("gateCloseBtn");
  const toggleBtn = document.getElementById("gateTogglePlatformBtn");

  const stopGate = () => {
    if (gateProgressTimer) { clearInterval(gateProgressTimer); gateProgressTimer = null; }
    if (gateTipTimer) { clearInterval(gateTipTimer); gateTipTimer = null; }
    if (modal.open) modal.close();
  };

  closeBtn?.addEventListener("click", stopGate);
  modal.addEventListener("click", (e) => {
    if (e.target === modal) stopGate();
  });
  modal.addEventListener("close", stopGate);

  toggleBtn?.addEventListener("click", () => {
    const nextPlatform = currentGatePlatform === "windows" ? "linux" : "windows";
    updateGatePlatformUI(nextPlatform);
  });

  document.addEventListener("click", (event) => {
    const link = event.target instanceof Element ? event.target.closest("a[href^='/download/'], a[data-open-download], #primaryDownloadBtn") : null;
    if (!link || event.defaultPrevented || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
    if (link.id === "gateFinalDownloadBtn") return;
    event.preventDefault();
    const href = link.getAttribute("href") || "";
    openDownloadGate(href);
  });
}

async function openDownloadGate(target) {
  const modal = document.getElementById("downloadGateModal");
  let requestedPlatform = detectedOS;
  if (typeof target === "string") {
    if (target.includes("windows") || target.endsWith(".exe")) {
      requestedPlatform = "windows";
    } else if (target.includes("linux") || target.endsWith(".AppImage")) {
      requestedPlatform = "linux";
    }
  }

  if (!modal) {
    const fallbackUrl = directDownloadUrls[requestedPlatform] || (requestedPlatform === "windows" ? "/download/windows" : "/download/linux");
    triggerBrowserDownload(fallbackUrl);
    return;
  }

  updateGatePlatformUI(requestedPlatform);

  const adblockView = document.getElementById("gateAdblockView");
  const prepView = document.getElementById("gatePreparingView");
  const readyActions = document.getElementById("gateReadyActions");
  const progressBar = document.getElementById("gateProgressBar");
  if (readyActions) readyActions.hidden = true;
  if (progressBar) progressBar.style.width = "0%";

  if (!modal.open) modal.showModal();

  const isBlocked = await checkAdBlock();
  if (isBlocked) {
    if (adblockView) adblockView.hidden = false;
    if (prepView) prepView.hidden = true;

    const retryBtn = document.getElementById("gateRetryAdblockBtn");
    const retryText = document.getElementById("gateRetryBtnText");
    if (retryBtn) {
      retryBtn.onclick = async () => {
        if (retryText) retryText.textContent = "Verificando...";
        retryBtn.disabled = true;
        await new Promise(r => setTimeout(r, 600));
        const stillBlocked = await checkAdBlock();
        retryBtn.disabled = false;
        if (stillBlocked) {
          if (retryText) retryText.textContent = "AdBlock ainda ativo! Desative no navegador e tente novamente";
          retryBtn.style.animation = "shake 0.4s ease";
          setTimeout(() => { if (retryBtn) retryBtn.style.animation = ""; }, 400);
        } else {
          if (retryText) retryText.textContent = "AdBlock desativado! Liberando...";
          setTimeout(() => {
            if (adblockView) adblockView.hidden = true;
            if (prepView) prepView.hidden = false;
            startDownloadSequence();
          }, 300);
        }
      };
    }
  } else {
    if (adblockView) adblockView.hidden = true;
    if (prepView) prepView.hidden = false;
    startDownloadSequence();
  }
}

function startDownloadSequence() {
  if (gateProgressTimer) clearInterval(gateProgressTimer);
  if (gateTipTimer) clearInterval(gateTipTimer);

  const progressBar = document.getElementById("gateProgressBar");
  const statusText = document.getElementById("gateStatusText");
  const statusSub = document.getElementById("gateStatusSub");
  const stageLabel = document.getElementById("gateStageLabel");
  const secRemaining = document.getElementById("gateSecRemaining");
  const readyActions = document.getElementById("gateReadyActions");
  const finalBtn = document.getElementById("gateFinalDownloadBtn");
  const tipText = document.getElementById("gateTipText");

  let tipIndex = 0;
  gateTipTimer = setInterval(() => {
    tipIndex = (tipIndex + 1) % GATE_TIPS.length;
    if (tipText) {
      tipText.style.opacity = "0";
      setTimeout(() => {
        if (tipText) {
          tipText.textContent = GATE_TIPS[tipIndex];
          tipText.style.opacity = "1";
        }
      }, 200);
    }
  }, 2800);

  const totalDuration = 10000;
  const stepMs = 50;
  let elapsed = 0;

  gateProgressTimer = setInterval(() => {
    elapsed += stepMs;
    const progress = Math.min(100, (elapsed / totalDuration) * 100);

    if (progressBar) progressBar.style.width = `${progress}%`;

    if (progress < 25) {
      if (statusText) statusText.textContent = "Conectando aos servidores de alta velocidade...";
      if (statusSub) statusSub.textContent = "Localizando o nó de distribuição mais rápido próximo a você...";
      if (stageLabel) stageLabel.textContent = "Conexão estabelecida...";
      if (secRemaining) secRemaining.textContent = "Iniciando...";
    } else if (progress < 55) {
      if (statusText) statusText.textContent = "Verificando pacotes e integridade SHA-256...";
      if (statusSub) statusSub.textContent = "Validando assinaturas digitais do binário oficial...";
      if (stageLabel) stageLabel.textContent = "Integridade verificada...";
      if (secRemaining) secRemaining.textContent = "Otimizando...";
    } else if (progress < 82) {
      if (statusText) statusText.textContent = "Otimizando empacotamento com binários nativos...";
      if (statusSub) statusSub.textContent = "Configurando aceleração gráfica e suporte nativo ao seu sistema...";
      if (stageLabel) stageLabel.textContent = "Montando pacote...";
      if (secRemaining) secRemaining.textContent = "Quase pronto...";
    } else if (progress < 100) {
      if (statusText) statusText.textContent = "Preparação concluída! O instalador já vai liberar...";
      if (statusSub) statusSub.textContent = "Finalizando empacotamento para transferência segura...";
      if (stageLabel) stageLabel.textContent = "Liberando arquivo...";
      if (secRemaining) secRemaining.textContent = "Finalizando...";
    } else {
      clearInterval(gateProgressTimer);
      gateProgressTimer = null;
      if (gateTipTimer) { clearInterval(gateTipTimer); gateTipTimer = null; }

      const targetUrl = directDownloadUrls[currentGatePlatform] || (currentGatePlatform === "windows" ? "/download/windows" : "/download/linux");

      if (statusText) statusText.textContent = "Download liberado com sucesso!";
      if (statusSub) statusSub.textContent = "O download iniciou. Se não começar automaticamente, clique no botão abaixo.";
      if (stageLabel) stageLabel.textContent = "Transferência iniciada";
      if (secRemaining) secRemaining.textContent = "Pronto!";
      if (readyActions) readyActions.hidden = false;
      if (finalBtn) {
        finalBtn.href = targetUrl;
        finalBtn.onclick = () => { triggerBrowserDownload(targetUrl); };
      }

      triggerBrowserDownload(targetUrl);
    }
  }, stepMs);
}

function initMobileNav() {
  const toggle = document.getElementById("navMobileToggle");
  const drawer = document.getElementById("mobileNavDrawer");
  if (!toggle || !drawer) return;

  const openMenu = () => {
    toggle.classList.add("open");
    toggle.setAttribute("aria-expanded", "true");
    drawer.classList.add("open");
    drawer.setAttribute("aria-hidden", "false");
    document.body.style.overflow = "hidden";
  };

  const closeMenu = () => {
    toggle.classList.remove("open");
    toggle.setAttribute("aria-expanded", "false");
    drawer.classList.remove("open");
    drawer.setAttribute("aria-hidden", "true");
    document.body.style.overflow = "";
  };

  toggle.addEventListener("click", () => {
    if (drawer.classList.contains("open")) {
      closeMenu();
    } else {
      openMenu();
    }
  });

  drawer.querySelectorAll("a").forEach(link => {
    link.addEventListener("click", () => {
      closeMenu();
    });
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && drawer.classList.contains("open")) {
      closeMenu();
    }
  });

  window.addEventListener("resize", () => {
    if (window.innerWidth > 820 && drawer.classList.contains("open")) {
      closeMenu();
    }
  }, { passive: true });
}

function initBackToTop() {
  const btn = document.getElementById("btnBackToTop");
  if (!btn) return;

  window.addEventListener("scroll", () => {
    if (window.scrollY > 380) {
      btn.classList.add("show");
    } else {
      btn.classList.remove("show");
    }
  }, { passive: true });

  btn.addEventListener("click", () => {
    window.scrollTo({
      top: 0,
      behavior: "smooth"
    });
  });
}

function initAnimatedCounters() {
  const counterElements = document.querySelectorAll("[data-counter-target]");
  if (!counterElements.length) return;

  if (typeof window === "undefined" || !("IntersectionObserver" in window)) {
    return;
  }

  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const el = entry.target;
        observer.unobserve(el);
        animateCounter(el);
      }
    });
  }, { threshold: 0.25 });

  counterElements.forEach(el => observer.observe(el));

  function animateCounter(el) {
    const target = parseFloat(el.getAttribute("data-counter-target")) || 0;
    const prefix = el.getAttribute("data-counter-prefix") || "";
    const suffix = el.getAttribute("data-counter-suffix") || "";
    const useLocale = el.getAttribute("data-counter-locale") === "true";
    const duration = 1400;
    const startTime = performance.now();

    function update(now) {
      const elapsed = now - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const ease = progress === 1 ? 1 : 1 - Math.pow(2, -10 * progress);
      const current = Math.round(target * ease);

      const formatted = useLocale ? current.toLocaleString("pt-BR") : current.toString();
      el.textContent = `${prefix}${formatted}${suffix}`;

      if (progress < 1) {
        requestAnimationFrame(update);
      }
    }

    requestAnimationFrame(update);
  }
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

function initStabilityVisualizer() {
  const terminal = document.querySelector(".diagnostic-window");
  if (!terminal) return;
  const lines = terminal.querySelectorAll(".diagnostic-line");
  if (!lines.length) return;

  const observer = new IntersectionObserver((entries, obs) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        lines.forEach((line, idx) => {
          line.style.opacity = "0";
          line.style.transform = "translateX(-8px)";
          line.style.transition = `all 0.3s ease ${idx * 0.12}s`;
          setTimeout(() => {
            line.style.opacity = "1";
            line.style.transform = "translateX(0)";
          }, 50);
        });
        obs.disconnect();
      }
    });
  }, { threshold: 0.2 });

  observer.observe(terminal);
}

function initServerStatusChecker() {
  const input = document.getElementById("serverIpInput");
  const btn = document.getElementById("btnCheckServer");
  const resultBox = document.getElementById("serverResultBox");
  const quickPills = document.querySelectorAll(".quick-server-pill");

  if (!input || !btn || !resultBox) return;

  quickPills.forEach(pill => {
    pill.addEventListener("click", () => {
      quickPills.forEach(p => p.classList.remove("active"));
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

  function formatMinecraftMotd(raw) {
    if (!raw) return "";
    const mcColors = {
      "0": "#000000", "1": "#0000AA", "2": "#00AA00", "3": "#00AAAA",
      "4": "#AA0000", "5": "#AA00AA", "6": "#FFAA00", "7": "#AAAAAA",
      "8": "#555555", "9": "#5555FF", "a": "#55FF55", "b": "#55FFFF",
      "c": "#FF5555", "d": "#FF55FF", "e": "#FFFF55", "f": "#FFFFFF"
    };
    let clean = raw.replace(/\r\n/g, "<br>").replace(/\n/g, "<br>");
    clean = clean.replace(/§([0-9a-fA-F])/g, (m, c) => {
      const color = mcColors[c.toLowerCase()] || "#ffffff";
      return `</span><span style="color:${color};">`;
    });
    clean = clean.replace(/§l/gi, '<span style="font-weight:700;">');
    clean = clean.replace(/§o/gi, '<span style="font-style:italic;">');
    clean = clean.replace(/§n/gi, '<span style="text-decoration:underline;">');
    clean = clean.replace(/§m/gi, '<span style="text-decoration:line-through;">');
    clean = clean.replace(/§k/gi, '<span>');
    clean = clean.replace(/§r/gi, '</span><span style="color:#e2e8f0;font-weight:normal;font-style:normal;">');
    return `<span>${clean}</span>`;
  }

  let lastServerCheck = null;

  function renderServerResult(address, data, pingMs) {
    if (!resultBox) return;
    const noResponseText = (window.LuxI18n && window.LuxI18n.t("servers.noResponse")) || "Servidor não respondeu ao ping";
    const offlineText = (window.LuxI18n && window.LuxI18n.t("servers.offline")) || "Offline / Inacessível";
    const offlineDescText = (window.LuxI18n && window.LuxI18n.t("servers.offlineDesc")) || "O servidor está desligado, em manutenção ou com proteção contra pings diretos. Verifique se o endereço foi digitado corretamente.";
    const onlineText = (window.LuxI18n && window.LuxI18n.t("servers.online")) || "Servidor Online";
    const connectedPlayersText = (window.LuxI18n && window.LuxI18n.t("servers.connectedPlayers")) || "Jogadores Conectados";
    const latencyText = (window.LuxI18n && window.LuxI18n.t("servers.latency")) || "Latência Estimada";
    const versionText = (window.LuxI18n && window.LuxI18n.t("servers.version")) || "Versão do Servidor";
    const btnCopyIpText = (window.LuxI18n && window.LuxI18n.t("servers.btnCopyIp")) || "Copiar IP";
    const btnConnectText = (window.LuxI18n && window.LuxI18n.t("servers.btnConnect")) || "Conectar via Luxmc";

    if (!data || !data.online) {
      resultBox.innerHTML = `
        <div class="server-card-content">
          <div class="server-card-top">
            <div class="server-identity">
              <div class="server-favicon" style="display:flex;align-items:center;justify-content:center;color:#ef4444;font-size:1.4rem;">✕</div>
              <div class="server-name-wrap">
                <div class="server-hostname">${escapeHtml(address)}</div>
                <div class="server-ip-copy-tag">${escapeHtml(noResponseText)}</div>
              </div>
            </div>
            <div class="server-status-pill offline">
              <span class="pill-dot" style="background:#ef4444;"></span>
              <span>${escapeHtml(offlineText)}</span>
            </div>
          </div>
          <div class="server-motd-container" style="color:#94a3b8;">
            ${escapeHtml(offlineDescText)}
          </div>
        </div>
      `;
      return;
    }

    const onlinePlayers = data.players?.online ?? 0;
    const maxPlayers = data.players?.max ?? 0;
    const versionStr = data.version?.name_clean || data.version || "Compatível com Luxmc";
    const motdHtml = data.motd?.html || escapeHtml(data.motd?.clean || address);
    const iconSrc = data.icon || "assets/logo.png";

    resultBox.innerHTML = `
      <div class="server-card-content">
        <div class="server-card-top">
          <div class="server-identity">
            <img src="${iconSrc}" alt="Ícone de ${escapeHtml(address)}" class="server-favicon" onerror="this.src='assets/logo.png'">
            <div class="server-name-wrap">
              <div class="server-hostname">${escapeHtml(address)}</div>
              <div class="server-ip-copy-tag">${escapeHtml(address)}</div>
            </div>
          </div>
          <div class="server-status-pill online">
            <span class="pill-dot" style="background:#10b981;"></span>
            <span>${escapeHtml(onlineText)}</span>
          </div>
        </div>

        <div class="server-metrics-grid">
          <div class="server-metric-item">
            <span class="metric-label">${escapeHtml(connectedPlayersText)}</span>
            <span class="metric-val">
              ${formatNumber(onlinePlayers)} <span style="font-size:0.75rem;color:#94a3b8;font-weight:600;">/ ${formatNumber(maxPlayers)}</span>
            </span>
          </div>
          <div class="server-metric-item">
            <span class="metric-label">${escapeHtml(latencyText)}</span>
            <span class="metric-val" style="color:#34d399;">
              ${pingMs} <span style="font-size:0.75rem;color:#94a3b8;font-weight:600;">ms</span>
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
          <a href="#download" class="btn-server-play">
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
            setTimeout(() => { label.textContent = old; }, 2000);
          }
        });
      });
    }
  }

  window.addEventListener("luxmc-language-changed", () => {
    if (lastServerCheck) {
      renderServerResult(lastServerCheck.address, lastServerCheck.data, lastServerCheck.pingMs);
    }
  });

  async function checkServer(address) {
    if (!address) return;

    const checkingText = (window.LuxI18n && window.LuxI18n.t("servers.checking")) || "Consultando status de";
    resultBox.innerHTML = `
      <div style="text-align: center; padding: 24px; color: var(--text-muted);">
        <div style="font-size: 1.6rem; margin-bottom: 8px;">⏳</div>
        <div>${escapeHtml(checkingText)} <strong>${escapeHtml(address)}</strong>...</div>
      </div>
    `;

    const startTs = performance.now();
    let data = null;
    let pingMs = 0;

    // 1. Primary: eu.mc-api.net (bypasses Cloudflare on Hypixel and handles international & BR servers)
    try {
      const res = await fetch(`https://eu.mc-api.net/v3/server/ping/${encodeURIComponent(address)}`, {
        signal: AbortSignal.timeout(6000)
      });
      if (res.ok) {
        const d = await res.json();
        if (d && (d.online === true || d.status === true)) {
          data = {
            online: true,
            icon: d.favicon || d.favicon_base64 || null,
            players: d.players || { online: 0, max: 0 },
            version: { name_clean: d.version?.name || "1.20+" },
            motd: {
              html: d.description ? formatMinecraftMotd(d.description) : ""
            }
          };
          pingMs = typeof d.took === "number" ? Math.max(1, Math.round(d.took)) : Math.round(performance.now() - startTs);
        }
      }
    } catch {}

    // 2. Fallback: api.mcstatus.io
    if (!data || !data.online) {
      try {
        const res1 = await fetch(`https://api.mcstatus.io/v2/status/java/${encodeURIComponent(address)}`, {
          signal: AbortSignal.timeout(5000)
        });
        if (res1.ok) {
          const d1 = await res1.json();
          if (d1 && d1.online) {
            data = d1;
            pingMs = Math.round(performance.now() - startTs);
          }
        }
      } catch {}
    }

    // 3. Fallback: api.mcsrvstat.us
    if (!data || !data.online) {
      try {
        const res2 = await fetch(`https://api.mcsrvstat.us/3/${encodeURIComponent(address)}`, {
          signal: AbortSignal.timeout(5000)
        });
        if (res2.ok) {
          const d2 = await res2.json();
          if (d2 && d2.online) {
            data = {
              online: true,
              icon: d2.icon || null,
              players: d2.players || { online: 0, max: 0 },
              version: { name_clean: d2.version || "1.20+" },
              motd: {
                html: d2.motd?.html ? d2.motd.html.join("<br>") : (d2.motd?.clean ? formatMinecraftMotd(d2.motd.clean.join("\n")) : "")
              }
            };
            pingMs = Math.round(performance.now() - startTs);
          }
        }
      } catch {}
    }

    lastServerCheck = { address, data, pingMs };
    renderServerResult(address, data, pingMs);
  }

  checkServer("hypixel.net");
}

function initShowcaseModal() {
  const modal = document.getElementById("showcaseModal");
  const modalImg = document.getElementById("showcaseModalImg");
  const modalTitle = document.getElementById("showcaseModalTitle");
  const closeBtn = document.getElementById("showcaseModalClose");
  const backdrop = document.getElementById("showcaseModalBackdrop");
  const triggerBtn = document.getElementById("btnZoomShowcase");
  const frame = document.getElementById("showcaseWindowFrame");

  if (!modal || !modalImg) return;

  function openModal() {
    const currentImg = document.getElementById("showcaseImage");
    const activeTab = document.querySelector(".showcase-tabs .tab-btn.active");
    if (currentImg && currentImg.src) {
      modalImg.src = currentImg.src;
    }
    if (activeTab && modalTitle) {
      const titleKey = activeTab.dataset.titleKey;
      const title = (titleKey && window.LuxI18n ? window.LuxI18n.t(titleKey) : null) || activeTab.dataset.title || "Interface";
      modalTitle.textContent = `${title} — Luxmc Launcher`;
    }
    modal.classList.add("active");
    modal.setAttribute("aria-hidden", "false");
    document.body.style.overflow = "hidden";
  }

  function closeModal() {
    modal.classList.remove("active");
    modal.setAttribute("aria-hidden", "true");
    document.body.style.overflow = "";
  }

  if (triggerBtn) triggerBtn.addEventListener("click", (e) => { e.stopPropagation(); openModal(); });
  if (frame) frame.addEventListener("click", openModal);
  if (closeBtn) closeBtn.addEventListener("click", closeModal);
  if (backdrop) backdrop.addEventListener("click", closeModal);

  window.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && modal.classList.contains("active")) {
      closeModal();
    }
  });
}



