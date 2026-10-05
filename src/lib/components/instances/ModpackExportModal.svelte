<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { fade, scale } from "svelte/transition";
	import {
		Package,
		Download,
		X,
		Check,
		FolderOpen,
		Sparkles,
		FileArchive
	} from "lucide-svelte";
	import { instanceExportModpack, instanceExportShareCode, type ExportResult } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let {
		isOpen = $bindable(false),
		profileId,
		instanceName,
		onClose
	}: {
		isOpen: boolean;
		profileId: string;
		instanceName: string;
		onClose: () => void;
	} = $props();

	let exportFormat = $state<"mrpack" | "zip">("mrpack");
	let customName = $state("MeuModpack");
	let isExporting = $state(false);
	let exportResult = $state<ExportResult | null>(null);
	let shareCode = $state<string | null>(null);
	let isGeneratingCode = $state(false);

	$effect(() => {
		if (isOpen) {
			customName = instanceName || "MeuModpack";
			exportResult = null;
			shareCode = null;
		}
	});

	async function handleExport() {
		isExporting = true;
		try {
			const res = await instanceExportModpack(profileId, exportFormat, customName.trim());
			exportResult = res;
			playSound("click");
			toast(uiText("ui.eedbfeea413bd317", {arg0: ((res.fileSize / (1024 * 1024)).toFixed(1))}), "success");
		} catch (e) {
			toast(uiText("ui.b3e16980c03ee756") + String(e), "error");
		} finally {
			isExporting = false;
		}
	}

	async function handleShareCode() {
		isGeneratingCode = true;
		try {
			shareCode = await instanceExportShareCode(profileId);
			await navigator.clipboard.writeText(shareCode);
			playSound("chime");
			toast(uiText("ui.f0088fc5edc7c9fa"), "success");
		} catch (error) {
			toast(uiText("ui.9f46d6ffe73ac2ca") + String(error), "error");
		} finally {
			isGeneratingCode = false;
		}
	}
</script>

