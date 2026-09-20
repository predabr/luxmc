
const GITHUB_REPO = "predabr/luxmc";
const MODRINTH_API = "https://api.modrinth.com/v2";

let currentCategory = "mod";
let searchDebounce = null;
let searchController = null;
let detectedOS = "linux";
const directDownloadUrls = {
  linux: "https://github.com/predabr/luxmc/releases/latest/download/Luxmc_1.7.5_amd64.AppImage",
  windows: "https://github.com/predabr/luxmc/releases/latest/download/Luxmc_1.7.5_x64-setup.exe",
  deb: "https://github.com/predabr/luxmc/releases/latest/download/Luxmc_1.7.5_amd64.deb"
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

document.addEventListener("DOMContentLoaded", () => {
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
});

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

function initOSDetection() {
  const ua = navigator.userAgent || "";
  const btn = document.getElementById("primaryDownloadBtn");
  const linuxCard = document.getElementById("card-linux");
  const windowsCard = document.getElementById("card-windows");

  const isWindows = ua.includes("Win");
  detectedOS = isWindows ? "windows" : "linux";

  if (isWindows) {
    if (windowsCard) {
      windowsCard.classList.add("is-detected");
      const badge = document.createElement("div");
      badge.className = "os-detected-badge";
      badge.innerHTML = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg> Seu Sistema Operacional Detectado (Windows)`;
      const header = windowsCard.querySelector(".download-card-header");
      if (header && !windowsCard.querySelector(".os-detected-badge")) {
        header.parentNode.insertBefore(badge, header);
      }
    }
    if (linuxCard) {
      linuxCard.classList.remove("featured");
    }
    if (btn) {
      btn.innerHTML = `<svg width="18" height="18" viewBox="0 0 88 88" fill="currentColor"><path d="M0 12.56L35.73 7.69V42.66H0V12.56ZM0 45.34H35.73V80.31L0 75.44V45.34ZM39.06 7.23L88 0V42.66H39.06V7.23ZM39.06 45.34H88V88L39.06 80.77V45.34Z"/></svg> Baixar para Windows (.exe)`;
      btn.href = "/download/windows";
    }
  } else {
    if (linuxCard) {
      linuxCard.classList.add("is-detected");
      const badge = document.createElement("div");
      badge.className = "os-detected-badge";
      badge.innerHTML = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg> Seu Sistema Operacional Detectado (Linux)`;
      const header = linuxCard.querySelector(".download-card-header");
      if (header && !linuxCard.querySelector(".os-detected-badge")) {
        header.parentNode.insertBefore(badge, header);
      }
    }
    if (btn) {
      btn.innerHTML = `<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139zm.529 3.405h.013c.213 0 .396.062.584.198.19.135.33.332.438.533.105.259.158.459.166.724 0-.02.006-.04.006-.06v.105a.086.086 0 01-.004-.021l-.004-.024a1.807 1.807 0 01-.15.706.953.953 0 01-.213.335.71.71 0 00-.088-.042c-.104-.045-.198-.064-.284-.133a1.312 1.312 0 00-.22-.066c.05-.06.146-.133.183-.198.053-.128.082-.264.088-.402v-.02a1.21 1.21 0 00-.061-.4c-.045-.134-.101-.2-.183-.333-.084-.066-.167-.132-.267-.132h-.016c-.093 0-.176.03-.262.132a.8.8 0 00-.205.334 1.18 1.18 0 00-.09.4v.019c.002.089.008.179.02.267-.193-.067-.438-.135-.607-.202a1.635 1.635 0 01-.018-.2v-.02a1.772 1.772 0 01.15-.768c.082-.22.232-.406.43-.533a.985.985 0 01.594-.2zm-2.962.059h.036c.142 0 .27.048.399.135.146.129.264.288.344.465.09.199.14.4.153.667v.004c.007.134.006.2-.002.266v.08c-.03.007-.056.018-.083.024-.152.055-.274.135-.393.2.012-.09.013-.18.003-.267v-.015c-.012-.133-.04-.2-.082-.333a.613.613 0 00-.166-.267.248.248 0 00-.183-.064h-.021c-.071.006-.13.04-.186.132a.552.552 0 00-.12.27.944.944 0 00-.023.33v.015c.012.135.037.2.08.334.046.134.098.2.166.268.01.009.02.018.034.024-.07.057-.117.07-.176.136a.304.304 0 01-.131.068 2.62 2.62 0 01-.275-.402 1.772 1.772 0 01-.155-.667 1.759 1.759 0 01.08-.668 1.43 1.43 0 01.283-.535c.128-.133.26-.2.418-.2zm1.37 1.706c.332 0 .733.065 1.216.399.293.2.523.269 1.052.468h.003c.255.136.405.266.478.399v-.131a.571.571 0 01.016.47c-.123.31-.516.643-1.063.842v.002c-.268.135-.501.333-.775.465-.276.135-.588.292-1.012.267a1.139 1.139 0 01-.448-.067 3.566 3.566 0 01-.322-.198c-.195-.135-.363-.332-.612-.465v-.005h-.005c-.4-.246-.616-.512-.686-.71-.07-.268-.005-.47.193-.6.224-.135.38-.271.483-.336.104-.074.143-.102.176-.131h.002v-.003c.169-.202.436-.47.839-.601.139-.036.294-.065.466-.065zm2.8 2.142c.358 1.417 1.196 3.475 1.735 4.473.286.534.855 1.659 1.102 3.024.156-.005.33.018.513.064.646-1.671-.546-3.467-1.089-3.966-.22-.2-.232-.335-.123-.335.59.534 1.365 1.572 1.646 2.757.13.535.16 1.104.021 1.67.067.028.135.06.205.067 1.032.534 1.413.938 1.23 1.537v-.043c-.06-.003-.12 0-.18 0h-.016c.151-.467-.182-.825-1.065-1.224-.915-.4-1.646-.336-1.77.465-.008.043-.013.066-.018.135-.068.023-.139.053-.209.064-.43.268-.662.669-.793 1.187-.13.533-.17 1.156-.205 1.869v.003c-.02.334-.17.838-.319 1.35-1.5 1.072-3.58 1.538-5.348.334a2.645 2.645 0 00-.402-.533 1.45 1.45 0 00-.275-.333c.182 0 .338-.03.465-.067a.615.615 0 00.314-.334c.108-.267 0-.697-.345-1.163-.345-.467-.931-.995-1.788-1.521-.63-.4-.986-.87-1.15-1.396-.165-.534-.143-1.085-.015-1.645.245-1.07.873-2.11 1.274-2.763.107-.065.037.135-.408.974-.396.751-1.14 2.497-.122 3.854a8.123 8.123 0 01.647-2.876c.564-1.278 1.743-3.504 1.836-5.268.048.036.217.135.289.202.218.133.38.333.59.465.21.201.477.335.876.335.039.003.075.006.11.006.412 0 .73-.134.997-.268.29-.134.52-.334.74-.4h.005c.467-.135.835-.402 1.044-.7zm2.185 8.958c.037.6.343 1.245.882 1.377.588.134 1.434-.333 1.791-.765l.211-.01c.315-.007.577.01.847.268l.003.003c.208.199.305.53.391.876.085.4.154.78.409 1.066.486.527.645.906.636 1.14l.003-.007v.018l-.003-.012c-.015.262-.185.396-.498.595-.63.401-1.746.712-2.457 1.57-.618.737-1.37 1.14-2.036 1.191-.664.053-1.237-.2-1.574-.898l-.005-.003c-.21-.4-.12-1.025.056-1.69.176-.668.428-1.344.463-1.897.037-.714.076-1.335.195-1.814.12-.465.308-.797.641-.984l.045-.022zm-10.814.049h.01c.053 0 .105.005.157.014.376.055.706.333 1.023.752l.91 1.664.003.003c.243.533.754 1.064 1.189 1.637.434.598.77 1.131.729 1.57v.006c-.057.744-.48 1.148-1.125 1.294-.645.135-1.52.002-2.395-.464-.968-.536-2.118-.469-2.857-.602-.369-.066-.61-.2-.723-.4-.11-.2-.113-.602.123-1.23v-.004l.002-.003c.117-.334.03-.752-.027-1.118-.055-.401-.083-.71.043-.94.16-.334.396-.4.69-.533.294-.135.64-.202.915-.47h.002v-.002c.256-.268.445-.601.668-.838.19-.201.38-.336.663-.336zm7.159-9.074c-.435.201-.945.535-1.488.535-.542 0-.97-.267-1.28-.466-.154-.134-.28-.268-.373-.335-.164-.134-.144-.333-.074-.333.109.016.129.134.199.2.096.066.215.2.36.333.292.2.68.467 1.167.467.485 0 1.053-.267 1.398-.466.195-.135.445-.334.648-.467.156-.136.149-.267.279-.267.128.016.034.134-.147.332a8.097 8.097 0 01-.69.468zm-1.082-1.583V5.64c-.006-.02.013-.042.029-.05.074-.043.18-.027.26.004.063 0 .16.067.15.135-.006.049-.085.066-.135.066-.055 0-.092-.043-.141-.068-.052-.018-.146-.008-.163-.065zm-.551 0c-.02.058-.113.049-.166.066-.047.025-.086.068-.14.068-.05 0-.13-.02-.136-.068-.01-.066.088-.133.15-.133.08-.031.184-.047.259-.005.019.009.036.03.03.05v.02h.003z"/></svg> Baixar para Linux (.AppImage)`;
      btn.href = "/download/linux";
    }
  }
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
        const btn = document.getElementById("primaryDownloadBtn");
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

    const exe = assetFor(data.assets, "windows");
    if (exe) {
      if (exe.browser_download_url) directDownloadUrls.windows = exe.browser_download_url;
      const link = document.getElementById("downloadExeLink");
      if (link) link.href = exe.browser_download_url;
      if (detectedOS === "windows") {
        const btn = document.getElementById("primaryDownloadBtn");
        if (btn) btn.href = exe.browser_download_url;
      }
      const meta = document.getElementById("exeSize");
      if (meta) {
        const sizeMb = (exe.size / (1024 * 1024)).toFixed(0);
        meta.innerText = `Instalador .exe · ${sizeMb} MB`;
      }
    }
    const heroAsset = detectedOS === "windows" ? exe : appImage;
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

  performModSearch("");
}

async function performModSearch(query) {
  searchController?.abort();
  const controller = new AbortController();
  searchController = controller;
  const container = document.getElementById("modsGrid");
  if (!container) return;

  container.innerHTML = `<div style="grid-column: 1/-1; text-align: center; padding: 40px; color: var(--text-muted);">
    <div style="font-size: 1.5rem; margin-bottom: 8px;">⏳</div>
    Buscando mods no Modrinth...
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
  const container = document.getElementById("modsGrid");
  if (!container) return;

  if (mods.length === 0) {
    container.innerHTML = `<div style="grid-column: 1/-1; text-align: center; padding: 40px; color: var(--text-muted);">
      Nenhum mod encontrado para essa pesquisa.
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
              <p class="project-author">por <span>${escapeHtml(author)}</span></p>
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
          ${["mod", "modpack"].includes(currentCategory) ? `<a class="btn-download-mod" data-luxmc href="luxmc://install/${currentCategory}?id=${encodeURIComponent(mod.project_id || slug)}&amp;source=modrinth">Instalar no Luxmc</a>` : ""}
          <a href="${downloadUrl}" target="_blank" rel="noopener noreferrer" class="btn-download-mod">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            Ver versões
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
  const btns = document.querySelectorAll(".mockup-nav-btn");
  const img = document.getElementById("mockupDisplayImg");
  const badge = document.getElementById("mockupTitleBadge");
  if (!btns.length || !img) return;

  let currentIndex = Math.max(0, Array.from(btns).findIndex(button => button.classList.contains("active")));
  let autoTimer = null;
  let userInteracted = false;

  function setMockup(index, manual = false) {
    if (manual) userInteracted = true;
    btns.forEach(b => { b.classList.remove("active"); b.setAttribute("aria-pressed", "false"); });
    const btn = btns[index];
    if (!btn) return;
    btn.classList.add("active");
    btn.setAttribute("aria-pressed", "true");

    const newSrc = btn.dataset.mockup;
    const title = btn.dataset.title || "Menu Principal";

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

  // Auto rotate every 6s if user hasn't clicked
  autoTimer = setInterval(() => {
    if (!userInteracted && !document.hidden && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      const next = (currentIndex + 1) % btns.length;
      setMockup(next);
    }
  }, 6000);

  const mockupContainer = document.querySelector(".window-mockup");
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

function showToast(msg) {
  let toast = document.getElementById("siteToast");
  if (!toast) {
    toast = document.createElement("div");
    toast.id = "siteToast";
    toast.className = "site-toast";
    document.body.appendChild(toast);
  }
  toast.textContent = `⚡ ${msg}`;
  toast.classList.add("show");
  clearTimeout(toast._timeout);
  toast._timeout = setTimeout(() => {
    toast.classList.remove("show");
  }, 3200);
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


