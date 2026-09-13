<script lang="ts">
	import { Command } from "cmdk-svelte";
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { 
		Search, 
		Gamepad2, 
		Layers, 
		Package, 
		Settings, 
		FileText, 
		Users, 
		Sparkles, 
		Volume2, 
		VolumeX,
		X
	} from "lucide-svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { playSound } from "$lib/utils/sound";

	let open = $state(false);
	let query = $state("");

	onMount(() => {
		function handleKeydown(e: KeyboardEvent) {
			if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
				e.preventDefault();
				open = !open;
				if (open) {
					query = "";
					playSound("click");
				}
			}
			if (e.key === "Escape" && open) {
				open = false;
			}
		}

		window.addEventListener("keydown", handleKeydown);
		return () => window.removeEventListener("keydown", handleKeydown);
	});

	function navigate(url: string) {
		open = false;
		playSound("click");
		goto(url);
	}

	function toggleSound() {
		const s = settings.value as { soundEnabled?: boolean };
		const current = s?.soundEnabled !== false;
		settings.patch({ soundEnabled: !current });
		playSound("click");
		open = false;
	}
</script>

{#if open}
	<div class="fixed inset-0 z-50 flex items-start justify-center pt-20 p-4 bg-black/75 backdrop-blur-md">
		<div class="w-full max-w-xl bg-[#141518] border border-white/10 rounded-2xl shadow-2xl overflow-hidden">
			<Command.Dialog {open} on:close={() => open = false}>
				<div class="flex items-center gap-3 px-4 border-b border-white/10">
					<Search class="w-4 h-4 text-white/40" />
					<Command.Input 
						value={query}
						placeholder="Digite um comando ou busque uma instância..." 
						class="w-full py-4 bg-transparent text-sm text-white placeholder:text-white/40 outline-none border-none font-medium"
					/>
					<button 
						type="button" 
						class="text-white/40 hover:text-white p-1 rounded-lg cursor-pointer"
						onclick={() => open = false}
					>
						<X class="w-4 h-4" />
					</button>
				</div>

				<Command.List class="max-h-80 overflow-y-auto p-2 custom-scrollbar space-y-1">
					<Command.Empty class="py-6 text-center text-xs text-white/40">
						Nenhum resultado encontrado.
					</Command.Empty>

					<Command.Group heading="Navegação">
						<Command.Item value="inicio home jogar" on:select={() => navigate("/")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-white/80 hover:text-white hover:bg-white/10 cursor-pointer transition-colors">
							<Gamepad2 class="w-4 h-4 text-brand-500" />
							<span>Início (Jogar)</span>
						</Command.Item>
						<Command.Item value="instancias instances gerenciar" on:select={() => navigate("/instances")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-white/80 hover:text-white hover:bg-white/10 cursor-pointer transition-colors">
							<Layers class="w-4 h-4 text-emerald-400" />
							<span>Gerenciar Instâncias</span>
						</Command.Item>
						<Command.Item value="mods modpacks central conteudo" on:select={() => navigate("/mods")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-white/80 hover:text-white hover:bg-white/10 cursor-pointer transition-colors">
							<Package class="w-4 h-4 text-purple-400" />
							<span>Central de Conteúdo (Mods & Modpacks)</span>
						</Command.Item>
						<Command.Item value="amigos friends p2p rede" on:select={() => navigate("/friends")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-white/80 hover:text-white hover:bg-white/10 cursor-pointer transition-colors">
							<Users class="w-4 h-4 text-blue-400" />
							<span>Amigos & P2P</span>
						</Command.Item>
						<Command.Item value="logs registros historico" on:select={() => navigate("/logs")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-white/80 hover:text-white hover:bg-white/10 cursor-pointer transition-colors">
							<FileText class="w-4 h-4 text-amber-400" />
							<span>Logs do Jogo</span>
						</Command.Item>
						<Command.Item value="configuracoes settings launcher" on:select={() => navigate("/settings")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-white/80 hover:text-white hover:bg-white/10 cursor-pointer transition-colors">
							<Settings class="w-4 h-4 text-white/60" />
							<span>Configurações do Launcher</span>
						</Command.Item>
					</Command.Group>

					<Command.Group heading="Instâncias">
						{#each profiles.list as p}
							<Command.Item value={`instancia ${p.name} ${p.mcVersion} ${p.loader}`} on:select={() => { profiles.activeId = p.id; navigate(`/instances/${p.id}`); }} class="flex items-center justify-between px-3 py-2.5 rounded-xl text-xs text-white/80 hover:text-white hover:bg-white/10 cursor-pointer transition-colors">
								<div class="flex items-center gap-3">
									<Sparkles class="w-4 h-4 text-brand-500" />
									<span class="font-bold">{p.name}</span>
								</div>
								<span class="text-[10px] text-white/40 font-mono">{p.mcVersion} · {p.loader}</span>
							</Command.Item>
						{/each}
					</Command.Group>

					<Command.Group heading="Ações Rápidas">
						<Command.Item value="som audio silenciar efeitos" on:select={toggleSound} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-white/80 hover:text-white hover:bg-white/10 cursor-pointer transition-colors">
							{#if (settings.value as { soundEnabled?: boolean })?.soundEnabled === false}
								<Volume2 class="w-4 h-4 text-emerald-400" />
								<span>Ativar Efeitos Sonoros</span>
							{:else}
								<VolumeX class="w-4 h-4 text-red-400" />
								<span>Silenciar Efeitos Sonoros</span>
							{/if}
						</Command.Item>
					</Command.Group>
				</Command.List>
			</Command.Dialog>
		</div>
	</div>
{/if}
