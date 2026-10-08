<script lang="ts">
    import { button as launcherButton } from "$lib/components/ui/button";
	import { onDestroy } from "svelte";
	import { CheckCircle2, AlertCircle, Info, X } from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	type Toast = {
		id: number;
		message: string;
		level: "info" | "success" | "warning" | "error";
		action?: { label: string; run: () => void };
	};

	let toasts = $state<Toast[]>([]);
	let counter = 0;
	let dismissing = $state<Set<number>>(new Set());
	const timers = new Map<number, ReturnType<typeof setTimeout>>();
	let destroyed = false;

	export function push(message: string, level: Toast["level"] = "info", action?: Toast["action"]) {
		if (destroyed || toasts.some(item => item.message === message && item.level === level && !dismissing.has(item.id))) return;
		if (toasts.length >= 20) {
            for (const item of toasts.slice(0, -15)) {
                clearTimeout(timers.get(item.id)); clearTimeout(timers.get(item.id + 0.5));
                timers.delete(item.id); timers.delete(item.id + 0.5); dismissing.delete(item.id);
            }
            toasts = toasts.slice(-15);
        }
		const id = ++counter;
		toasts = [...toasts, { id, message, level, action }];
		const t1 = setTimeout(() => {
			timers.delete(id);
			if (!destroyed) dismiss(id);
		}, level === "error" || action ? 10000 : 4500);
		timers.set(id, t1);
	}

	function color(level: Toast["level"]) {
		switch (level) {
			case "success":
				return { color: "rgb(var(--success))", Icon: CheckCircle2 };
			case "error":
				return { color: "rgb(var(--danger))", Icon: AlertCircle };
			case "warning":
				return { color: "rgb(var(--warning))", Icon: AlertCircle };
			default:
				return { color: "rgb(var(--brand-400))", Icon: Info };
		}
	}

	function dismiss(id: number) {
		if (dismissing.has(id)) return;
		clearTimeout(timers.get(id));
		timers.delete(id);
		dismissing.add(id);
		dismissing = dismissing;
		const t2 = setTimeout(() => {
			timers.delete(id + 0.5);
			if (destroyed) return;
			toasts = toasts.filter((t) => t.id !== id);
			dismissing.delete(id);
			dismissing = dismissing;
		}, 200);
		timers.set(id + 0.5, t2);
	}

	onDestroy(() => {
		destroyed = true;
		timers.forEach((t) => clearTimeout(t));
		timers.clear();
	});
</script>

<div class="pointer-events-none fixed right-4 top-4 z-[200] flex w-[min(380px,calc(100vw-32px))] flex-col gap-3">
    {#each toasts as item (item.id)}
        {@const cfg = color(item.level)}
        <div class="notification pointer-events-auto relative flex items-start gap-3 overflow-hidden rounded-2xl border border-border bg-bg-elevated p-4 text-fg shadow-elevated {dismissing.has(item.id) ? 'leaving' : ''}" role={item.level === 'error' ? 'alert' : 'status'}>
            <span class="absolute bottom-4 left-0 top-4 w-0.5 rounded-full" style:background={cfg.color}></span>
            <div class="grid h-9 w-9 shrink-0 place-items-center rounded-xl bg-bg-subtle" style:color={cfg.color}><cfg.Icon class="h-4 w-4" /></div>
            <div class="min-w-0 flex-1 py-1"><p class="break-words text-[13px] leading-relaxed">{item.message}</p>{#if item.action}<button type="button" class={launcherButton({variant:'ghostBrand',size:'sm',class:'mt-2 px-0'})} onclick={() => { item.action?.run(); dismiss(item.id); }}>{item.action.label}</button>{/if}</div>
            <button type="button" class="grid h-7 w-7 shrink-0 place-items-center rounded-lg text-fg-subtle transition-colors hover:bg-fg/5 hover:text-fg focus-visible:ring-2 focus-visible:ring-brand-400" onclick={() => dismiss(item.id)} aria-label={t('common.close')}><X class="h-4 w-4" /></button>
        </div>
    {/each}
</div>
<style>
    .notification { animation: arrive 180ms ease-out; }
    .notification.leaving { opacity: 0; transform: translateX(16px); transition: opacity 180ms, transform 180ms; }
    @keyframes arrive { from { opacity: 0; transform: translateY(-8px); } to { opacity: 1; transform: translateY(0); } }
    @media (prefers-reduced-motion: reduce) { .notification { animation: none; } .notification.leaving { transition: none; } }
</style>
