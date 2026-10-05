<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { quickProfiles } from "$lib/utils/quickProfiles";
	import EnvironmentDiagnostics from "$lib/components/ui/EnvironmentDiagnostics.svelte";
	import { instancesOpenFolder } from "$lib/api/instances";
	import { toast } from "$lib/stores/toasts.svelte";
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
	let diagnosticsOpen = $state(false);

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

		const openDiagnostics = () => { open = false; diagnosticsOpen = true; };
		window.addEventListener("luxmc-open-diagnostics", openDiagnostics);
		window.addEventListener("keydown", handleKeydown);
		return () => { window.removeEventListener("keydown", handleKeydown); window.removeEventListener("luxmc-open-diagnostics", openDiagnostics); };
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

	const filteredProfiles = $derived(quickProfiles(profiles.list, query));
    async function openInstanceFolder(id: string) {
        open = false;
        try { await instancesOpenFolder(id); }
        catch (error) { toast(String(error), "error"); }
    }

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
			aria-label={uiText("ui.5b63975b7d828550")}
		>
			<Command.Dialog {open} on:close={() => open = false}>
				<div class="flex items-center gap-3 px-4 border-b border-fg/10">
					<Search class="w-4 h-4 text-fg/40 shrink-0" />
					<Command.Input 
						bind:value={query}
						placeholder={uiText("ui.264777c968f2dba3")} 
						class="w-full py-4 bg-transparent text-sm text-fg placeholder:text-fg/40 outline-none border-none font-medium"
					/>
					<div class="flex items-center gap-1.5 shrink-0">
						<kbd class="text-[10px] text-fg/25 bg-fg/5 border border-fg/10 px-1.5 py-0.5 rounded-md font-mono">{uiText("ui.c5b7d0ec4f697629")}</kbd>
						<button 
							type="button" 
							class={launcherButton({ variant: "ghost", size: "icon", class: "" })}
							onclick={() => open = false}
						>
							<X class="w-4 h-4" />
						</button>
					</div>
				</div>

				<Command.List class="max-h-[360px] overflow-y-auto p-2 custom-scrollbar space-y-1">
					<Command.Empty class="py-8 text-center text-xs text-fg/40">
						{uiText("ui.c03e4e4fbca406c2")}<span class="text-fg/60">{query}</span>"
					</Command.Empty>

					<Command.Group heading="Navegação">
						<Command.Item value="inicio home jogar play" onSelect={() => navigate("/")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Play class="w-4 h-4 text-brand-500" />
							<span>{uiText("nav.home")}</span>
						</Command.Item>
						<Command.Item value="instancias instances gerenciar" onSelect={() => navigate("/instances")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Layers class="w-4 h-4 text-emerald-400" />
							<span>{uiText("ui.a659d3e33d649113")}</span>
						</Command.Item>
						<Command.Item value="mods modpacks central conteudo" onSelect={() => navigate("/mods")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Package class="w-4 h-4 text-purple-400" />
							<span>{uiText("ui.8c9059dd7c2e7a90")}</span>
						</Command.Item>
						<Command.Item value="skins capas skin capa personalizacao" onSelect={() => navigate("/skins")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Shirt class="w-4 h-4 text-pink-400" />
							<span>{uiText("ui.786fa4f02a1a0dfc")}</span>
						</Command.Item>
						<Command.Item value="amigos friends p2p rede" onSelect={() => navigate("/friends")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Users class="w-4 h-4 text-blue-400" />
							<span>{uiText("nav.friends")}</span>
						</Command.Item>
						<Command.Item value="screenshots capturas fotos" onSelect={() => navigate("/screenshots")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Camera class="w-4 h-4 text-yellow-400" />
							<span>{uiText("ui.067348ce68943b63")}</span>
						</Command.Item>
						<Command.Item value="noticias news blog" onSelect={() => navigate("/news")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Newspaper class="w-4 h-4 text-amber-400" />
							<span>{uiText("home.news")}</span>
						</Command.Item>
						<Command.Item value="logs registros historico erros" onSelect={() => navigate("/logs")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<FileText class="w-4 h-4 text-orange-400" />
							<span>{uiText("settings.catLogs")}</span>
						</Command.Item>
						<Command.Item value="configuracoes settings launcher tema" onSelect={() => navigate("/settings")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Settings class="w-4 h-4 text-fg/60" />
							<span>{uiText("ui.76b0fb6ad18939ac")}</span>
						</Command.Item>
					</Command.Group>

					{#if filteredProfiles.length > 0}
						<Command.Group heading="Instâncias">
							{#each filteredProfiles as p}
								<Command.Item value={`instancia ${p.name} ${p.name.normalize("NFD").replace(/\p{Diacritic}/gu, "")} ${p.mcVersion} ${p.loader}`} onSelect={() => { profiles.activeId = p.id; navigate(`/instances/${p.id}`); }} class="flex items-center justify-between px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
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
                        <Command.Item value="diagnostico sistema windows ambiente java gpu memoria verificar" onSelect={() => { open = false; diagnosticsOpen = true; }} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:bg-fg/10 cursor-pointer aria-selected:bg-fg/10"><Settings class="w-4 h-4 text-brand-400" /><span>{uiText("ui.d5986b954d27350d")}</span></Command.Item>
                        {#if profiles.activeId}
                            <Command.Item value="pasta explorador arquivos instancia ativa" onSelect={() => void openInstanceFolder(profiles.activeId!)} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:bg-fg/10 cursor-pointer aria-selected:bg-fg/10"><Layers class="w-4 h-4 text-brand-400" /><span>{uiText("ui.680b9210ff6e5c28")}</span></Command.Item>
                        {/if}

						<Command.Item value="java runtime jvm instalar gerenciar" onSelect={openJavaManager} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Coffee class="w-4 h-4 text-orange-400" />
							<span>{uiText("ui.ed7ad2f20f94a37f")}</span>
						</Command.Item>
						<Command.Item value="tema cor aparencia visual" onSelect={() => navigate("/settings")} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							<Palette class="w-4 h-4 text-violet-400" />
							<span>{uiText("ui.8c429219edef8f25")}</span>
						</Command.Item>
						<Command.Item value="som audio silenciar efeitos" onSelect={toggleSound} class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs text-fg/80 hover:text-fg hover:bg-fg/10 cursor-pointer transition-colors aria-selected:bg-fg/10">
							{#if (settings.value as { soundEnabled?: boolean })?.soundEnabled === false}
								<Volume2 class="w-4 h-4 text-emerald-400" />
								<span>{uiText("ui.0128db193f53bb30")}</span>
							{:else}
								<VolumeX class="w-4 h-4 text-red-400" />
								<span>{uiText("ui.0266cae02b862726")}</span>
							{/if}
						</Command.Item>
					</Command.Group>
				</Command.List>

				<div class="px-3 py-2 border-t border-fg/5 flex items-center justify-between">
					<div class="flex items-center gap-3 text-[10px] text-fg/25">
						<span><kbd class="font-mono">↑↓</kbd> {uiText("ui.e907f5d07d99bc56")}</span>
						<span><kbd class="font-mono">↵</kbd> {uiText("ui.1a07c7242e6cce5c")}</span>
						<span><kbd class="font-mono">{uiText("ui.52f878edb34fa14f")}</kbd> {uiText("ui.df1deb26dff33cc9")}</span>
					</div>
					<span class="text-[10px] text-fg/20 font-mono">{uiText("ui.199f6f88d1fcf30b")}</span>
				</div>
			</Command.Dialog>
		</div>
	</div>
{/if}

<JavaManagerModal open={javaModalOpen} onClose={() => (javaModalOpen = false)} />

<EnvironmentDiagnostics open={diagnosticsOpen} onClose={() => diagnosticsOpen = false} />
