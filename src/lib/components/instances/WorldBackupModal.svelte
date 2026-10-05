<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import {
		Archive,
		Download,
		X,
		Check,
		FolderOpen,
		RefreshCw,
		HardDrive,
		Clock,
		ShieldCheck
	} from "lucide-svelte";
	import {
		instanceListWorldBackups,
		instanceBackupWorld,
		type WorldBackupEntry,
		type WorldDetail
	} from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let {
		isOpen = $bindable(false),
		profileId,
		worldsList,
		onClose
	}: {
		isOpen: boolean;
		profileId: string;
		worldsList: WorldDetail[];
		onClose: () => void;
	} = $props();

	let backups = $state<WorldBackupEntry[]>([]);
	let isLoading = $state(true);
	let isBackingUp = $state(false);
	let selectedWorldToBackup = $state<string>("");

	$effect(() => {
		if (isOpen && profileId) {
			if (worldsList.length > 0 && !selectedWorldToBackup) {
				selectedWorldToBackup = worldsList[0].folderName;
			}
			loadBackups();
		}
	});

	async function loadBackups() {
		isLoading = true;
		try {
			backups = await instanceListWorldBackups(profileId);
		} catch (e) {
			toast(uiText("ui.083266c511d0f671") + String(e), "error");
		} finally {
			isLoading = false;
		}
	}

	async function handleCreateBackup() {
		if (!selectedWorldToBackup) {
			toast(uiText("ui.fc1ad215380be4fe"), "warning");
			return;
		}
		isBackingUp = true;
		try {
			const entry = await instanceBackupWorld(profileId, selectedWorldToBackup);
			toast(uiText("ui.746bb3ad8b64756e", {arg0: (entry.worldName)}), "success");
			playSound("click");
			loadBackups();
		} catch (e) {
			toast(uiText("ui.b75153e767c784f6") + String(e), "error");
		} finally {
			isBackingUp = false;
		}
	}
</script>

{#if isOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-md select-none" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="w-full max-w-2xl rounded-3xl bg-bg-elevated border border-fg/15 p-6 shadow-2xl space-y-5 max-h-[90vh] flex flex-col" in:scale={{ easing: backOut, start: 0.95, duration: 260 }}>
			<!-- Header -->
			<div class="flex items-center justify-between border-b border-fg/10 pb-4 shrink-0">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
						<ShieldCheck class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-black text-fg">{uiText("ui.0f9f54375d34efe4")}</h3>
						<p class="text-xs text-fg/50 mt-0.5">{uiText("ui.98a4da599ed37e78")}</p>
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

			<!-- Create Backup Action Card -->
			<div class="p-4 rounded-2xl bg-bg-elevated border border-fg/10 flex flex-col sm:flex-row items-center justify-between gap-3 shrink-0">
				<div class="flex items-center gap-3 w-full sm:w-auto">
					<span class="text-xs font-bold text-fg whitespace-nowrap">{uiText("ui.d56424910a34130b")}</span>
					<select
						bind:value={selectedWorldToBackup}
						class="flex-1 sm:w-60 px-3 py-2 rounded-xl bg-bg-overlay/40 border border-fg/10 text-xs text-fg font-bold focus:outline-none focus:border-brand-500"
					>
						{#each worldsList as w (w.folderName)}
							<option value={w.folderName}>{w.name} ({w.folderName})</option>
						{/each}
					</select>
				</div>

				<button
					type="button"
					class={launcherButton({ variant: "primary", size: "sm", class: "w-full sm:w-auto uppercase tracking-wider flex items-center justify-center gap-2 disabled:opacity-50" })}
					onclick={handleCreateBackup}
					disabled={isBackingUp || !selectedWorldToBackup}
				>
					{#if isBackingUp}
						<RefreshCw class="w-4 h-4 animate-spin" />
						<span>{uiText("ui.69ee9c2aa5f3caf0")}</span>
					{:else}
						<Archive class="w-4 h-4 stroke-[2.5]" />
						<span>{uiText("ui.154c293fbbed8dfe")}</span>
					{/if}
				</button>
			</div>

			<!-- Backups List -->
			<div class="flex-1 overflow-y-auto pr-1 space-y-2.5 custom-scrollbar">
				<div class="flex items-center justify-between text-xs text-fg/50 font-bold px-1">
					<span>{uiText("ui.e0e6a8abdd00b5ef")}{backups.length})</span>
					<span class="text-[10px] text-fg/40">{uiText("ui.d4285b24752a6e23")}</span>
				</div>

				{#if isLoading}
					<div class="p-10 text-center text-fg/40 flex flex-col items-center justify-center gap-2">
						<RefreshCw class="w-6 h-6 animate-spin text-emerald-400" />
						<span class="text-xs font-bold">{uiText("ui.72203f5d85fc670e")}</span>
					</div>
				{:else if backups.length === 0}
					<div class="p-12 text-center text-fg/40 rounded-2xl bg-bg-elevated border border-fg/5">
						<p class="text-xs">{uiText("ui.70b86cc810ccdd39")}</p>
						<p class="text-[11px] text-fg/30 mt-1">{uiText("ui.b350644e9ea7b638")}</p>
					</div>
				{:else}
					{#each backups as b (b.filePath)}
						<div class="p-3.5 rounded-2xl bg-bg-elevated border border-fg/5 hover:border-fg/15 transition-[color,background-color,border-color,box-shadow,transform,opacity] flex items-center justify-between gap-3">
							<div class="min-w-0 space-y-0.5">
								<div class="flex items-center gap-2">
									<h4 class="text-xs font-bold text-fg truncate">{b.worldName}</h4>
									<span class="text-[9px] font-mono font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">
										{(b.sizeBytes / (1024 * 1024)).toFixed(2)} MB
									</span>
								</div>
								<div class="flex items-center gap-2 text-[10px] text-fg/40 font-mono">
									<Clock class="w-3 h-3 text-fg/30" />
									<span>{b.createdAt}</span>
									<span>·</span>
									<span class="truncate max-w-[280px]">{b.fileName}</span>
								</div>
							</div>

							<div class="flex items-center gap-2 shrink-0">
								<button
									type="button"
									class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
									onclick={() => {
										navigator.clipboard.writeText(b.filePath);
										toast("Caminho do backup copiado!", "success");
									}}
									title={uiText("ui.6012ca7178f15af1")}
								>
									<span>{uiText("ui.1c031de96f1a554d")}</span>
								</button>
							</div>
						</div>
					{/each}
				{/if}
			</div>

			<!-- Footer -->
			<div class="flex items-center justify-end pt-3 border-t border-fg/10 shrink-0">
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
					onclick={onClose}
				>
					{uiText("statusBanner.dismiss")}
				</button>
			</div>
		</div>
	</div>
{/if}
