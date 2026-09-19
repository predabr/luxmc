<script lang="ts">
	import { Wifi, WifiOff } from "lucide-svelte";
	import { onMount } from "svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	let online = $state(true);
	let manuallyTriggered = $state(false);

	onMount(() => {
		online = typeof navigator === "undefined" ? true : navigator.onLine;
		const goOnline = () => {
			online = true;
			manuallyTriggered = false;
		};
		const goOffline = () => {
			online = false;
		};
		window.addEventListener("online", goOnline);
		window.addEventListener("offline", goOffline);
		return () => {
			window.removeEventListener("online", goOnline);
			window.removeEventListener("offline", goOffline);
		};
	});

	function dismiss() {
		manuallyTriggered = true;
	}
</script>

{#if !online && !manuallyTriggered}
	<div
		role="status"
		aria-live="polite"
		class="fixed bottom-4 right-4 z-50 flex items-center justify-between gap-3 rounded-2xl border px-4 py-2.5 text-xs shadow-2xl bg-bg-elevated/95 backdrop-blur-md"
		style="border-color: rgb(var(--warning) / 0.4); color: rgb(var(--warning));"
	>
		<span class="flex items-center gap-2">
			<WifiOff class="h-4 w-4" />
			<span>
				<strong>{t("statusBanner.offlineMode")}</strong> {t("statusBanner.offlineText")}
			</span>
		</span>
		<button
			type="button"
			class="rounded-lg px-2.5 py-1 text-[10px] font-bold uppercase tracking-wide transition-colors hover:bg-fg/10 border border-warning/30 cursor-pointer"
			onclick={dismiss}
		>
			{t("statusBanner.dismiss")}
		</button>
	</div>
{/if}
