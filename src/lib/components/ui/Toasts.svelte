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

	export function push(message: string, level: Toast["level"] = "info") {
		const id = ++counter;
		toasts = [...toasts, { id, message, level }];
		setTimeout(() => {
			toasts = toasts.filter((t) => t.id !== id);
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
		toasts = toasts.filter((t) => t.id !== id);
	}
</script>

<div class="pointer-events-none fixed right-4 top-4 z-[200] flex w-80 flex-col gap-2">
	{#each toasts as toastItem (toastItem.id)}
		{@const cfg = color(toastItem.level)}
		<div
			class="pointer-events-auto flex items-start gap-2 rounded-lg p-3 text-xs shadow-lg"
			style="border: 1px solid {cfg.color}; background: rgb(var(--bg-elevated)); color: rgb(var(--fg));"
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
		</div>
	{/each}
</div>
