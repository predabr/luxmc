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
		class="flex items-center justify-between gap-2 rounded-lg border px-3 py-2 text-xs"
		style="border-color: rgba(250, 204, 21, 0.3); background: rgba(250, 204, 21, 0.08); color: rgb(252, 211, 77);"
	>
		<span class="flex items-center gap-2">
			<WifiOff class="h-3.5 w-3.5" />
			<span>
				<strong>{t("statusBanner.offlineMode")}</strong> {t("statusBanner.offlineText")}
			</span>
		</span>
		<button
			type="button"
			class="rounded px-2 py-0.5 text-[10px] uppercase tracking-wide transition-colors hover:bg-white/5"
			style="border: 1px solid rgba(250, 204, 21, 0.3);"
			onclick={dismiss}
		>
			{t("statusBanner.dismiss")}
		</button>
	</div>
{:else if online}
	<div
		class="flex items-center gap-2 text-[10px]"
		style="color: rgb(var(--fg-subtle));"
	>
		<Wifi class="h-3 w-3" />
		<span>{t("statusBanner.online")}</span>
	</div>
{/if}
