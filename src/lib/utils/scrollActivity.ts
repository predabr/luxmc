import { appState } from "$lib/stores/app.svelte";

export function scrollActivity(node: HTMLElement) {
    let timer: ReturnType<typeof setTimeout> | undefined;
    const finish = () => {
        appState.isScrolling = false;
        node.classList.remove("is-scrolling");
        document.documentElement.classList.remove("scroll-active");
    };
    const handleScroll = () => {
        if (!appState.isScrolling) {
            appState.isScrolling = true;
            node.classList.add("is-scrolling");
            document.documentElement.classList.add("scroll-active");
        }
        clearTimeout(timer);
        timer = setTimeout(finish, 160);
    };
    node.addEventListener("scroll", handleScroll, { capture: true, passive: true });
    node.addEventListener("wheel", handleScroll, { passive: true });
    node.addEventListener("touchmove", handleScroll, { passive: true });
    return {
        destroy() {
            node.removeEventListener("scroll", handleScroll, true);
            node.removeEventListener("wheel", handleScroll);
            node.removeEventListener("touchmove", handleScroll);
            clearTimeout(timer);
            finish();
        }
    };
}
