// Luxmc Website - Dynamic GitHub Releases, Modrinth Explorer & Scroll Animations

const GITHUB_REPO = "predabr/luxmc";
const MODRINTH_API = "https://api.modrinth.com/v2";

let currentCategory = "mod";
let searchDebounce = null;

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
  initOSDetection();
  initGitHubRelease();
  initModrinthExplorer();
  initShowcaseTabs();
  initScrollAnimations();
  initNavbarScroll();
  initKeyboardShortcuts();
});

// Detect user OS and adjust primary download button
function initOSDetection() {
  const ua = navigator.userAgent || "";
  const btn = document.getElementById("primaryDownloadBtn");
  const meta = document.getElementById("primaryDownloadMeta");

  let isWindows = ua.includes("Win");

  if (btn) {
    if (isWindows) {
      btn.innerHTML = `<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M0 3.449L9.75 2.1v9.451H0m10.949-9.602L24 0v11.4H10.949M0 12.6h9.75v9.451L0 20.699M10.949 12.6H24V24l-12.901-1.751"/></svg> Baixar para Windows (.exe)`;
      btn.href = "#download-windows";
      if (meta) meta.innerText = "Windows 10 / 11 64-bit · Instalador oficial";
    } else {
      btn.innerHTML = `<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12.003 2c-2.26 0-4.093 1.833-4.093 4.094 0 1.25.56 2.37 1.442 3.123-.393.18-.74.453-1.01.8-1.07 1.378-1.07 3.327-.01 4.717.37.48.86.83 1.42 1.03-.49.52-.79 1.21-.79 1.97 0 1.62 1.32 2.94 2.94 2.94 1.63 0 2.95-1.32 2.95-2.94 0-.76-.3-1.45-.79-1.97.56-.2 1.05-.55 1.42-1.03 1.06-1.39 1.06-3.339-.01-4.717-.27-.347-.617-.62-1.01-.8.882-.753 1.442-1.873 1.442-3.123 0-2.261-1.834-4.094-4.094-4.094z"/></svg> Baixar para Linux (.AppImage)`;
      btn.href = `https://github.com/${GITHUB_REPO}/releases/latest/download/Luxmc-1.7.3.AppImage`;
      if (meta) meta.innerText = "Universal Linux · Sem dependências extras";
    }
  }
}

// Live GitHub release fetch - syncs immediately when app updates on GitHub
async function initGitHubRelease() {
  try {
    const res = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`);
    if (!res.ok) return;
    const data = await res.json();
    
    const tag = data.tag_name || "v1.7.3";
    document.querySelectorAll(".live-version-tag").forEach(el => {
      el.textContent = tag;
    });

    const appImage = data.assets?.find(a => a.name.endsWith(".AppImage"));
    if (appImage) {
      const link = document.getElementById("downloadAppImageLink");
      if (link) {
        link.href = appImage.browser_download_url;
      }
      const meta = document.getElementById("appImageSize");
      if (meta) {
        const sizeMb = (appImage.size / (1024 * 1024)).toFixed(0);
        meta.innerText = `${sizeMb} MB · Portátil`;
      }
    }

    const deb = data.assets?.find(a => a.name.endsWith(".deb"));
    if (deb) {
      const link = document.getElementById("downloadDebLink");
      if (link) link.href = deb.browser_download_url;
    }

    const exe = data.assets?.find(a => a.name.endsWith(".exe"));
    if (exe) {
      const link = document.getElementById("downloadExeLink");
      if (link) link.href = exe.browser_download_url;
    }
  } catch (e) {
    console.debug("GitHub API fetch fallback:", e);
  }
}

// Modrinth API search and live download
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
  const container = document.getElementById("modsGrid");
  if (!container) return;

  container.innerHTML = `<div style="grid-column: 1/-1; text-align: center; padding: 40px; color: var(--text-muted);">
    <div style="font-size: 1.5rem; margin-bottom: 8px;">⏳</div>
    Buscando mods no Modrinth...
  </div>`;

  try {
    let facets = `[["project_type:${currentCategory}"]]`;
    const url = `${MODRINTH_API}/search?query=${encodeURIComponent(query)}&limit=6&facets=${encodeURIComponent(facets)}`;
    const res = await fetch(url);
    if (!res.ok) throw new Error("Modrinth API error");
    const data = await res.json();

    if (data.hits && data.hits.length > 0) {
      renderModCards(data.hits);
      return;
    }
  } catch (e) {
    console.debug("Using fallback mods data:", e);
  }

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
    const downloads = formatNumber(mod.downloads || 1500000);
    const categories = (mod.categories || []).slice(0, 3);
    const downloadUrl = `https://modrinth.com/mod/${encodeURIComponent(slug)}/versions`;

    return `
      <div class="project-card reveal active">
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
          <a href="${downloadUrl}" target="_blank" rel="noopener noreferrer" class="btn-download-mod">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            Baixar (.jar)
          </a>
        </div>
      </div>
    `;
  }).join("");
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

// Feature Showcase tab switcher with crossfade
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

// Scroll-triggered reveal animations
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
    threshold: 0.1,
    rootMargin: "0px 0px -40px 0px"
  });

  reveals.forEach(el => observer.observe(el));
}

// Navbar shadow on scroll
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

// Keyboard shortcuts
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

// Copy Code to Clipboard
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

// Toast notification
function showToast(msg) {
  let toast = document.getElementById("siteToast");
  if (!toast) {
    toast = document.createElement("div");
    toast.id = "siteToast";
    toast.className = "site-toast";
    document.body.appendChild(toast);
  }
  toast.innerHTML = `<span style="color: #ffffff">⚡</span> ${msg}`;
  toast.classList.add("show");
  clearTimeout(toast._timeout);
  toast._timeout = setTimeout(() => {
    toast.classList.remove("show");
  }, 3200);
}
