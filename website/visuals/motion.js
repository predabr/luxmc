const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
const toggle = document.getElementById("motionToggle");
const events = new AbortController();
let paused = false, scrollRequest = 0;
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
const revealObserver = new IntersectionObserver((entries) => {
  for (const entry of entries)
    if (entry.isIntersecting) {
      entry.target.classList.add("motion-entered");
      revealObserver.unobserve(entry.target);
    }
}, { threshold: 0.15 });
document.querySelectorAll(".section-heading,.diagnostic-copy,.skin-preview-copy,.social-copy").forEach((element) => revealObserver.observe(element));
const buttons = [...document.querySelectorAll("[data-product-screen]")], image = document.getElementById("productTourImage");
function selectScreen(index) {
  const selected = buttons[index];
  if (!selected || !image || selected.getAttribute("aria-pressed") === "true")
    return;
  image.src = selected.dataset.productScreen || image.src;
  image.alt = selected.dataset.productAlt || "";
  buttons.forEach((button) => button.setAttribute("aria-pressed", String(button === selected)));
  const label = document.getElementById("productScreenLabel");
  if (label)
    label.textContent = ["01 / INSTANCES", "02 / CONTENT", "03 / SKIN STUDIO"][index];
}
buttons.forEach((button, index) => button.addEventListener("click", () => {
  selectScreen(index);
}, { signal: events.signal }));
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
    ...diagnostic.querySelectorAll(".diagnostic-line")
  ].entries()) {
    line.style.opacity = "1";
    line.style.transform = "";
    if (reducedMotion.matches || paused)
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
  cancelAnimationFrame(scrollRequest);
  diagnosticTimers.forEach(clearTimeout);
  revealObserver.disconnect();
  diagnosticObserver.disconnect();
}, { signal: events.signal });
updateMotion();
scrollFrame();
