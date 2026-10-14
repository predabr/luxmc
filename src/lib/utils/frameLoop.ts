export function frameLoop(render: (now: number) => void, fps: number): () => void {
    let frame = 0;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let stopped = false;
    let nextRender: number | undefined;
    const interval = 1000 / Math.max(1, Math.min(60, fps));
    const tick = (now: number) => {
        if (stopped) return;
        if (nextRender === undefined || now >= nextRender - .1) {
            render(now);
            if (stopped) return;
            nextRender = (nextRender ?? now) + interval;
            if (nextRender <= now) nextRender = now + interval;
        }
        const delay = Math.max(0, nextRender - performance.now() - 4);
        if (delay === 0) frame = requestAnimationFrame(tick);
        else timer = setTimeout(() => { if (!stopped) frame = requestAnimationFrame(tick); }, delay);
    };
    frame = requestAnimationFrame(tick);
    return () => { stopped = true; cancelAnimationFrame(frame); clearTimeout(timer); };
}