{#if isOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-md select-none" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="w-full max-w-md rounded-3xl bg-bg-elevated border border-fg/15 p-6 shadow-2xl space-y-5" in:scale={{ easing: backOut, start: 0.95, duration: 260 }}>
			<!-- Header -->
			<div class="flex items-center justify-between border-b border-fg/10 pb-4">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center text-brand-400">
						<Package class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-sm font-black text-fg">{uiText("ui.cf47ee649de9bebc")}</h3>
						<p class="text-[11px] text-fg/50">{uiText("ui.50d86095bfda4096")}</p>
					</div>
				</div>

				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					onclick={onClose}
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			{#if exportResult}
				<!-- Result Card -->
				<div class="p-5 rounded-2xl bg-emerald-500/10 border border-emerald-500/30 space-y-3" in:fade={{ easing: quintOut, duration: 220 }}>
					<div class="flex items-center gap-2 text-emerald-400 font-bold text-xs">
						<Check class="w-4 h-4 stroke-[3]" />
						<span>{uiText("ui.9df6ed355d1e33ab")}</span>
					</div>
					<div class="space-y-1 text-xs text-fg/70 font-mono">
						<div>{uiText("ui.01b27f9c9a539b33")} <span class="text-fg font-bold">{exportResult.filePath.split('/').pop()}</span></div>
						<div>{uiText("ui.999e8aab92fc9678")} <span class="text-fg font-bold">{(exportResult.fileSize / (1024 * 1024)).toFixed(2)} MB</span></div>
						<div>{uiText("ui.7b28006943e70357")} <span class="text-fg font-bold">{exportResult.totalMods} {uiText("ui.695073cb6649c0a4")}</span></div>
					</div>
					<p class="text-[10px] text-fg/40 pt-1">
						{uiText("ui.7ebb0804f71b8c7f")} <br />
						<span class="text-fg/60 select-all font-mono break-all">{exportResult.filePath}</span>
					</p>
				</div>
			{:else}
				<!-- Configuration -->
				<div class="space-y-4">
					<div class="space-y-1.5">
						<label for="export-name-input" class="text-xs font-bold text-fg/80 block">{uiText("ui.bcaae46e1a25cdbd")}</label>
						<input
							id="export-name-input"
							type="text"
							bind:value={customName}
							placeholder={uiText("ui.3ed0bb6dda005c5d")}
							class="w-full px-4 py-2.5 rounded-xl bg-bg-overlay/40 border border-fg/10 text-xs text-fg focus:outline-none focus:border-brand-500 font-mono"
						/>
					</div>

					<div class="space-y-1.5">
						<span class="text-xs font-bold text-fg/80 block">{uiText("ui.de849082518c6b9a")}</span>
						<div class="grid grid-cols-2 gap-2.5">
							<button
								type="button"
								class="p-3 rounded-2xl border text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex flex-col justify-between {exportFormat === 'mrpack' ? 'bg-emerald-500/15 border-emerald-500/40 text-fg' : 'bg-bg-elevated hover:bg-bg-subtle border-fg/5 text-fg/60'}"
								onclick={() => exportFormat = 'mrpack'}
							>
								<div class="flex items-center justify-between">
									<span class="text-xs font-black">{uiText("ui.12c5a295d3930ca1")}</span>
									<span class="text-[9px] font-bold text-emerald-400 bg-emerald-500/20 px-1.5 py-0.5 rounded">Modrinth</span>
								</div>
								<span class="text-[10px] text-fg/40 mt-1.5">{uiText("ui.cf02b2793ff1b9ef")}</span>
							</button>

							<button
								type="button"
								class="p-3 rounded-2xl border text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex flex-col justify-between {exportFormat === 'zip' ? 'bg-amber-500/15 border-amber-500/40 text-fg' : 'bg-bg-elevated hover:bg-bg-subtle border-fg/5 text-fg/60'}"
								onclick={() => exportFormat = 'zip'}
							>
								<div class="flex items-center justify-between">
									<span class="text-xs font-black">{uiText("ui.141b90a31b4cbfbd")}</span>
									<span class="text-[9px] font-bold text-amber-400 bg-amber-500/20 px-1.5 py-0.5 rounded">CurseForge</span>
								</div>
								<span class="text-[10px] text-fg/40 mt-1.5">{uiText("ui.d429a9f9d661ba1b")}</span>
							</button>
						</div>
					</div>

					<div class="p-3 rounded-2xl bg-fg/5 border border-fg/5 text-[11px] text-fg/50 leading-relaxed">
						{uiText("ui.bb96e8263c95f96f")}
					</div>
				</div>
			{/if}

			<div class="rounded-2xl border border-brand-500/20 bg-brand-500/5 p-4 space-y-3">
				<div><p class="text-xs font-black text-fg">{uiText("ui.aaba02f2deb782dc")}</p><p class="mt-1 text-[10px] text-fg/50">{uiText("ui.3066016ea380efae")}</p></div>
				{#if shareCode}<textarea readonly class="h-20 w-full rounded-xl border border-fg/10 bg-bg-overlay/40 p-2 font-mono text-[10px] text-fg" value={shareCode}></textarea>{/if}
				<button type="button" class={launcherButton({ variant: "ghostBrand", size: "sm", class: "disabled:opacity-50" })} disabled={isGeneratingCode} onclick={handleShareCode}>{isGeneratingCode ? uiText("ui.f16f04e767446b97") : uiText("ui.e72d97ef42b859cc")}</button>
			</div>

			<!-- Actions -->
			<div class="flex items-center justify-end gap-2.5 pt-2 border-t border-fg/10">
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
					onclick={onClose}
				>
					{exportResult ? uiText("statusBanner.dismiss") : uiText("common.cancel")}
				</button>

				{#if !exportResult}
					<button
						type="button"
						class={launcherButton({ variant: "primary", size: "sm", class: "uppercase tracking-wider disabled:opacity-50 flex items-center gap-2" })}
						onclick={handleExport}
						disabled={isExporting}
					>
						{#if isExporting}
							<div class="w-4 h-4 rounded-full border-2 border-bg-overlay border-t-transparent animate-spin"></div>
							<span>{uiText("ui.e50a7ac800f3396c")}</span>
						{:else}
							<Download class="w-4 h-4 stroke-[2.5]" />
							<span>{uiText("ui.bd1322ed1c800669")}</span>
						{/if}
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}
