<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { quintOut } from "svelte/easing";
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
		if (isLaunching || appState.isLaunching) return;
		if (!activeInstance || !account.value) {
			toast(uiText("ui.4cf884f78e1be613"), "warning");
			return;
		}

		isLaunching = true;
		appState.isLaunching = true;
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
			toast(uiText("ui.61910b3c799dce6f") + String(e), "error");
		} finally {
			isLaunching = false;
			appState.isLaunching = false;
		}
	}

	async function handleQuickJoin() {
		if (isLaunching || appState.isLaunching) return;
		if (!activeInstance || !account.value) {
			toast(uiText("ui.f79c6bf8346b292f"), "warning");
			return;
		}

		isLaunching = true;
		appState.isLaunching = true;
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
			toast(uiText("ui.9ef3feaaface462b") + String(e), "error");
		} finally {
			isLaunching = false;
			appState.isLaunching = false;
		}
	}
</script>

{#if layoutStore.isCompactMode}
	<div 
		class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-4 bg-bg-elevated border border-fg/15 px-5 py-3 rounded-full shadow-2xl shadow-black/80 select-none animate-in fade-in slide-in-from-bottom-4 duration-300"
		transition:fade={{ easing: quintOut, duration: 220 }}
	>
		<div class="flex items-center gap-3 pr-2 border-r border-fg/10">
			<div class="w-9 h-9 rounded-full bg-bg-overlay/60 border border-fg/10 overflow-hidden flex items-center justify-center shrink-0">
				{#if account.value?.skinUrl}
					<img loading="lazy" decoding="async" src={account.value.skinUrl} alt={uiText("ui.1fc91201eedbfcb7")} class="w-full h-full object-cover scale-150" />
				{:else}
					<ShieldCheck class="w-5 h-5 text-brand-500" />
				{/if}
			</div>
			<div class="flex flex-col">
				<span class="text-xs font-black text-fg truncate max-w-[140px]">
					{activeInstance?.name ?? uiText("ui.928243013c6d235d")}
				</span>
				<span class="text-[10px] text-fg/40 font-mono">
					{activeInstance ? `${activeInstance.mcVersion} · ${activeInstance.loader}` : "Modo Compacto"}
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2">
			<button 
				type="button"
				class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-2 disabled:opacity-50" })}
				disabled={isLaunching || !activeInstance}
				onclick={handlePlay}
			>
				{#if isLaunching}
					<Loader2 class="w-3.5 h-3.5 animate-spin" />
					<span>{uiText("home.starting")}</span>
				{:else}
					<Play class="w-3.5 h-3.5 fill-current" />
					<span>{uiText("instances.play")}</span>
				{/if}
			</button>

			<button 
				type="button"
				class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
				title={uiText("ui.73f523998b1903ee")}
				disabled={isLaunching || !activeInstance}
				onclick={handleQuickJoin}
			>
				<Server class="w-4 h-4 text-emerald-400" />
			</button>

			<button 
				type="button"
				class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
				title={uiText("ui.0e53c33c6c67c5fc")}
				onclick={() => { layoutStore.isCompactMode = false; playSound("click"); }}
			>
				<Maximize2 class="w-4 h-4" />
			</button>
		</div>
	</div>
{/if}
