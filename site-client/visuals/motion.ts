const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
const toggle = document.getElementById("motionToggle") as HTMLButtonElement;
const events = new AbortController();
let paused = false, scrollRequest = 0;
let tourAnimation: Animation | null = null;
let screenRequest = 0;
let pointerRequest = 0;
let pointerX = 0, pointerY = 0;
const capture = document.querySelector<HTMLElement>(".hero-capture");
const finePointer = matchMedia("(hover: hover) and (pointer: fine)");
const detailAnimations = new Set<Animation>();
const motionAllowed = () => !paused && !reducedMotion.matches && !document.hidden;
function clearPointer() {
    cancelAnimationFrame(pointerRequest);
    pointerRequest = 0;
    pointerX = 0; pointerY = 0;
    capture?.style.removeProperty("--capture-x");
    capture?.style.removeProperty("--capture-y");
}
const text = (key: any, fallback: any) => window.LuxI18n?.t(key) || fallback;
function updateMotion() {
    const stopped = paused || reducedMotion.matches || document.hidden;
    document.body.classList.toggle("motion-paused", stopped);
    toggle?.setAttribute("aria-pressed", String(paused || reducedMotion.matches));
    toggle?.setAttribute("aria-label", reducedMotion.matches
        ? text("world.reduced", "Movimento reduzido")
        : paused
            ? text("world.resume", "Retomar movimento")
            : text("world.pause", "Pausar movimento"));
    if (toggle) {
        toggle.textContent = stopped ? "▷" : "Ⅱ";
        toggle.disabled = reducedMotion.matches;
    }
    if (stopped) {
        clearPointer();
        detailAnimations.forEach(animation => animation.cancel());
        detailAnimations.clear();
        tourAnimation?.cancel();
        tourAnimation = null;
        diagnosticTimers.forEach(clearTimeout);
        diagnosticTimers = [];
        diagnostic?.querySelectorAll<HTMLElement>(".diagnostic-line").forEach((line: any) => {
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
    signal: events.signal,
});
document.addEventListener("visibilitychange", updateMotion, {
    signal: events.signal,
});
window.addEventListener("luxmc-language-changed", updateMotion, {
    signal: events.signal,
});
const revealObserver = new IntersectionObserver((entries: any) => {
    for (const entry of entries)
        if (entry.isIntersecting) {
            entry.target.classList.add("motion-entered");
            revealObserver.unobserve(entry.target);
        }
}, { threshold: 0.15 });
const revealTargets = ".hero-kicker,.hero-title,.hero-copy,.hero-capture,.section-heading,.diagnostic-copy,.skin-preview-copy,.social-copy,.download-card,.product-step,.feature-strip article,.identity-capture,.party-card,.performance-readout article,.faq-item,.project-card";
function observeReveals(root: ParentNode) {
    root.querySelectorAll<HTMLElement>(revealTargets).forEach((element, index) => {
        if (element.classList.contains("motion-item")) return;
        element.classList.add("motion-item");
        element.style.setProperty("--motion-delay", `${(index % 4) * 65}ms`);
        revealObserver.observe(element);
    });
}
observeReveals(document);
const catalog = document.getElementById("modsGrid");
const catalogObserver = new MutationObserver(() => { if (catalog) observeReveals(catalog); });
if (catalog) catalogObserver.observe(catalog, { childList: true, subtree: true });
capture?.addEventListener("pointermove", event => {
    if (!motionAllowed() || !finePointer.matches) return;
    const bounds = capture.getBoundingClientRect();
    pointerX = ((event.clientX - bounds.left) / bounds.width - .5) * 12;
    pointerY = ((event.clientY - bounds.top) / bounds.height - .5) * 10;
    if (pointerRequest) return;
    pointerRequest = requestAnimationFrame(() => {
        pointerRequest = 0;
        if (!motionAllowed()) return;
        capture.style.setProperty("--capture-x", `${pointerX}px`);
        capture.style.setProperty("--capture-y", `${pointerY}px`);
    });
}, { passive: true, signal: events.signal });
capture?.addEventListener("pointerleave", clearPointer, { signal: events.signal });
document.querySelectorAll<HTMLDetailsElement>(".faq-item").forEach(details => {
    details.addEventListener("toggle", () => {
        if (!details.open || !motionAllowed()) return;
        const content = details.querySelector("p");
        if (!content) return;
        const animation = content.animate([{ opacity: 0, translate: "0 -8px" }, { opacity: 1, translate: "0 0" }], { duration: 300, easing: "cubic-bezier(.16,1,.3,1)" });
        detailAnimations.add(animation);
        animation.onfinish = () => { animation.cancel(); detailAnimations.delete(animation); };
    }, { signal: events.signal });
});
const buttons = [...document.querySelectorAll<HTMLElement>("[data-product-screen]")], image = document.getElementById("productTourImage") as HTMLImageElement;
async function selectScreen(index: number) {
    const selected = buttons[index];
    if (!selected || !image) return;
    const request = ++screenRequest;
    if (selected.getAttribute("aria-pressed") === "true") return;
    const source = selected.dataset.productScreen || image.src;
    const preload = new Image();
    preload.src = source;
    try { await preload.decode(); } catch { return; }
    if (request !== screenRequest) return;
    tourAnimation?.cancel();
    image.src = source;
    image.alt = selected.dataset.productAlt || '';
    buttons.forEach((button: any) => button.setAttribute("aria-pressed", String(button === selected)));
    const label = document.getElementById("productScreenLabel") as HTMLElement;
    if (label)
        label.textContent = selected.dataset.productLabel || ["01 / INSTANCES", "02 / CONTENT", "03 / SKIN STUDIO"][index] || '';
    if (!paused && !reducedMotion.matches && !document.hidden) {
        tourAnimation = image.animate([
            { opacity: 0.15, transform: "translateY(18px) scale(.985)" },
            { opacity: 1, transform: "translateY(0) scale(1)" }
        ], { duration: 480, easing: "cubic-bezier(.16,1,.3,1)" });
    }
}
buttons.forEach((button: any, index: any) => button.addEventListener("click", () => {
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
    signal: events.signal,
});
window.addEventListener("resize", scrollFrame, {
    passive: true,
    signal: events.signal,
});
const diagnostic = document.getElementById("diagnosticBody") as HTMLElement;
let diagnosticTimers: ReturnType<typeof setTimeout>[] = [];
function revealDiagnostic() {
    diagnosticTimers.forEach(clearTimeout);
    diagnosticTimers = [];
    for (const [index, line] of [
        ...diagnostic.querySelectorAll<HTMLElement>(".diagnostic-line"),
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
const diagnosticObserver = new IntersectionObserver((entries: any) => {
    if (entries[0].isIntersecting) {
        revealDiagnostic();
        diagnosticObserver.disconnect();
    }
}, { threshold: 0.5 });
if (diagnostic)
    diagnosticObserver.observe(diagnostic);
(document
    .getElementById("replayDiagnostic") as HTMLButtonElement)?.addEventListener("click", revealDiagnostic, { signal: events.signal });
window.addEventListener("pagehide", (event: any) => {
    if (event.persisted)
        return;
    events.abort();
    ++screenRequest;
    tourAnimation?.cancel();
    cancelAnimationFrame(scrollRequest);
    diagnosticTimers.forEach(clearTimeout);
    revealObserver.disconnect();
    catalogObserver.disconnect();
    clearPointer();
    detailAnimations.forEach(animation => animation.cancel());
    detailAnimations.clear();
    diagnosticObserver.disconnect();
}, { signal: events.signal });
updateMotion();
scrollFrame();
