<script lang="ts">
import { translateUi as uiText, currentUiLocale } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { 
		Archive, 
		Plus, 
		RotateCcw, 
		Trash2, 
		X, 
		CheckCircle2, 
		Clock, 
		HardDrive, 
		Sparkles 
	} from "lucide-svelte";
	import { 
		instanceWorldSnapshotsList, 
		instanceWorldSnapshotCreate, 
		instanceWorldSnapshotRestore, 
		instanceWorldSnapshotDelete 
	} from "$lib/api/instances";
	import type { WorldSnapshotInfo } from "$lib/api/types";
	import { toast } from "$lib/stores/toasts.svelte";
	import Button from "./Button.svelte";

	type Props = {
		open: boolean;
		profileId: string;
		worldName: string;
		folderName: string;
		onClose: () => void;
		onRestored?: () => void;
	};

	let { open, profileId, worldName, folderName, onClose, onRestored }: Props = $props();

	let snapshots = $state<WorldSnapshotInfo[]>([]);
	let loading = $state(false);
	let creating = $state(false);
	let newLabel = $state("");
	let restoringId = $state<string | null>(null);

	async function loadSnapshots() {
		if (!profileId || !folderName) return;
		loading = true;
		try {
			snapshots = await instanceWorldSnapshotsList(profileId, folderName);
		} catch (e) {
			console.warn("Failed to load snapshots:", e);
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (open) {
			loadSnapshots();
		}
	});

	async function handleCreateSnapshot() {
		if (creating) return;
		creating = true;
		try {
			const snap = await instanceWorldSnapshotCreate(profileId, folderName, newLabel.trim() || undefined);
			snapshots = [snap, ...snapshots];
			newLabel = "";
			toast(uiText("ui.1f27d349af739371"), "success");
		} catch (e) {
			toast(uiText("ui.0f740ddf0c82093f") + String(e), "error");
		} finally {
			creating = false;
		}
	}

	async function handleRestore(filename: string) {
		if (!confirm(uiText("ui.02d8975f8a49a480", {arg0: (filename)}))) {
			return;
		}
		restoringId = filename;
		try {
			await instanceWorldSnapshotRestore(profileId, folderName, filename);
			toast(uiText("ui.32f8e6cb2c0962d2"), "success");
			onRestored?.();
			onClose();
		} catch (e) {
			toast(uiText("ui.bcf51d72622863de") + String(e), "error");
		} finally {
			restoringId = null;
		}
	}

	async function handleDelete(filename: string) {
		try {
			await instanceWorldSnapshotDelete(profileId, folderName, filename);
			snapshots = snapshots.filter(s => s.filename !== filename);
			toast(uiText("ui.3e7512221104485b"), "info");
		} catch (e) {
			toast(uiText("ui.56583931a7978e59") + String(e), "error");
		}
	}

	function formatDate(unixSeconds: number): string {
		const d = new Date(unixSeconds * 1000);
		return d.toLocaleDateString(currentUiLocale(), {
			day: "2-digit",
			month: "2-digit",
			year: "numeric",
			hour: "2-digit",
			minute: "2-digit",
		});
	}
</script>

