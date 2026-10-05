import Toasts from "$lib/components/ui/Toasts.svelte";

type ToastLevel = "info" | "success" | "warning" | "error";
type ToastAction = { label: string; run: () => void };

let _instance: Toasts | null = null;
const _pending: Array<{ message: string; level: ToastLevel; action?: ToastAction }> = [];

export function setToastInstance(t: Toasts | null) {
	_instance = t;
	if (!_instance || _pending.length === 0) return;
	const queued = _pending.splice(0, _pending.length);
	for (const item of queued) {
		_instance.push(item.message, item.level, item.action);
	}
}

export function toast(message: string, level: ToastLevel = "info", action?: ToastAction) {
	if (_instance) {
		_instance.push(message, level, action);
		return;
	}
	_pending.push({ message, level, action });
	if (_pending.length > 20) _pending.shift();
}
