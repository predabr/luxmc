import { generateCapeDataUrl } from "./capes.js";

const $ = (id) => document.getElementById(id);
const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
const events = new AbortController();
let viewer,
  generation = 0,
  destroyed = false,
  currentSkinUrl = "assets/cinema/steve.png",
  currentTexture = currentSkinUrl,
  controller,
  renderQueue = Promise.resolve();
window.addEventListener("pagehide", (event) => {
  if (event.persisted) return;
  destroyed = true;
  generation++;
  controller?.abort();
  events.abort();
  viewer?.destroy();
});
const model = () =>
  document.querySelector("input[name=skinModel]:checked").value;
const status = (message, error = false) => {
  $("skinStatus").textContent = message;
  $("skinStatus").classList.toggle("error", error);
};
const reader = (file) =>
  new Promise((resolve, reject) => {
    const input = new FileReader();
    input.onload = () => resolve(input.result);
    input.onerror = reject;
    input.readAsDataURL(file);
  });
function updateModel() {
  const slim = model() === "slim";
  $("labelModelClassic").classList.toggle("selected", !slim);
  $("labelModelSlim").classList.toggle("selected", slim);
  $("currentModelBadge").textContent = slim ? "Slim / 3px" : "Classic / 4px";
  if (viewer)
    renderQueue = renderQueue
      .then(() => viewer.loadSkin(currentTexture, model()))
      .catch(() => status("Não foi possível alterar o modelo.", true));
}
async function applyTexture(texture, url, token) {
  renderQueue = renderQueue.then(async () => {
    if (token !== generation || destroyed) return;
    if (viewer) await viewer.loadSkin(texture, model());
    if (token !== generation) return;
    currentSkinUrl = url;
    currentTexture = texture;
    status(
      viewer
        ? "Skin carregada. Confira o resultado em 3D."
        : "Textura pronta. A prévia 3D não está disponível neste navegador.",
    );
  });
  return renderQueue;
}
async function loadByNick(nickname) {
  if (!/^[A-Za-z0-9_]{1,16}$/.test(nickname)) {
    $("playerNickInput").setAttribute("aria-invalid", "true");
    $("playerNickInput").focus();
    status("Use um nickname com até 16 letras, números ou sublinhados.", true);
    return;
  }
  $("playerNickInput").removeAttribute("aria-invalid");
  $("playerNickInput").value = nickname;
  $("nameMcProfileLink").href =
    `https://namemc.com/profile/${encodeURIComponent(nickname)}`;
  controller?.abort();
  controller = new AbortController();
  const token = ++generation,
    button = $("loadSkinButton");
  button.disabled = true;
  button.textContent = "Carregando…";
  status("Buscando a textura do personagem…");
  const local = /^(Steve|Alex)$/i.test(nickname);
  const url = local
    ? `assets/cinema/${nickname.toLowerCase()}.png`
    : `https://mineskin.eu/skin/${encodeURIComponent(nickname)}`;
  try {
    const response = await fetch(url, {
      signal: AbortSignal.any([controller.signal, AbortSignal.timeout(12000)]),
    });
    if (!response.ok) throw new Error("unavailable");
    const blob = await response.blob();
    const texture = await reader(blob);
    if (token !== generation) return;
    await applyTexture(
      texture,
      local ? new URL(url, location.href).href : url,
      token,
    );
  } catch {
    if (token === generation) {
      renderQueue = Promise.resolve();
      status(
        "Não foi possível carregar essa skin. Verifique o nickname ou importe um PNG.",
        true,
      );
    }
  } finally {
    if (token === generation) {
      button.disabled = false;
      button.textContent = "Carregar ↗";
    }
  }
}
async function importSkin(file) {
  if (!file) return;
  if (file.type !== "image/png" || file.size > 2097152) {
    status("Escolha um arquivo PNG de até 2 MB.", true);
    return;
  }
  const token = ++generation;
  controller?.abort();
  try {
    const bitmap = await createImageBitmap(file);
    const valid =
      bitmap.width === 64 && (bitmap.height === 32 || bitmap.height === 64);
    bitmap.close();
    if (!valid) {
      status("A textura precisa ter 64 × 64 ou 64 × 32 pixels.", true);
      return;
    }
    const texture = await reader(file);
    await applyTexture(texture, texture, token);
    $("skinDropzone").textContent = file.name;
  } catch {
    if (token === generation) {
      renderQueue = Promise.resolve();
      status("Esse PNG não pôde ser lido. Tente outro arquivo.", true);
    }
  } finally {
    if (token === generation) {
      $("loadSkinButton").disabled = false;
      $("loadSkinButton").textContent = "Carregar ↗";
    }
  }
}
$("skinSearchForm").addEventListener(
  "submit",
  (event) => {
    event.preventDefault();
    void loadByNick($("playerNickInput").value.trim());
  },
  { signal: events.signal },
);
document.querySelectorAll("[data-skin-nick]").forEach((button) =>
  button.addEventListener(
    "click",
    () => {
      document.querySelector(
        `input[name=skinModel][value=${button.dataset.skinModel}]`,
      ).checked = true;
      updateModel();
      void loadByNick(button.dataset.skinNick);
    },
    { signal: events.signal },
  ),
);
document
  .querySelectorAll("input[name=skinModel]")
  .forEach((input) =>
    input.addEventListener("change", updateModel, { signal: events.signal }),
  );
