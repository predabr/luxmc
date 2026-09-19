const activeDialogs: HTMLElement[] = [];

export function focusTrap(element: HTMLElement) {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    activeDialogs.push(element);
    const controls = () => Array.from(element.querySelectorAll<HTMLElement>('button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex="0"]')).filter(control => !control.hidden && control.getAttribute("aria-hidden") !== "true" && control.getClientRects().length > 0);
    const focus = () => (controls()[0] || element).focus({ preventScroll: true });
    queueMicrotask(() => { if (element.isConnected && activeDialogs.at(-1) === element) focus(); });
    const onKey = (event: KeyboardEvent) => {
        if (activeDialogs.at(-1) !== element || event.key !== "Tab") return;
        const items = controls();
        const first = items[0];
        const last = items.at(-1);
        if (!first || (event.shiftKey && (document.activeElement === first || document.activeElement === element)) || (!event.shiftKey && document.activeElement === last)) {
            event.preventDefault();
            (event.shiftKey ? last || element : first || element).focus();
        }
    };
    const onFocus = (event: FocusEvent) => {
        if (activeDialogs.at(-1) === element && event.target instanceof Node && !element.contains(event.target)) focus();
    };
    document.addEventListener("keydown", onKey, true);
    document.addEventListener("focusin", onFocus);
    return { destroy() {
        activeDialogs.splice(activeDialogs.indexOf(element), 1);
        document.removeEventListener("keydown", onKey, true);
        document.removeEventListener("focusin", onFocus);
        if (previous?.isConnected) previous.focus({ preventScroll: true });
    } };
}
