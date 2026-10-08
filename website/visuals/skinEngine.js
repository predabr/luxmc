import { SkinViewer, IdleAnimation, WalkingAnimation, RunningAnimation, FlyingAnimation } from "skinview3d";
export function createStudioViewer(container, reducedMotion) {
  const canvas = container.querySelector("canvas");
  if (!canvas) throw new Error("Skin preview canvas unavailable");
  const viewer = new SkinViewer({
    canvas,
    width: container.clientWidth,
    height: container.clientHeight,
    pixelRatio: Math.min(devicePixelRatio, innerWidth < 768 ? 1 : 1.5),
    renderPaused: reducedMotion.matches
  });
  viewer.zoom = 0.8;
  viewer.camera.position.set(20, 12, 45);
  viewer.controls.update();
  viewer.fxaaPass.enabled = false;
  viewer.controls.enablePan = false;
  viewer.controls.enableZoom = true;
  viewer.controls.enableRotate = true;
  viewer.animation = new IdleAnimation();
  if (viewer.animation) viewer.animation.speed = 0.7;
  const events = new AbortController();
  let visible = true, lost = false;
  const renderInteraction = () => {
    if (viewer.renderPaused && !lost && !document.hidden)
      viewer.render();
  };
  viewer.controls.addEventListener("change", renderInteraction);
  function update() {
    viewer.renderPaused = document.hidden || !visible || reducedMotion.matches || lost;
    if (viewer.animation)
      viewer.animation.paused = reducedMotion.matches;
    if (!document.hidden && !lost)
      viewer.render();
  }
  const resize = new ResizeObserver(() => {
    viewer.setSize(container.clientWidth, container.clientHeight);
    if (!lost)
      viewer.render();
  });
  resize.observe(container);
  const intersection = new IntersectionObserver((entries) => {
    visible = entries[0].isIntersecting;
    update();
  }, { threshold: 0.01 });
  intersection.observe(container);
  document.addEventListener("visibilitychange", update, {
    signal: events.signal
  });
  reducedMotion.addEventListener("change", update, { signal: events.signal });
  canvas.addEventListener("webglcontextlost", (event) => {
    event.preventDefault();
    lost = true;
    update();
  }, { signal: events.signal });
  canvas.addEventListener("webglcontextrestored", () => {
    lost = false;
    update();
  }, { signal: events.signal });
  canvas.tabIndex = 0;
  const rotate = (amount) => {
    viewer.playerObject.rotation.y += amount;
    viewer.render();
  };
  canvas.addEventListener("keydown", (event) => {
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      event.preventDefault();
      rotate(event.key === "ArrowLeft" ? 0.3 : -0.3);
    }
    if (event.key === "+" || event.key === "-") {
      event.preventDefault();
      viewer.zoom = Math.max(0.4, Math.min(1.3, viewer.zoom + (event.key === "+" ? 0.1 : -0.1)));
      viewer.render();
    }
  }, { signal: events.signal });
  return {
    async loadSkin(url, model) {
      await viewer.loadSkin(url, {
        model: model === "slim" ? "slim" : "default"
      });
      viewer.render();
    },
    async loadCape(url) {
      await viewer.loadCape(url);
      viewer.render();
    },
    animate(name) {
      const animations = {
        idle: IdleAnimation,
        walk: WalkingAnimation,
        run: RunningAnimation,
        fly: FlyingAnimation
      };
      const Animation = animations[name] || IdleAnimation;
      viewer.animation = new Animation();
      if (viewer.animation) viewer.animation.speed = 0.7;
      update();
    },
    rotate,
    reset() {
      viewer.playerObject.rotation.y = 0;
      viewer.zoom = 0.8;
      viewer.camera.position.set(20, 12, 45);
      viewer.controls.update();
      viewer.render();
    },
    destroy() {
      viewer.controls.removeEventListener("change", renderInteraction);
      events.abort();
      resize.disconnect();
      intersection.disconnect();
      viewer.dispose();
    }
  };
}
