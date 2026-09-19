<script lang="ts">
	import { onMount } from "svelte";
	import { 
		Download, 
		Package, 
		Check, 
		RefreshCw, 
		HardDrive, 
		ExternalLink, 
		Layers, 
		Boxes, 
		FolderOpen,
		Sparkles
	} from "lucide-svelte";
	import { api } from "$lib/api/client";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import Modal from "$lib/components/ui/Modal.svelte";
	import Button from "$lib/components/ui/Button.svelte";

	type ExternalInstance = {
		launcher: string;
		name: string;
		path: string;
		mcVersion: string;
		loader: string;
		modCount: number;
		hasSaves: boolean;
	};

	type ImportResult = {
		success: boolean;
		profileId: string;
		name: string;
		message: string;
	};

	type Props = {
		open: boolean;
		onClose: () => void;
	};

	let { open, onClose }: Props = $props();

	let detected = $state<ExternalInstance[]>([]);
	let loading = $state(false);
	let importingPath = $state<string | null>(null);

	async function scanLaunchers() {
		loading = true;
		try {
			detected = await api.invoke<ExternalInstance[]>("importer_detect_launchers");
		} catch (e) {
			toast(`Erro ao escanear launchers: ${e}`, "error");
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (open) {
			scanLaunchers();
		}
	});

	async function handleImport(inst: ExternalInstance) {
		importingPath = inst.path;
		toast(`Importando ${inst.name} do ${inst.launcher}...`, "info");

		try {
			const res = await api.invoke<ImportResult>("importer_execute_import", {
				sourcePath: inst.path,
				targetName: inst.name,
				mcVersion: inst.mcVersion,
				loader: inst.loader
			});

			if (res.success) {
				toast(res.message, "success");
				await profiles.refresh();
				onClose();
			}
		} catch (e) {
			toast(`Falha na importação: ${e}`, "error");
		} finally {
			importingPath = null;
		}
	}
</script>

<Modal isOpen={open} {onClose} title="Importador Universal em 1 Clique">
	<div class="flex flex-col gap-4 text-xs">
		<div class="flex items-center justify-between">
			<p class="text-fg/70 leading-relaxed max-w-md">
				O Luxmc varre seu computador procurando instâncias do <strong>Prism Launcher</strong>, <strong>CurseForge</strong>, <strong>Lunar Client</strong> e <strong>Badlion</strong> para você migrar com 1 clique sem precisar rebaixar nada.
			</p>
			<Button variant="secondary" size="sm" disabled={loading} onclick={scanLaunchers}>
				<RefreshCw class="w-3.5 h-3.5 {loading ? 'animate-spin' : ''}" />
				Reescanear
			</Button>
		</div>

		{#if loading}
			<div class="py-12 flex flex-col items-center justify-center gap-2 text-fg/50">
				<RefreshCw class="w-6 h-6 animate-spin text-brand-400" />
				<span>Varrendo discos do sistema...</span>
			</div>
		{:else if detected.length === 0}
			<div class="py-10 flex flex-col items-center justify-center gap-2 text-center bg-bg-subtle rounded-2xl border border-fg/5 p-6">
				<HardDrive class="w-8 h-8 text-fg/30" />
				<h4 class="text-fg font-bold text-sm">Nenhum launcher externo encontrado</h4>
				<p class="text-fg/50 text-[11px] max-w-xs">
					Não encontramos pastas padrão do Prism, CurseForge, Lunar ou Badlion no seu sistema.
				</p>
			</div>
		{:else}
			<div class="flex flex-col gap-2.5 max-h-[380px] overflow-y-auto custom-scrollbar pr-1">
				{#each detected as inst}
					<div class="bg-bg-subtle border border-fg/10 hover:border-brand-400/40 rounded-2xl p-3.5 flex items-center justify-between gap-3 transition">
						<div class="flex items-center gap-3 min-w-0">
							<div class="w-10 h-10 rounded-xl bg-fg/5 border border-fg/10 flex items-center justify-center shrink-0">
								<Boxes class="w-5 h-5 text-brand-400" />
							</div>
							<div class="min-w-0">
								<div class="flex items-center gap-2">
									<h4 class="text-fg font-bold text-xs truncate">{inst.name}</h4>
									<span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-brand-400/15 text-brand-400 border border-brand-400/30 shrink-0">
										{inst.launcher}
									</span>
								</div>
								<div class="flex items-center gap-3 text-[11px] text-fg/50 mt-1 font-mono">
									<span>MC {inst.mcVersion}</span>
									<span>·</span>
									<span class="capitalize">{inst.loader}</span>
									{#if inst.modCount > 0}
										<span>·</span>
										<span>{inst.modCount} mods</span>
									{/if}
									{#if inst.hasSaves}
										<span>·</span>
										<span class="text-emerald-400 font-sans font-semibold">Com Mundos</span>
									{/if}
								</div>
							</div>
						</div>

						<Button 
							variant="primary" 
							size="sm" 
							disabled={importingPath === inst.path} 
							onclick={() => handleImport(inst)}
						>
							{#if importingPath === inst.path}
								<RefreshCw class="w-3.5 h-3.5 animate-spin" />
								<span>Importando...</span>
							{:else}
								<Download class="w-3.5 h-3.5" />
								<span>Importar</span>
							{/if}
						</Button>
					</div>
				{/each}
			</div>
		{/if}

		<div class="flex items-center justify-end pt-3 border-t border-fg/5">
			<Button variant="secondary" size="sm" onclick={onClose}>
				Fechar
			</Button>
		</div>
	</div>
</Modal>
