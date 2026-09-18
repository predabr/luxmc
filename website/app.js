// Luxmc Website - GitHub Releases Sync, Dynamic Modrinth Explorer & Fluid Animations

const GITHUB_REPO = "predabr/luxmc";
const MODRINTH_API = "https://api.modrinth.com/v2";

let currentCategory = "mod";
let searchDebounce = null;
let detectedOS = "linux";

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
});

// Interactive background particles responding to mouse movement
function initBackgroundParticles() {
  const canvas = document.getElementById("bgCanvas");
  if (!canvas) return;

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
        if (dist < 130) {
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

      for (let j = i + 1; j < particles.length; j++) {
        const p2 = particles[j];
        const dist = Math.hypot(p.x - p2.x, p.y - p2.y);
        if (dist < 95) {
          ctx.beginPath();
          ctx.moveTo(p.x, p.y);
          ctx.lineTo(p2.x, p2.y);
          ctx.strokeStyle = `rgba(255, 255, 255, ${0.1 * (1 - dist / 95)})`;
          ctx.lineWidth = 0.6;
          ctx.stroke();
        }
      }
    }

    requestAnimationFrame(frame);
  }

  requestAnimationFrame(frame);
}

// Cursor Spotlight effect for cards
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

// Detect user OS and adjust download buttons & highlights
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
      btn.href = "https://github.com/" + GITHUB_REPO + "/releases/latest";
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
      btn.innerHTML = `<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12.003 2c-2.26 0-4.093 1.833-4.093 4.094 0 1.25.56 2.37 1.442 3.123-.393.18-.74.453-1.01.8-1.07 1.378-1.07 3.327-.01 4.717.37.48.86.83 1.42 1.03-.49.52-.79 1.21-.79 1.97 0 1.62 1.32 2.94 2.94 2.94 1.63 0 2.95-1.32 2.95-2.94 0-.76-.3-1.45-.79-1.97.56-.2 1.05-.55 1.42-1.03 1.06-1.39 1.06-3.339-.01-4.717-.27-.347-.617-.62-1.01-.8.882-.753 1.442-1.873 1.442-3.123 0-2.261-1.834-4.094-4.094-4.094z"/></svg> Baixar para Linux (.AppImage)`;
      btn.href = `https://github.com/${GITHUB_REPO}/releases/latest/download/Luxmc-1.7.4.AppImage`;
    }
  }
}

// Live GitHub release fetch - uses Cloudflare Pages edge endpoint or fallback to GitHub
async function initGitHubRelease() {
  try {
    let data = null;
    try {
      const edgeRes = await fetch("/api/latest-release");
      if (edgeRes.ok) data = await edgeRes.json();
    } catch {}

    if (!data || !data.tag_name) {
      const ghRes = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`);
      if (ghRes.ok) data = await ghRes.json();
    }

    if (!data) return;
    
    const tag = data.tag_name || "v1.7.4";
    document.querySelectorAll(".live-version-tag").forEach(el => {
      el.textContent = tag;
    });

    const appImage = data.assets?.find(a => a.name.endsWith(".AppImage"));
    if (appImage) {
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
        codeSnippet.textContent = `chmod +x ${appImage.name} && ./${appImage.name}`;
      }
    }

    const deb = data.assets?.find(a => a.name.endsWith(".deb"));
    if (deb) {
      const link = document.getElementById("downloadDebLink");
      if (link) link.href = deb.browser_download_url;
      const meta = document.getElementById("debSize");
      if (meta) {
        const sizeMb = (deb.size / (1024 * 1024)).toFixed(0);
        meta.innerText = `Instalador .deb · ${sizeMb} MB`;
      }
    }

    const exe = data.assets?.find(a => a.name.endsWith(".exe"));
    if (exe) {
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
          <a href="${downloadUrl}" target="_blank" rel="noopener noreferrer" class="btn-download-mod">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            Baixar (.jar)
          </a>
        </div>
      </div>
    `;
  }).join("");

  initSpotlightCards();
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

// Interactive Window Mockup tabs & live crossfade
function initMockupTabs() {
  const btns = document.querySelectorAll(".mockup-nav-btn");
  const img = document.getElementById("mockupDisplayImg");
  const badge = document.getElementById("mockupTitleBadge");
  if (!btns.length || !img) return;

  let currentIndex = 0;
  let autoTimer = null;
  let userInteracted = false;

  function setMockup(index, manual = false) {
    if (manual) userInteracted = true;
    btns.forEach(b => b.classList.remove("active"));
    const btn = btns[index];
    if (!btn) return;
    btn.classList.add("active");

    const newSrc = btn.dataset.mockup;
    const title = btn.dataset.title || "Menu Principal";

    img.style.opacity = "0.2";
    img.style.transform = "scale(0.995)";
    setTimeout(() => {
      img.src = newSrc;
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
    if (!userInteracted) {
      const next = (currentIndex + 1) % btns.length;
      setMockup(next);
    }
  }, 6000);

  const mockupContainer = document.querySelector(".window-mockup");
  if (mockupContainer) {
    mockupContainer.addEventListener("mouseenter", () => clearInterval(autoTimer));
  }
}

// Interactive FAQ Accordion
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
    threshold: 0.08,
    rootMargin: "0px 0px -30px 0px"
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
