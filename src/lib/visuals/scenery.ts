export function scenery(node: HTMLElement, enabled = true) {
    const media = matchMedia("(prefers-reduced-motion: reduce)");
    let visible = false;
    let frame = 0;
    const reset = () => {
        node.style.setProperty("--scene-x", "0");
        node.style.setProperty("--scene-y", "0");
    };
    const sync = () => {
        const active = enabled && visible && !media.matches && !document.hidden;
        node.dataset.sceneActive = String(active);
        if (!active) { cancelAnimationFrame(frame); frame = 0; reset(); }
    };
    const observer = new IntersectionObserver(entries => {
        visible = entries[0].isIntersecting;
        sync();
    }, { threshold: .05 });
    observer.observe(node);
    const move = (event: PointerEvent) => {
        if (node.dataset.sceneActive !== "true" || event.pointerType !== "mouse") return;
        cancelAnimationFrame(frame);
        frame = requestAnimationFrame(() => {
            frame = 0;
            const rect = node.getBoundingClientRect();
            node.style.setProperty("--scene-x", String((event.clientX - rect.left) / rect.width - .5));
            node.style.setProperty("--scene-y", String((event.clientY - rect.top) / rect.height - .5));
        });
    };
    node.addEventListener("pointermove", move);
    node.addEventListener("pointerleave", reset);
    media.addEventListener("change", sync);
    document.addEventListener("visibilitychange", sync);
    sync();
    return {
        update(value: boolean) { enabled = value; sync(); },
        destroy() {
            cancelAnimationFrame(frame);
            observer.disconnect();
            node.removeEventListener("pointermove", move);
            node.removeEventListener("pointerleave", reset);
            media.removeEventListener("change", sync);
            document.removeEventListener("visibilitychange", sync);
        }
    };
}
