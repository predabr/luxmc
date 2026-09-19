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
		X,
		Coffee,
		Palette,
		LayoutGrid,
		Newspaper,
		Server,
		Camera,
		Shirt,
		Play
	} from "lucide-svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { playSound } from "$lib/utils/sound";
	import JavaManagerModal from "$lib/components/ui/JavaManagerModal.svelte";
	import { appState } from "$lib/stores/app.svelte";

	let open = $state(false);
	let query = $state("");
	let javaModalOpen = $state(false);

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

	function openJavaManager() {
		open = false;
		javaModalOpen = true;
		playSound("click");
	}

	const filteredProfiles = $derived(
		query.trim()
			? profiles.list.filter((p) =>
				[p.name, p.mcVersion, p.loader].join(" ").toLowerCase().includes(query.toLowerCase())
			)
			: profiles.list.slice(0, 6)
	);
</script>

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-start justify-center pt-16 p-4 bg-bg-overlay/75 backdrop-blur-md"
		onclick={() => (open = false)}
		onkeydown={(e) => { if (e.key === "Escape") open = false; }}
		role="presentation"
	>
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="w-full max-w-xl bg-bg-elevated border border-fg/10 rounded-2xl shadow-2xl overflow-hidden animate-slide-up"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			tabindex="-1"
			aria-modal="true"
			aria-label="Paleta de comandos"
		>
			<Command.Dialog {open} on:close={() => open = false}>
				<div class="flex items-center gap-3 px-4 border-b border-fg/10">
					<Search class="w-4 h-4 text-fg/40 shrink-0" />
					<Command.Input 
						value={query}
						placeholder="Buscar instâncias, ações, páginas…" 
						class="w-full py-4 bg-transparent text-sm text-fg placeholder:text-fg/40 outline-none border-none font-medium"
					/>
					<div class="flex items-center gap-1.5 shrink-0">
						<kbd class="text-[10px] text-fg/25 bg-fg/5 border border-fg/10 px-1.5 py-0.5 rounded-md font-mono">ESC</kbd>
						<button 
							type="button" 
							class="text-fg/40 hover:text-fg p-1 rounded-lg cursor-pointer"
							onclick={() => open = false}
						>
							<X class="w-4 h-4" />
						</button>
					</div>
				</div>

				<Command.List class="max-h-[360px] overflow-y-auto p-2 custom-scrollbar space-y-1">
					<Command.Empty class="py-8 text-center text-xs text-fg/40">
						Nenhum resultado encontrado para "<span class="text-fg/60">{query}</span>"
					</Command.Empty>

					<Command.Group heading="Navegação">
						<Command.Item value="inicio home jogar play" on:select={() => navigate("/")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Play class="w-4 h-4 text-brand-500" />
							<span>Início</span>
						</Command.Item>
						<Command.Item value="instancias instances gerenciar" on:select={() => navigate("/instances")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Layers class="w-4 h-4 text-emerald-400" />
							<span>Gerenciar Instâncias</span>
						</Command.Item>
						<Command.Item value="mods modpacks central conteudo" on:select={() => navigate("/mods")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Package class="w-4 h-4 text-purple-400" />
							<span>Central de Conteúdo (Mods & Modpacks)</span>
						</Command.Item>
						<Command.Item value="skins capas skin capa personalizacao" on:select={() => navigate("/skins")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Shirt class="w-4 h-4 text-pink-400" />
							<span>Skins & Capas</span>
						</Command.Item>
						<Command.Item value="amigos friends p2p rede" on:select={() => navigate("/")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Users class="w-4 h-4 text-blue-400" />
							<span>Amigos</span>
						</Command.Item>
						<Command.Item value="screenshots capturas fotos" on:select={() => navigate("/screenshots")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Camera class="w-4 h-4 text-yellow-400" />
							<span>Screenshots</span>
						</Command.Item>
						<Command.Item value="noticias news blog" on:select={() => navigate("/news")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Newspaper class="w-4 h-4 text-amber-400" />
							<span>Notícias</span>
						</Command.Item>
						<Command.Item value="logs registros historico erros" on:select={() => navigate("/logs")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<FileText class="w-4 h-4 text-orange-400" />
							<span>Logs do Jogo</span>
						</Command.Item>
						<Command.Item value="configuracoes settings launcher tema" on:select={() => navigate("/settings")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Settings class="w-4 h-4 text-fg/60" />
							<span>Configurações</span>
						</Command.Item>
					</Command.Group>

					{#if filteredProfiles.length > 0}
						<Command.Group heading="Instâncias">
							{#each filteredProfiles as p}
								<Command.Item value={`instancia ${p.name} ${p.mcVersion} ${p.loader}`} on:select={() => { profiles.activeId = p.id; navigate(`/instances/${p.id}`); }} class="flex items-center justify-between px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
									<div class="flex items-center gap-3">
										<Sparkles class="w-4 h-4 text-brand-500" />
										<span class="font-bold">{p.name}</span>
									</div>
									<span class="text-[10px] text-fg/40 font-mono">{p.mcVersion} · {p.loader}</span>
								</Command.Item>
							{/each}
						</Command.Group>
					{/if}

					<Command.Group heading="Ferramentas">
						<Command.Item value="java runtime jvm instalar gerenciar" on:select={openJavaManager} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Coffee class="w-4 h-4 text-orange-400" />
							<span>Gerenciador de Java</span>
						</Command.Item>
						<Command.Item value="tema cor aparencia visual" on:select={() => navigate("/settings")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Palette class="w-4 h-4 text-violet-400" />
							<span>Tema & Aparência</span>
						</Command.Item>
						<Command.Item value="som audio silenciar efeitos" on:select={toggleSound} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
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

				<div class="px-3 py-2 border-t border-fg/5 flex items-center justify-between">
					<div class="flex items-center gap-3 text-[10px] text-fg/25">
						<span><kbd class="font-mono">↑↓</kbd> navegar</span>
						<span><kbd class="font-mono">↵</kbd> selecionar</span>
						<span><kbd class="font-mono">Esc</kbd> fechar</span>
					</div>
					<span class="text-[10px] text-fg/20 font-mono">Ctrl+K</span>
				</div>
			</Command.Dialog>
		</div>
	</div>
{/if}

<JavaManagerModal open={javaModalOpen} onClose={() => (javaModalOpen = false)} />