$("skinFileInput").addEventListener(
  "change",
  (event) => void importSkin(event.target.files[0]),
  { signal: events.signal },
);
const drop = $("skinDropzone");
for (const name of ["dragenter", "dragover"])
  drop.addEventListener(
    name,
    (event) => {
      event.preventDefault();
      drop.classList.add("drag-over");
    },
    { signal: events.signal },
  );
drop.addEventListener("dragleave", () => drop.classList.remove("drag-over"), {
  signal: events.signal,
});
drop.addEventListener(
  "drop",
  (event) => {
    event.preventDefault();
    drop.classList.remove("drag-over");
    void importSkin(event.dataTransfer.files[0]);
  },
  { signal: events.signal },
);
document.querySelectorAll("[data-animation]").forEach((button) =>
  button.addEventListener(
    "click",
    () => {
      viewer?.animate(button.dataset.animation);
      document
        .querySelectorAll("[data-animation]")
        .forEach((item) =>
          item.setAttribute("aria-pressed", String(item === button)),
        );
    },
    { signal: events.signal },
  ),
);
$("rotateSkinLeft").addEventListener("click", () => viewer?.rotate(0.35), {
  signal: events.signal,
});
$("rotateSkinRight").addEventListener("click", () => viewer?.rotate(-0.35), {
  signal: events.signal,
});
$("resetCamera").addEventListener("click", () => viewer?.reset(), {
  signal: events.signal,
});
$("capeSelection").addEventListener(
  "change",
  async () => {
    try {
      await viewer?.loadCape(generateCapeDataUrl($("capeSelection").value));
      status("Capa alterada na prévia.");
    } catch {
      status("Não foi possível carregar a capa.", true);
    }
  },
  { signal: events.signal },
);
$("applySkin").addEventListener(
  "click",
  () => {
    if (
      !currentSkinUrl.startsWith("https://") ||
      currentSkinUrl.includes("luxmc-r92.pages.dev")
    ) {
      status("Baixe este PNG e use Importar no Studio do launcher.");
      return;
    }
    window.openLuxmc(
      `luxmc://skin/apply?${new URLSearchParams({ url: currentSkinUrl, model: model() })}`,
    );
  },
  { signal: events.signal },
);
$("downloadSkin").addEventListener(
  "click",
  () => {
    const link = document.createElement("a");
    link.href = currentTexture;
    link.download = `luxmc-skin-${$("playerNickInput").value || "custom"}.png`;
    document.body.appendChild(link);
    link.click();
    link.remove();
    status("PNG preparado para download.");
  },
  { signal: events.signal },
);
try {
  const { createStudioViewer } = await import("./skinEngine.js");
  if (!destroyed) {
    viewer = createStudioViewer($("studioCanvasWrap"), reducedMotion);
    await viewer.loadSkin(currentTexture, model());
    await viewer.loadCape(generateCapeDataUrl($("capeSelection").value));
    $("studioLoading").hidden = true;
  }
} catch {
  $("studioLoading").querySelector("span").textContent =
    "WebGL indisponível. Você ainda pode buscar, importar e baixar a textura.";
  status("A prévia 3D não está disponível neste navegador.");
}
const params = new URLSearchParams(location.search),
  nick = params.get("nick");
if (params.has("model")) {
  document.querySelector(
    `input[name=skinModel][value=${["alex", "slim"].includes(params.get("model")) ? "slim" : "classic"}]`,
  ).checked = true;
  updateModel();
}
if (nick && /^[A-Za-z0-9_]{1,16}$/.test(nick)) void loadByNick(nick);
