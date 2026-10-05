<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { DownloadCloud, Sparkles, ArrowRight, X, Loader2, ExternalLink, Copy } from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { updaterStore } from "$lib/stores/updater.svelte";
	import { settings } from "$lib/stores/settings.svelte";

	onMount(() => {
		const timer = setTimeout(() => {
			if (settings.value.autoCheckUpdates) updaterStore.check(false);
		}, 3000);
		const interval = setInterval(() => {
			if (settings.value.autoCheckUpdates) updaterStore.check(false);
		}, 1800000);
		return () => {
			clearTimeout(timer);
			clearInterval(interval);
		};
	});
</script>

{#if updaterStore.showModal}
	<div
		class="fixed inset-0 z-[100] flex items-center justify-center p-6 bg-bg-overlay/80 backdrop-blur-md"
		in:fade={{ easing: quintOut, duration: 300 }}
		out:fade={{ easing: quintOut, duration: 260 }}
	>
		<div
			class="surface-glass relative w-full max-w-lg border-brand-500/30 bg-bg-elevated/85 p-8 shadow-2xl backdrop-blur-2xl overflow-hidden"
			in:scale={{ easing: backOut, start: 0.95, duration: 300, opacity: 0 }}
		>
			<div class="absolute -top-20 -right-20 w-64 h-64 bg-brand-500/20 rounded-full blur-3xl pointer-events-none"></div>
			<div class="absolute -bottom-20 -left-20 w-64 h-64 bg-brand-500/10 rounded-full blur-3xl pointer-events-none"></div>

			{#if !updaterStore.isUpdating}
				<button
					class={launcherButton({ variant: "secondary", size: "icon", class: "absolute top-4 right-4" })}
					aria-label={uiText("ui.63601b4e70ebcb80")}
					onclick={() => (updaterStore.showModal = false)}
				>
					<X class="w-5 h-5" />
				</button>
			{/if}

			<div class="relative flex flex-col items-center text-center space-y-6">
				<div
					class="w-20 h-20 bg-brand-500/15 border-2 border-brand-500/30 rounded-full flex items-center justify-center shadow-glow"
				>
					{#if updaterStore.isUpdating}
						<Loader2 class="w-10 h-10 text-brand-500 animate-spin" />
					{:else}
						<DownloadCloud class="w-10 h-10 text-brand-500" />
					{/if}
				</div>

				<div>
					<h2 class="text-2xl font-black text-fg flex items-center justify-center gap-2">
						{#if updaterStore.isUpdating}
							{uiText("ui.4977706263fc4df6")}
						{:else}
							<Sparkles class="w-5 h-5 text-brand-500" /> {uiText("ui.f9b8aac1e8480069")}
						{/if}
					</h2>
					<p class="text-sm text-fg/60 mt-2">
						{#if updaterStore.isUpdating}
							{updaterStore.statusText || uiText("ui.b4e3a595b63952ab")}
						{:else}
							{uiText("ui.2570262db0d9b108")}
						{/if}
					</p>
				</div>

				<div class="flex items-center justify-center gap-4 w-full bg-bg-overlay/40 rounded-2xl p-4 border border-fg/5">
					<div class="flex flex-col items-center">
						<span class="text-[10px] text-fg/40 font-bold uppercase tracking-widest">{uiText("ui.078d1fdbabd0769e")}</span>
						<span class="text-lg font-mono text-fg/80 font-bold">v{updaterStore.currentVersion}</span>
					</div>
					<ArrowRight class="w-5 h-5 text-fg/30" />
					<div class="flex flex-col items-center">
						<span class="text-[10px] text-brand-500 font-bold uppercase tracking-widest">{uiText("ui.f79e00e9eecb4a42")}</span>
						<span class="text-lg font-mono text-brand-500 font-black">v{updaterStore.latestVersion}</span>
					</div>
				</div>

				{#if updaterStore.isUpdating}
					<div class="w-full space-y-3 bg-bg-overlay/30 p-4 rounded-2xl border border-fg/5">
						<div class="flex justify-between text-xs font-bold text-fg/70">
							<span>{updaterStore.statusText}</span>
							<span class="text-brand-500 font-mono">{updaterStore.progressPercent}%</span>
						</div>
						<p class="text-xs font-mono text-fg/50">{(updaterStore.transferredBytes / 1048576).toFixed(1)} MB{updaterStore.totalBytes > 0 ? ` / ${(updaterStore.totalBytes / 1048576).toFixed(1)} MB` : ""}</p>
						<div class="w-full bg-fg/10 rounded-full h-3 overflow-hidden">
							<div
								class="bg-brand-500 h-full rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-300"
								style="width: {updaterStore.progressPercent}%;"
							></div>
						</div>
					</div>
				{:else}
					<div class="w-full text-left bg-fg/5 rounded-2xl p-4 max-h-44 overflow-y-auto custom-scrollbar border border-fg/5">
						<span class="text-[10px] text-fg/40 font-bold uppercase tracking-widest block mb-2">{uiText("ui.8b4b6e51a3a788e8")}</span>
						<p class="text-xs text-fg/80 whitespace-pre-line leading-relaxed">
							{updaterStore.releaseNotes}
						</p>
					</div>
					{#if updaterStore.updateError}
						<div class="w-full rounded-2xl border border-danger/30 bg-danger/10 p-4 text-left"><p class="text-xs font-bold text-danger">{uiText("ui.7cc311f43f53578b")}</p><p class="mt-1 text-xs text-fg/65 break-words">{updaterStore.updateError}</p></div>
					{/if}
					{#if updaterStore.terminalCommand}
						<div class="w-full rounded-2xl border border-fg/10 bg-bg-overlay/50 p-3 text-left"><p class="text-[10px] font-bold uppercase tracking-wider text-fg/45">{uiText("ui.dc297a8bbc37b86b")}</p><code class="mt-2 block break-all text-xs text-fg/80">{updaterStore.terminalCommand}</code><button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "mt-3 inline-flex items-center gap-2" })} onclick={() => navigator.clipboard.writeText(updaterStore.terminalCommand)}><Copy class="w-3.5 h-3.5" />{uiText("ui.ffd3736638c756e3")}</button></div>
					{/if}

					{#if updaterStore.downloadUrl}
					<button
						class={launcherButton({ variant: "primary", size: "lg", class: "w-full flex items-center justify-center gap-2" })}
						onclick={() => updaterStore.startUpdate()}
					>
						{updaterStore.updateError ? uiText("ui.45824b20097cc608") : uiText("ui.fac72bdc709994d8")} <DownloadCloud class="w-4 h-4" />
					</button>
					{/if}
						<button class={launcherButton({ variant: "secondary", size: "lg", class: "w-full flex items-center justify-center gap-2" })} onclick={() => openUrl(updaterStore.releaseUrl || "https://github.com/predabr/luxmc/releases/latest")}>{updaterStore.updateError || !updaterStore.downloadUrl ? uiText("ui.c052e97de09a5bcb") : uiText("ui.5c3f24bca8de14ee")} <ExternalLink class="w-4 h-4" /></button>

					<button
						class={launcherButton({ variant: "ghost", size: "sm", class: "" })}
						onclick={() => (updaterStore.showModal = false)}
					>
						{uiText("ui.060ea5eae9c1ff2f")}
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}
