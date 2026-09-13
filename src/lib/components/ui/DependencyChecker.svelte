<script lang="ts">
	import { PackageCheck, AlertTriangle, Download, ChevronDown, ChevronUp, Loader2 } from "lucide-svelte";
	import { modsCheckMissingDeps, modsInstallMissingDeps } from "$lib/api/java";
	import type { MissingDep } from "$lib/api/types";
	import { toast } from "$lib/stores/toasts.svelte";

	type Props = {
		profileId: string;
		mcVersion: string;
		loader: string;
		onInstalled?: () => void;
	};

	let { profileId, mcVersion, loader, onInstalled }: Props = $props();

	let checking = $state(false);
	let installing = $state(false);
	let checked = $state(false);
	let missing = $state<MissingDep[]>([]);
	let expanded = $state(false);

	async function check() {
		checking = true;
		try {
			const result = await modsCheckMissingDeps(profileId, mcVersion, loader);
			missing = result.missing;
			checked = true;
			expanded = result.missing.length > 0;
		} catch {
			toast("Erro ao verificar dependências", "error");
		} finally {
			checking = false;
		}
	}

	async function installAll() {
		if (missing.length === 0) return;
		installing = true;
		try {
			const ids = missing.map((d) => d.projectId);
			const count = await modsInstallMissingDeps(profileId, mcVersion, loader, ids);
			toast(`${count} dependência(s) instalada(s) com sucesso!`, "success");
			missing = [];
			onInstalled?.();
		} catch (e) {
			toast(`Erro ao instalar dependências: ${e}`, "error");
		} finally {
			installing = false;
		}
	}
</script>

<div class="rounded-2xl border overflow-hidden transition-all {checked && missing.length > 0 ? 'border-warning/30 bg-warning/5' : checked ? 'border-emerald-500/20 bg-emerald-500/5' : 'border-white/5 bg-bg-subtle'}">
	<button
		type="button"
		onclick={() => { if (!checked) { check(); } else { expanded = !expanded; } }}
		class="w-full flex items-center justify-between gap-3 px-4 py-3 cursor-pointer"
	>
		<div class="flex items-center gap-3">
			{#if checking}
				<Loader2 class="w-4 h-4 text-brand-400 animate-spin" />
				<span class="text-xs font-bold text-white/70">Verificando dependências...</span>
			{:else if checked && missing.length === 0}
				<PackageCheck class="w-4 h-4 text-emerald-400" />
				<span class="text-xs font-bold text-emerald-400">Todas as dependências estão instaladas</span>
			{:else if checked && missing.length > 0}
				<AlertTriangle class="w-4 h-4 text-warning" />
				<span class="text-xs font-bold text-warning">{missing.length} dependência(s) faltando</span>
			{:else}
				<PackageCheck class="w-4 h-4 text-white/40" />
				<span class="text-xs font-medium text-white/50">Verificar dependências de mods</span>
			{/if}
		</div>
		{#if checked && missing.length > 0}
			{#if expanded}
				<ChevronUp class="w-3.5 h-3.5 text-white/30" />
			{:else}
				<ChevronDown class="w-3.5 h-3.5 text-white/30" />
			{/if}
		{/if}
	</button>

	{#if expanded && missing.length > 0}
		<div class="border-t border-white/5 px-4 py-3 flex flex-col gap-3">
			<div class="flex flex-col gap-1.5">
				{#each missing as dep}
					<div class="flex items-center justify-between text-[11px] py-1">
						<span class="font-bold text-white/80">{dep.name}</span>
						<span class="text-white/30 font-mono">{dep.slug}</span>
					</div>
				{/each}
			</div>
			<button
				type="button"
				disabled={installing}
				onclick={installAll}
				class="flex items-center justify-center gap-2 w-full py-2.5 rounded-xl bg-brand-500 hover:bg-brand-400 disabled:opacity-60 text-white text-xs font-black transition-all cursor-pointer shadow-glow"
			>
				{#if installing}
					<Loader2 class="w-3.5 h-3.5 animate-spin" />
					Instalando...
				{:else}
					<Download class="w-3.5 h-3.5" />
					Instalar {missing.length} Dependência(s) — 1 Clique
				{/if}
			</button>
		</div>
	{/if}
</div>
