<script lang="ts">
	import { Zap, Gamepad2 } from "lucide-svelte";
	import { discordSetActivity } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";

	type Props = {
		performanceMode: boolean;
		discordRpc: boolean;
		curseforgeApiKey: string;
		onPerformanceModeChange: (val: boolean) => void;
		onDiscordRpcChange: (val: boolean) => void;
		onCurseforgeApiKeyChange: (val: string) => void;
	};

	let {
		performanceMode,
		discordRpc,
		curseforgeApiKey,
		onPerformanceModeChange,
		onDiscordRpcChange,
		onCurseforgeApiKeyChange,
	}: Props = $props();

	async function toggleDiscordRpc() {
		const next = !discordRpc;
		onDiscordRpcChange(next);
		if (next) {
			const ok = await discordSetActivity({
				details: "Configurações do Launcher",
				state: "v1.3.1-alpha · Linux",
				largeText: "Luxmc Launcher (Linux)",
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: "grass",
				smallText: "Minecraft Linux"
			});
			if (ok) {
				toast("Discord Rich Presence conectado com sucesso!", "success");
			} else {
				toast("Discord RPC ativado! Certifique-se de que o Discord está aberto.", "info");
			}
		} else {
			toast("Discord Rich Presence desativado.", "info");
		}
	}
</script>

<div class="space-y-3">
	<!-- Performance Mode -->
	<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3 flex items-center justify-between hover:border-white/10 transition-all">
		<div class="flex items-center gap-3">
			<div class="w-9 h-9 rounded-xl bg-white/5 flex items-center justify-center text-orange-400">
				<Zap class="w-4 h-4" />
			</div>
			<div>
				<div class="text-xs font-bold text-white">Modo Desempenho</div>
				<div class="text-[10px] text-white/40">Remove efeitos visuais pesados para melhorar FPS</div>
			</div>
		</div>
		<button
			type="button"
			role="switch"
			aria-label="Modo desempenho"
			aria-checked={performanceMode}
			class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {performanceMode ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
			onclick={() => onPerformanceModeChange(!performanceMode)}
		>
			<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {performanceMode ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
		</button>
	</div>

	<!-- Discord RPC -->
	<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3 flex items-center justify-between hover:border-white/10 transition-all">
		<div class="flex items-center gap-3">
			<div class="w-9 h-9 rounded-xl bg-white/5 flex items-center justify-center text-indigo-400">
				<Gamepad2 class="w-4 h-4" />
			</div>
			<div>
				<div class="text-xs font-bold text-white">Discord Rich Presence</div>
				<div class="text-[10px] text-white/40">Mostra seu status no Discord enquanto joga</div>
			</div>
		</div>
		<button
			type="button"
			role="switch"
			aria-label="Discord Rich Presence"
			aria-checked={discordRpc}
			class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {discordRpc ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
			onclick={toggleDiscordRpc}
		>
			<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {discordRpc ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
		</button>
	</div>

	<!-- CurseForge API Key -->
	<div>
		<label for="curseforge-api" class="text-xs font-bold text-white block mb-1.5">Chave API do CurseForge (opcional)</label>
		<input
			id="curseforge-api"
			type="text"
			placeholder="$2a$10$..."
			value={curseforgeApiKey}
			onchange={(e) => onCurseforgeApiKeyChange((e.target as HTMLInputElement).value)}
			class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white font-mono focus:border-brand-500 focus:outline-none placeholder:text-white/30"
		/>
		<p class="text-[10px] text-white/40 mt-1">Necessário para baixar mods premium do CurseForge</p>
	</div>
</div>