{#if open}
	<div 
		class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/75 backdrop-blur-md"
		transition:fade={{ easing: quintOut, duration: 240 }}
	>
		<div 
			class="relative w-full max-w-lg bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-2xl overflow-hidden flex flex-col gap-5 max-h-[85vh]"
			transition:scale={{ easing: backOut, start: 0.95, duration: 260 }}
		>
			<!-- Top glow -->
			<div class="absolute -top-24 -right-20 w-48 h-48 bg-purple-500/15 rounded-full blur-3xl pointer-events-none"></div>

			<!-- Header -->
			<div class="flex items-start justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-purple-500/20 border border-purple-500/30 flex items-center justify-center text-purple-400">
						<Archive class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-bold text-fg">{uiText("ui.d15c18fff9858551")}</h3>
						<p class="text-xs text-fg/50">{worldName} ({folderName})</p>
					</div>
				</div>
				<button 
					type="button" 
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					onclick={onClose}
					aria-label={uiText("statusBanner.dismiss")}
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			<!-- Create Snapshot Bar -->
			<div class="flex items-center gap-2 bg-bg-subtle p-2 rounded-2xl border border-fg/5">
				<input 
					type="text" 
					placeholder={uiText("ui.a0329378a34bfc69")} 
					bind:value={newLabel}
					class="flex-1 bg-transparent px-3 py-1.5 text-xs text-fg placeholder-fg/30 outline-none"
				/>
				<Button 
					variant="primary" 
					size="sm" 
					disabled={creating}
					class="shrink-0 flex items-center gap-1.5"
					onclick={handleCreateSnapshot}
				>
					<Plus class="w-3.5 h-3.5" />
					{creating ? "Comprimindo zstd..." : uiText("ui.98b1dfca4ee6df6a")}
				</Button>
			</div>

			<!-- Snapshot list -->
			<div class="flex-1 overflow-y-auto space-y-2.5 custom-scrollbar pr-1 min-h-[160px]">
				{#if loading}
					<div class="flex items-center justify-center py-12 text-fg/40 text-xs">
						{uiText("ui.9158b55f1f6ce2ea")}
					</div>
				{:else if snapshots.length === 0}
					<div class="flex flex-col items-center justify-center py-12 text-center text-fg/40">
						<Archive class="w-10 h-10 mb-2 opacity-30" />
						<div class="text-xs font-bold text-fg/60">{uiText("ui.5ae0229aba431fe0")}</div>
						<div class="text-[11px] mt-1">{uiText("ui.6266c84f36ddccb2")}</div>
					</div>
				{:else}
					{#each snapshots as snap}
						<div class="bg-bg-subtle border border-fg/5 hover:border-fg/10 rounded-2xl p-3.5 flex items-center justify-between transition-[color,background-color,border-color,box-shadow,transform,opacity] group">
							<div class="flex items-center gap-3 min-w-0">
								<div class="w-8 h-8 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400 shrink-0">
									<Clock class="w-4 h-4" />
								</div>
								<div class="min-w-0">
									<div class="text-xs font-bold text-fg truncate">{snap.label}</div>
									<div class="flex items-center gap-2 mt-0.5 text-[10px] text-fg/40">
										<span>{formatDate(snap.createdAt)}</span>
										<span>·</span>
										<span>{(snap.sizeBytes / (1024 * 1024)).toFixed(2)} {uiText("ui.477ae5a1a2358517")}</span>
									</div>
								</div>
							</div>

							<div class="flex items-center gap-1.5 shrink-0">
								<button 
									type="button"
									class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5 disabled:opacity-50" })}
									disabled={restoringId === snap.filename}
									onclick={() => handleRestore(snap.filename)}
									title={uiText("ui.5ad9d5ce30138874")}
								>
									<RotateCcw class="w-3 h-3 {restoringId === snap.filename ? 'animate-spin' : ''}" />
									{restoringId === snap.filename ? "Restaurando..." : uiText("shortcutsModal.reset")}
								</button>
								<button 
									type="button"
									class={launcherButton({ variant: "danger", size: "icon", class: "" })}
									onclick={() => handleDelete(snap.filename)}
									title={uiText("ui.75877f2d1f55fbe5")}
									aria-label={uiText("ui.75877f2d1f55fbe5")}
								>
									<Trash2 class="w-3.5 h-3.5" />
								</button>
							</div>
						</div>
					{/each}
				{/if}
			</div>

			<!-- Footer info -->
			<div class="pt-2 border-t border-fg/5 flex items-center justify-between text-[11px] text-fg/40">
				<span>{uiText("ui.5d6e0cf88c17d1e5")}</span>
				<Button variant="secondary" size="sm" onclick={onClose}>
					{uiText("statusBanner.dismiss")}
				</Button>
			</div>
		</div>
	</div>
{/if}
