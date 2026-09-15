<script lang="ts">
	import { fade, slide } from "svelte/transition";
	import { 
		Play, 
		Maximize2, 
		Clock, 
		Server, 
		Sparkles, 
		Loader2,
		ShieldCheck
	} from "lucide-svelte";
	import { layoutStore } from "$lib/stores/layout.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { launchGame } from "$lib/api";
	import { playSound } from "$lib/utils/sound";

	const activeInstance = $derived(profiles.active || profiles.list[0] || null);
	let isLaunching = $state(false);

	async function handlePlay() {
		if (!activeInstance || !account.value) {
			toast("Selecione uma conta e instância para jogar", "warning");
			return;
		}

		isLaunching = true;
		playSound("launch");

		try {
			await launchGame({
				versionId: activeInstance.mcVersion,
				accountId: account.value.id,
				profileId: activeInstance.id,
				enableVulkan: activeInstance.useVulkan ?? false,
				skinUrl: account.value.skinUrl,
				skinVariant: account.value.skinVariant,
				capeUrl: account.value.capeUrl
			});
			appState.isGameRunning = true;
			appState.activeGameDetails = {
				name: activeInstance.name,
				version: activeInstance.mcVersion,
				loader: activeInstance.loader
			};
			toast(`Iniciando ${activeInstance.name}...`, "success");
		} catch (e) {
			toast("Erro ao iniciar jogo: " + String(e), "error");
		} finally {
			isLaunching = false;
		}
	}

	async function handleQuickJoin() {
		if (!activeInstance || !account.value) {
			toast("Selecione uma conta e instância", "warning");
			return;
		}

		isLaunching = true;
		playSound("launch");

		try {
			await launchGame({
				versionId: activeInstance.mcVersion,
				accountId: account.value.id,
				profileId: activeInstance.id,
				enableVulkan: activeInstance.useVulkan ?? false,
				skinUrl: account.value.skinUrl,
				skinVariant: account.value.skinVariant,
				capeUrl: account.value.capeUrl,
				serverIp: "jogar.mush.com.br",
				serverPort: 25565
			});
			toast(`Conectando diretamente ao MushMC...`, "success");
		} catch (e) {
			toast("Erro ao conectar ao servidor: " + String(e), "error");
		} finally {
			isLaunching = false;
		}
	}
</script>

{#if layoutStore.isCompactMode}
	<div 
		class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-4 bg-[#141518] border border-white/15 px-5 py-3 rounded-full shadow-2xl shadow-black/80 select-none animate-in fade-in slide-in-from-bottom-4 duration-300"
		transition:fade={{ duration: 150 }}
	>
		<div class="flex items-center gap-3 pr-2 border-r border-white/10">
			<div class="w-9 h-9 rounded-full bg-black/60 border border-white/10 overflow-hidden flex items-center justify-center shrink-0">
				{#if account.value?.skinUrl}
					<img src={account.value.skinUrl} alt="Skin" class="w-full h-full object-cover scale-150" />
				{:else}
					<ShieldCheck class="w-5 h-5 text-brand-500" />
				{/if}
			</div>
			<div class="flex flex-col">
				<span class="text-xs font-black text-white truncate max-w-[140px]">
					{activeInstance?.name ?? "Nenhuma instância"}
				</span>
				<span class="text-[10px] text-white/40 font-mono">
					{activeInstance ? `${activeInstance.mcVersion} · ${activeInstance.loader}` : "Modo Compacto"}
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2">
			<button 
				type="button"
				class="bg-brand-500 hover:bg-brand-400 text-black font-black text-xs px-5 py-2 rounded-full transition-all flex items-center gap-2 shadow-lg shadow-brand-500/20 cursor-pointer disabled:opacity-50"
				disabled={isLaunching || !activeInstance}
				onclick={handlePlay}
			>
				{#if isLaunching}
					<Loader2 class="w-3.5 h-3.5 animate-spin" />
					<span>Iniciando...</span>
				{:else}
					<Play class="w-3.5 h-3.5 fill-current" />
					<span>Jogar</span>
				{/if}
			</button>

			<button 
				type="button"
				class="bg-white/5 hover:bg-white/10 text-white/80 hover:text-white p-2 rounded-full transition-all cursor-pointer border border-white/5"
				title="Conectar ao MushMC"
				disabled={isLaunching || !activeInstance}
				onclick={handleQuickJoin}
			>
				<Server class="w-4 h-4 text-emerald-400" />
			</button>

			<button 
				type="button"
				class="bg-white/5 hover:bg-white/10 text-white/80 hover:text-white p-2 rounded-full transition-all cursor-pointer border border-white/5"
				title="Restaurar visualização completa"
				onclick={() => { layoutStore.isCompactMode = false; playSound("click"); }}
			>
				<Maximize2 class="w-4 h-4" />
			</button>
		</div>
	</div>
{/if}
