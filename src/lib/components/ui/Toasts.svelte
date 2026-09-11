<script lang="ts">
	import { CheckCircle2, AlertCircle, Info, X, Bell } from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	type Toast = {
		id: number;
		message: string;
		level: "info" | "success" | "warning" | "error";
	};

	let toasts = $state<Toast[]>([]);
	let counter = 0;
	let dismissing = $state<Set<number>>(new Set());

	export function push(message: string, level: Toast["level"] = "info") {
		const id = ++counter;
		toasts = [...toasts, { id, message, level }];
		setTimeout(() => {
			dismiss(id);
		}, 4000);
	}

	function color(level: Toast["level"]) {
		switch (level) {
			case "success":
				return { color: "rgb(74, 222, 128)", Icon: CheckCircle2 };
			case "error":
				return { color: "rgb(248, 113, 113)", Icon: AlertCircle };
			case "warning":
				return { color: "rgb(250, 204, 21)", Icon: AlertCircle };
			default:
				return { color: "rgb(94, 214, 198)", Icon: Info };
		}
	}

	function dismiss(id: number) {
		dismissing.add(id);
		dismissing = dismissing;
		setTimeout(() => {
			toasts = toasts.filter((t) => t.id !== id);
			dismissing.delete(id);
			dismissing = dismissing;
		}, 200);
	}
</script>

<div class="pointer-events-none fixed right-4 top-4 z-[200] flex w-80 flex-col gap-2">
	{#each toasts as toastItem (toastItem.id)}
		{@const cfg = color(toastItem.level)}
		<div
			class="pointer-events-auto flex items-start gap-2 rounded-lg p-3 text-xs shadow-lg"
			style="border: 1px solid {cfg.color}; background: rgb(var(--bg-elevated)); color: rgb(var(--fg)); animation: slideIn 200ms ease-out forwards, fadeOut 200ms ease-in 3.8s forwards;"
			role="status"
			aria-live="polite"
		>
			<cfg.Icon class="h-4 w-4 shrink-0" style="color: {cfg.color};" />
			<p class="flex-1">{toastItem.message}</p>
			<button
				type="button"
				class="grid h-5 w-5 place-items-center rounded transition-colors hover:bg-white/5"
				style="color: rgb(var(--fg-subtle));"
				onclick={() => dismiss(toastItem.id)}
				aria-label={t("common.close")}
			>
				<X class="h-3 w-3" />
			</button>
			<div class="absolute bottom-0 left-0 h-0.5 rounded-b-lg" style="background: {cfg.color}; animation: progress 4s linear forwards;"></div>
		</div>
	{/each}
</div>

<style>
	@keyframes slideIn {
		from {
			opacity: 0;
			transform: translateX(100%);
		}
		to {
			opacity: 1;
			transform: translateX(0);
		}
	}

	@keyframes fadeOut {
		from {
			opacity: 1;
		}
		to {
			opacity: 0;
		}
	}

	@keyframes progress {
		from {
			width: 100%;
		}
		to {
			width: 0%;
		}
	}
</style>
