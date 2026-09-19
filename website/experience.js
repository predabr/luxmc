const reduced = matchMedia("(prefers-reduced-motion: reduce)");
const progress = document.querySelector(".reading-progress");
let ticking = false;
function scrollFrame() {
  if (ticking) return;
  ticking = true;
  requestAnimationFrame(() => {
    const distance = document.documentElement.scrollHeight - innerHeight;
    if (progress) progress.style.transform = `scaleX(${distance > 0 ? scrollY / distance : 0})`;
    ticking = false;
  });
}
addEventListener("scroll", scrollFrame, { passive: true });
scrollFrame();
document.addEventListener("visibilitychange", () => document.body.classList.toggle("motion-paused", document.hidden));
const children = document.querySelectorAll(".bento-card, .download-card, .journey article, .section-header");
if (!reduced.matches && "IntersectionObserver" in window) {
  const observer = new IntersectionObserver(entries => {
    for (const entry of entries) if (entry.isIntersecting) { entry.target.classList.add("entered"); observer.unobserve(entry.target); }
  }, { threshold: .12 });
  children.forEach((element, index) => { element.classList.add("scroll-child"); element.style.setProperty("--stagger", `${index % 3 * 80}ms`); observer.observe(element); });
}
for (const element of document.querySelectorAll("[data-tilt]")) {
  let frame = 0;
  element.addEventListener("pointermove", event => {
    if (reduced.matches || event.pointerType !== "mouse" || innerWidth < 1050) return;
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      const box = element.getBoundingClientRect();
      const x = (event.clientX - box.left) / box.width - .5;
      const y = (event.clientY - box.top) / box.height - .5;
      element.style.transform = `perspective(1500px) rotateX(${-y * 5}deg) rotateY(${x * 5}deg)`;
    });
  });
  element.addEventListener("pointerleave", () => { cancelAnimationFrame(frame); element.style.transform = ""; });
}
fetch("/api/account/me", { method: "POST", headers: { "Content-Type": "application/json" }, body: "{}" }).then(async response => {
  if (!response.ok) return;
  const { account } = await response.json();
  if (typeof account?.username !== "string") return;
  for (const link of document.querySelectorAll("[data-account-link]")) link.textContent = account.username;
}).catch(() => {});
