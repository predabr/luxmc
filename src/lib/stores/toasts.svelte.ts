import Toasts from "$lib/components/ui/Toasts.svelte";

let _instance: Toasts | null = null;

export function setToastInstance(t: Toasts) {
	_instance = t;
}

export function toast(message: string, level: "info" | "success" | "warning" | "error" = "info", action?: { label: string; run: () => void }) {
	_instance?.push(message, level, action);
}
