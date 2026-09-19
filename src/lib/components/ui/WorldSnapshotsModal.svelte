<script lang="ts">
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
			toast("Snapshot ultra-rápido (zstd) criado com sucesso!", "success");
		} catch (e) {
			toast("Erro ao criar snapshot: " + String(e), "error");
		} finally {
			creating = false;
		}
	}

	async function handleRestore(filename: string) {
		if (!confirm(`Deseja restaurar o mundo para o estado deste snapshot (${filename})? O estado atual do mundo será sobrescrito.`)) {
			return;
		}
		restoringId = filename;
		try {
			await instanceWorldSnapshotRestore(profileId, folderName, filename);
			toast("Mundo restaurado com sucesso para este ponto!", "success");
			onRestored?.();
			onClose();
		} catch (e) {
			toast("Erro ao restaurar snapshot: " + String(e), "error");
		} finally {
			restoringId = null;
		}
	}

	async function handleDelete(filename: string) {
		try {
			await instanceWorldSnapshotDelete(profileId, folderName, filename);
			snapshots = snapshots.filter(s => s.filename !== filename);
			toast("Snapshot excluído!", "info");
		} catch (e) {
			toast("Erro ao excluir snapshot: " + String(e), "error");
		}
	}

	function formatDate(unixSeconds: number): string {
		const d = new Date(unixSeconds * 1000);
		return d.toLocaleDateString("pt-BR", {
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
		transition:fade={{ duration: 180 }}
	>
		<div 
			class="relative w-full max-w-lg bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-2xl overflow-hidden flex flex-col gap-5 max-h-[85vh]"
			transition:scale={{ start: 0.95, duration: 200 }}
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
						<h3 class="text-base font-bold text-fg">Time Machine de Mundos</h3>
						<p class="text-xs text-fg/50">{worldName} ({folderName})</p>
					</div>
				</div>
				<button 
					type="button" 
					class="p-2 rounded-xl text-fg/40 hover:text-fg hover:bg-fg/5 transition-colors cursor-pointer"
					onclick={onClose}
					aria-label="Fechar"
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			<!-- Create Snapshot Bar -->
			<div class="flex items-center gap-2 bg-bg-subtle p-2 rounded-2xl border border-fg/5">
				<input 
					type="text" 
					placeholder="Identificador do Snapshot (ex: Antes do Wither)..." 
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
					{creating ? "Comprimindo zstd..." : "Criar Snapshot"}
				</Button>
			</div>

			<!-- Snapshot list -->
			<div class="flex-1 overflow-y-auto space-y-2.5 custom-scrollbar pr-1 min-h-[160px]">
				{#if loading}
					<div class="flex items-center justify-center py-12 text-fg/40 text-xs">
						Carregando snapshots do mundo...
					</div>
				{:else if snapshots.length === 0}
					<div class="flex flex-col items-center justify-center py-12 text-center text-fg/40">
						<Archive class="w-10 h-10 mb-2 opacity-30" />
						<div class="text-xs font-bold text-fg/60">Nenhum snapshot gravado</div>
						<div class="text-[11px] mt-1">Crie seu primeiro ponto de restauração instantâneo com compressão zstd!</div>
					</div>
				{:else}
					{#each snapshots as snap}
						<div class="bg-bg-subtle border border-fg/5 hover:border-fg/10 rounded-2xl p-3.5 flex items-center justify-between transition-all group">
							<div class="flex items-center gap-3 min-w-0">
								<div class="w-8 h-8 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400 shrink-0">
									<Clock class="w-4 h-4" />
								</div>
								<div class="min-w-0">
									<div class="text-xs font-bold text-fg truncate">{snap.label}</div>
									<div class="flex items-center gap-2 mt-0.5 text-[10px] text-fg/40">
										<span>{formatDate(snap.createdAt)}</span>
										<span>·</span>
										<span>{(snap.sizeBytes / (1024 * 1024)).toFixed(2)} MB (zstd)</span>
									</div>
								</div>
							</div>

							<div class="flex items-center gap-1.5 shrink-0">
								<button 
									type="button"
									class="px-3 py-1.5 rounded-xl bg-purple-500/15 hover:bg-purple-500/25 text-purple-300 border border-purple-500/30 text-xs font-bold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
									disabled={restoringId === snap.filename}
									onclick={() => handleRestore(snap.filename)}
									title="Restaurar mundo para este ponto"
								>
									<RotateCcw class="w-3 h-3 {restoringId === snap.filename ? 'animate-spin' : ''}" />
									{restoringId === snap.filename ? "Restaurando..." : "Restaurar"}
								</button>
								<button 
									type="button"
									class="p-2 rounded-xl text-fg/40 hover:text-red-400 hover:bg-red-500/10 transition-colors cursor-pointer"
									onclick={() => handleDelete(snap.filename)}
									title="Excluir snapshot"
									aria-label="Excluir snapshot"
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
				<span>Algoritmo: Zstandard level 3</span>
				<Button variant="secondary" size="sm" onclick={onClose}>
					Fechar
				</Button>
			</div>
		</div>
	</div>
{/if}
