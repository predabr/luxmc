<script lang="ts">
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
			toast("Falha ao carregar lista de backups: " + String(e), "error");
		} finally {
			isLoading = false;
		}
	}

	async function handleCreateBackup() {
		if (!selectedWorldToBackup) {
			toast("Selecione um mundo para fazer backup", "warning");
			return;
		}
		isBackingUp = true;
		try {
			const entry = await instanceBackupWorld(profileId, selectedWorldToBackup);
			toast(`Backup de "${entry.worldName}" criado com sucesso!`, "success");
			playSound("click");
			loadBackups();
		} catch (e) {
			toast("Erro ao criar backup: " + String(e), "error");
		} finally {
			isBackingUp = false;
		}
	}
</script>

{#if isOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md select-none" in:fade={{ duration: 150 }}>
		<div class="w-full max-w-2xl rounded-3xl bg-[#141518] border border-white/15 p-6 shadow-2xl space-y-5 max-h-[90vh] flex flex-col" in:scale={{ start: 0.95, duration: 200 }}>
			<!-- Header -->
			<div class="flex items-center justify-between border-b border-white/10 pb-4 shrink-0">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
						<ShieldCheck class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-black text-white">Nuvem Local & Backups de Mundos</h3>
						<p class="text-xs text-white/50 mt-0.5">Snapshots compactados para nunca perder suas construções</p>
					</div>
				</div>

				<button
					type="button"
					class="p-2 rounded-xl text-white/50 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
					onclick={onClose}
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			<!-- Create Backup Action Card -->
			<div class="p-4 rounded-2xl bg-[#18191c] border border-white/10 flex flex-col sm:flex-row items-center justify-between gap-3 shrink-0">
				<div class="flex items-center gap-3 w-full sm:w-auto">
					<span class="text-xs font-bold text-white whitespace-nowrap">Mundo:</span>
					<select
						bind:value={selectedWorldToBackup}
						class="flex-1 sm:w-60 px-3 py-2 rounded-xl bg-black/40 border border-white/10 text-xs text-white font-bold focus:outline-none focus:border-brand-500"
					>
						{#each worldsList as w}
							<option value={w.folderName}>{w.name} ({w.folderName})</option>
						{/each}
					</select>
				</div>

				<button
					type="button"
					class="w-full sm:w-auto px-6 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-black font-black text-xs uppercase tracking-wider flex items-center justify-center gap-2 transition-all cursor-pointer shadow-lg shadow-emerald-500/20 active:scale-95 disabled:opacity-50"
					onclick={handleCreateBackup}
					disabled={isBackingUp || !selectedWorldToBackup}
				>
					{#if isBackingUp}
						<RefreshCw class="w-4 h-4 animate-spin" />
						<span>Criando Backup...</span>
					{:else}
						<Archive class="w-4 h-4 stroke-[2.5]" />
						<span>Criar Backup Agora</span>
					{/if}
				</button>
			</div>

			<!-- Backups List -->
			<div class="flex-1 overflow-y-auto pr-1 space-y-2.5 custom-scrollbar">
				<div class="flex items-center justify-between text-xs text-white/50 font-bold px-1">
					<span>Backups Existentes ({backups.length})</span>
					<span class="text-[10px] text-white/40">Mantém automaticamente os últimos 5 backups por mundo</span>
				</div>

				{#if isLoading}
					<div class="p-10 text-center text-white/40 flex flex-col items-center justify-center gap-2">
						<RefreshCw class="w-6 h-6 animate-spin text-emerald-400" />
						<span class="text-xs font-bold">Verificando snapshots...</span>
					</div>
				{:else if backups.length === 0}
					<div class="p-12 text-center text-white/40 rounded-2xl bg-[#18191c] border border-white/5">
						<p class="text-xs">Nenhum snapshot de backup encontrado ainda.</p>
						<p class="text-[11px] text-white/30 mt-1">Crie seu primeiro backup seguro acima.</p>
					</div>
				{:else}
					{#each backups as b}
						<div class="p-3.5 rounded-2xl bg-[#18191c] border border-white/5 hover:border-white/15 transition-all flex items-center justify-between gap-3">
							<div class="min-w-0 space-y-0.5">
								<div class="flex items-center gap-2">
									<h4 class="text-xs font-bold text-white truncate">{b.worldName}</h4>
									<span class="text-[9px] font-mono font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">
										{(b.sizeBytes / (1024 * 1024)).toFixed(2)} MB
									</span>
								</div>
								<div class="flex items-center gap-2 text-[10px] text-white/40 font-mono">
									<Clock class="w-3 h-3 text-white/30" />
									<span>{b.createdAt}</span>
									<span>·</span>
									<span class="truncate max-w-[280px]">{b.fileName}</span>
								</div>
							</div>

							<div class="flex items-center gap-2 shrink-0">
								<button
									type="button"
									class="px-3.5 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 text-white/80 hover:text-white text-xs font-bold border border-white/10 transition-all cursor-pointer flex items-center gap-1.5 active:scale-95"
									onclick={() => {
										navigator.clipboard.writeText(b.filePath);
										toast("Caminho do backup copiado!", "success");
									}}
									title="Copiar caminho no disco"
								>
									<span>Copiar Caminho</span>
								</button>
							</div>
						</div>
					{/each}
				{/if}
			</div>

			<!-- Footer -->
			<div class="flex items-center justify-end pt-3 border-t border-white/10 shrink-0">
				<button
					type="button"
					class="px-6 py-2.5 rounded-2xl bg-white/5 hover:bg-white/10 text-white/80 hover:text-white font-bold text-xs transition-colors cursor-pointer"
					onclick={onClose}
				>
					Fechar
				</button>
			</div>
		</div>
	</div>
{/if}
