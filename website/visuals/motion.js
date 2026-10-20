const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
const toggle = document.getElementById("motionToggle");
const events = new AbortController();
let paused = false, scrollRequest = 0;
try {
  paused = localStorage.getItem("luxmc:site-motion") === "paused";
} catch {
}
let tourAnimation = null;
let screenRequest = 0;
let pointerRequest = 0;
let pointerX = 0, pointerY = 0;
const capture = document.querySelector(".hero-capture");
const finePointer = matchMedia("(min-width: 901px) and (hover: hover) and (pointer: fine)");
const detailAnimations = /* @__PURE__ */ new Set();
const motionAllowed = () => !paused && !reducedMotion.matches && !document.hidden;
const buttonAnimations = /* @__PURE__ */ new WeakMap();
const surfaceSelector = ".hero-capture,.plans-capture,.product-step,.download-card,.project-card,.plan-card,.feature-strip article,.party-card,.performance-readout article";
let activeSurface = null;
let surfaceRequest = 0;
let surfaceX = 0, surfaceY = 0;
let preloadTimer;
const visibleObserver = new IntersectionObserver((entries) => {
  for (const entry of entries) entry.target.classList.toggle("motion-visible", entry.isIntersecting);
}, { rootMargin: "40px" });
document.addEventListener("click", (event) => {
  if (!motionAllowed()) return;
  const button = event.target?.closest(".btn-primary,.btn-secondary,.btn-primary-nav,.btn-download-main,.btn-download-mod,.commerce-button,.plans-explore,.search-pill,.install-tab,.product-step button,.nav-link");
  if (!button || button.matches(':disabled,[aria-disabled="true"]')) return;
  const previous = buttonAnimations.get(button);
  if (previous) {
    previous.cancel();
    detailAnimations.delete(previous);
  }
  const animation = button.animate([{ scale: "1" }, { scale: ".97", offset: 0.35 }, { scale: "1" }], { duration: 260, easing: "cubic-bezier(.16,1,.3,1)" });
  buttonAnimations.set(button, animation);
  detailAnimations.add(animation);
  animation.onfinish = () => {
    animation.cancel();
    detailAnimations.delete(animation);
    buttonAnimations.delete(button);
  };
}, { signal: events.signal });
function clearSurface() {
  cancelAnimationFrame(surfaceRequest);
  surfaceRequest = 0;
  activeSurface?.classList.remove("motion-pointed");
  activeSurface?.style.removeProperty("--surface-x");
  activeSurface?.style.removeProperty("--surface-y");
  activeSurface = null;
}
function clearPointer() {
  cancelAnimationFrame(pointerRequest);
  pointerRequest = 0;
  pointerX = 0;
  pointerY = 0;
  capture?.style.removeProperty("--capture-x");
  capture?.style.removeProperty("--capture-y");
}
const text = (key, fallback) => window.LuxI18n?.t(key) || fallback;
function updateMotion() {
  const stopped = paused || reducedMotion.matches || document.hidden;
  document.body.classList.toggle("motion-paused", stopped);
  toggle?.setAttribute("aria-pressed", String(paused || reducedMotion.matches));
  toggle?.setAttribute("aria-label", reducedMotion.matches ? text("world.reduced", "Movimento reduzido") : paused ? text("world.resume", "Retomar movimento") : text("world.pause", "Pausar movimento"));
  if (toggle) {
    toggle.textContent = stopped ? "▷" : "Ⅱ";
    toggle.disabled = reducedMotion.matches;
  }
  if (stopped) {
    clearPointer();
    clearSurface();
    detailAnimations.forEach((animation) => animation.cancel());
    detailAnimations.clear();
    tourAnimation?.cancel();
    tourAnimation = null;
    diagnosticTimers.forEach(clearTimeout);
    diagnosticTimers = [];
    diagnostic?.querySelectorAll(".diagnostic-line").forEach((line) => {
      line.style.opacity = "1";
      line.style.transform = "";
    });
  }
}
toggle?.addEventListener("click", () => {
  paused = !paused;
  try {
    localStorage.setItem("luxmc:site-motion", paused ? "paused" : "playing");
  } catch {
  }
  updateMotion();
}, { signal: events.signal });
reducedMotion.addEventListener("change", updateMotion, {
  signal: events.signal
});
document.addEventListener("visibilitychange", updateMotion, {
  signal: events.signal
});
window.addEventListener("luxmc-language-changed", updateMotion, {
  signal: events.signal
});
finePointer.addEventListener("change", () => {
  clearPointer();
  clearSurface();
}, { signal: events.signal });
const revealObserver = new IntersectionObserver((entries) => {
  for (const entry of entries)
    if (entry.isIntersecting) {
      entry.target.classList.add("motion-entered");
      revealObserver.unobserve(entry.target);
    }
}, { threshold: 0.15 });
const revealTargets = ".hero-kicker,.hero-title,.hero-copy,.hero-capture,.section-heading,.diagnostic-copy,.social-copy,.download-card,.product-step,.feature-strip article,.party-card,.performance-readout article,.faq-item,.project-card,.plan-card,.plans-intro,.plans-capture,.plans-kicker,.hero-facts > span,.membership-summary,.social-card,.commerce-panel";
function observeReveals(root) {
  root.querySelectorAll(revealTargets).forEach((element, index) => {
    if (element.classList.contains("motion-item")) return;
    element.classList.add("motion-item");
    element.style.setProperty("--motion-delay", `${index % 4 * 65}ms`);
    revealObserver.observe(element);
  });
  root.querySelectorAll(surfaceSelector).forEach((element) => {
    if (element.classList.contains("motion-surface")) return;
    element.classList.add("motion-surface");
  });
}
observeReveals(document);
document.querySelectorAll(".hero-section,.plans-hero").forEach((element) => visibleObserver.observe(element));
const catalog = document.getElementById("modsGrid");
const catalogObserver = new MutationObserver((records) => {
  for (const record of records) for (const node of record.removedNodes) {
    if (!(node instanceof HTMLElement)) continue;
    revealObserver.unobserve(node);
    node.querySelectorAll(".motion-item").forEach((element) => revealObserver.unobserve(element));
  }
  if (activeSurface && !activeSurface.isConnected) clearSurface();
  if (catalog) observeReveals(catalog);
});
if (catalog) catalogObserver.observe(catalog, { childList: true, subtree: true });
document.addEventListener("pointermove", (event) => {
  if (!motionAllowed() || !finePointer.matches || event.pointerType === "touch") return;
  const surface = event.target?.closest(".motion-surface");
  if (surface !== activeSurface) {
    clearSurface();
    activeSurface = surface || null;
    activeSurface?.classList.add("motion-pointed");
  }
  if (!activeSurface) return;
  surfaceX = event.clientX;
  surfaceY = event.clientY;
  if (surfaceRequest) return;
  surfaceRequest = requestAnimationFrame(() => {
    surfaceRequest = 0;
    if (!activeSurface || !motionAllowed()) return;
    const bounds = activeSurface.getBoundingClientRect();
    activeSurface.style.setProperty("--surface-x", `${Math.round(surfaceX - bounds.left)}px`);
    activeSurface.style.setProperty("--surface-y", `${Math.round(surfaceY - bounds.top)}px`);
  });
}, { passive: true, signal: events.signal });
document.addEventListener("pointerleave", clearSurface, { signal: events.signal });
capture?.addEventListener("pointermove", (event) => {
  if (!motionAllowed() || !finePointer.matches) return;
  const bounds = capture.getBoundingClientRect();
  pointerX = ((event.clientX - bounds.left) / bounds.width - 0.5) * 12;
  pointerY = ((event.clientY - bounds.top) / bounds.height - 0.5) * 10;
  if (pointerRequest) return;
  pointerRequest = requestAnimationFrame(() => {
    pointerRequest = 0;
    if (!motionAllowed()) return;
    capture.style.setProperty("--capture-x", `${pointerX}px`);
    capture.style.setProperty("--capture-y", `${pointerY}px`);
  });
}, { passive: true, signal: events.signal });
capture?.addEventListener("pointerleave", clearPointer, { signal: events.signal });
document.querySelectorAll(".faq-item,.plans-page .faq details").forEach((details) => {
  details.addEventListener("toggle", () => {
    if (!details.open || !motionAllowed()) return;
    const content = details.querySelector("p");
    if (!content) return;
    const animation = content.animate([{ opacity: 0, translate: "0 -8px" }, { opacity: 1, translate: "0 0" }], { duration: 300, easing: "cubic-bezier(.16,1,.3,1)" });
    detailAnimations.add(animation);
    animation.onfinish = () => {
      animation.cancel();
      detailAnimations.delete(animation);
    };
  }, { signal: events.signal });
});
const buttons = [...document.querySelectorAll("[data-product-screen]")], image = document.getElementById("productTourImage");
const decodedScreens = /* @__PURE__ */ new Map();
function decodeScreen(source) {
  const previous = decodedScreens.get(source);
  if (previous) return previous;
  const preload = new Image();
  preload.src = source;
  const promise = preload.decode().catch((error) => {
    decodedScreens.delete(source);
    throw error;
  });
  decodedScreens.set(source, promise);
  return promise;
}
async function selectScreen(index) {
  const selected = buttons[index];
  if (!selected || !image) return;
  const request = ++screenRequest;
  if (selected.getAttribute("aria-pressed") === "true") return;
  const source = selected.dataset.productScreen || image.src;
  try {
    await decodeScreen(source);
  } catch {
    return;
  }
  if (request !== screenRequest) return;
  tourAnimation?.cancel();
  image.src = source;
  image.alt = selected.dataset.productAlt || "";
  buttons.forEach((button) => button.setAttribute("aria-pressed", String(button === selected)));
  const label = document.getElementById("productScreenLabel");
  if (label)
    label.textContent = selected.dataset.productLabel || ["01 / INSTANCES", "02 / CONTENT", "03 / FRIENDS"][index] || "";
  if (motionAllowed()) {
    tourAnimation = image.animate([
      { opacity: 0.15, transform: "translateY(18px) scale(.985)" },
      { opacity: 1, transform: "translateY(0) scale(1)" }
    ], { duration: 480, easing: "cubic-bezier(.16,1,.3,1)" });
    const animation = tourAnimation;
    animation.onfinish = () => {
      animation.cancel();
      if (tourAnimation === animation) tourAnimation = null;
    };
  }
}
buttons.forEach((button, index) => button.addEventListener("click", () => {
  selectScreen(index);
}, { signal: events.signal }));
buttons.forEach((button, index) => button.addEventListener("keydown", (event) => {
  const next = event.key === "ArrowRight" || event.key === "ArrowDown" ? (index + 1) % buttons.length : event.key === "ArrowLeft" || event.key === "ArrowUp" ? (index - 1 + buttons.length) % buttons.length : event.key === "Home" ? 0 : event.key === "End" ? buttons.length - 1 : -1;
  if (next < 0) return;
  event.preventDefault();
  buttons[next].focus({ preventScroll: true });
  void selectScreen(next);
}, { signal: events.signal }));
const connection = navigator.connection;
if (buttons.length && !connection?.saveData && !/^(slow-)?2g$/.test(connection?.effectiveType || "")) {
  preloadTimer = setTimeout(() => {
    if (document.hidden) return;
    void (async () => {
      for (const button of buttons) {
        if (events.signal.aborted || document.hidden) return;
        if (button.dataset.productScreen) try {
          await decodeScreen(button.dataset.productScreen);
        } catch {
        }
      }
    })();
  }, 1200);
}
function scrollFrame() {
  if (scrollRequest)
    return;
  scrollRequest = requestAnimationFrame(() => {
    scrollRequest = 0;
    const distance = document.documentElement.scrollHeight - innerHeight;
    document.body.style.setProperty("--reading-progress", String(distance > 0 ? scrollY / distance : 0));
  });
}
window.addEventListener("scroll", scrollFrame, {
  passive: true,
  signal: events.signal
});
window.addEventListener("scroll", clearSurface, { passive: true, signal: events.signal });
window.addEventListener("resize", scrollFrame, {
  passive: true,
  signal: events.signal
});
const diagnostic = document.getElementById("diagnosticBody");
let diagnosticTimers = [];
function revealDiagnostic() {
  diagnosticTimers.forEach(clearTimeout);
  diagnosticTimers = [];
  for (const [index, line] of [
    ...diagnostic?.querySelectorAll(".diagnostic-line") || []
  ].entries()) {
    line.style.opacity = "1";
    line.style.transform = "";
    if (!motionAllowed())
      continue;
    line.style.opacity = "0";
    line.style.transform = "translateX(-5px)";
    diagnosticTimers.push(setTimeout(() => {
      line.style.opacity = "1";
      line.style.transform = "";
    }, 150 + index * 260));
  }
}
const diagnosticObserver = new IntersectionObserver((entries) => {
  if (entries[0].isIntersecting) {
    revealDiagnostic();
    diagnosticObserver.disconnect();
  }
}, { threshold: 0.5 });
if (diagnostic)
  diagnosticObserver.observe(diagnostic);
document.getElementById("replayDiagnostic")?.addEventListener("click", revealDiagnostic, { signal: events.signal });
window.addEventListener("pagehide", (event) => {
  if (event.persisted)
    return;
  events.abort();
  ++screenRequest;
  tourAnimation?.cancel();
  cancelAnimationFrame(scrollRequest);
  clearTimeout(preloadTimer);
  diagnosticTimers.forEach(clearTimeout);
  revealObserver.disconnect();
  catalogObserver.disconnect();
  visibleObserver.disconnect();
  clearPointer();
  clearSurface();
  detailAnimations.forEach((animation) => animation.cancel());
  detailAnimations.clear();
  diagnosticObserver.disconnect();
}, { signal: events.signal });
updateMotion();
scrollFrame();
