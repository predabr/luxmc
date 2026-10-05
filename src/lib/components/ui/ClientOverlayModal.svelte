<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { fade, scale } from "svelte/transition";
	import {
		X,
		Home,
		PenTool,
		Settings,
		Info,
		Search,
		Shield,
		Activity,
		Crosshair,
		Compass,
		Zap,
		Keyboard,
		Cpu,
		Image,
		Wifi,
		Clock,
		FlaskConical,
		Gauge,
		ListOrdered,
		Globe,
		Sun,
		Bomb,
		Waves,
		MapPin,
		CloudRain,
		ZoomIn,
		Layers,
		Eye,
		Users,
		MessageSquare,
		Sparkles,
		Heart,
		Terminal,
		Smile,
		Wand2,
		Palette,
		Sliders,
		Check,
		RotateCcw,
		LayoutTemplate
	} from "lucide-svelte";
	import { clientMods, type ClientModsConfig } from "$lib/stores/clientMods.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";
	import { clientOverlayClose } from "$lib/api";
	import ClientModuleConfigModal from "./ClientModuleConfigModal.svelte";
	import ClientHudPreview from "./ClientHudPreview.svelte";

	let searchQuery = $state("");
	let activeSection = $state<"all" | "hud" | "visual" | "combat" | "utility">("all");

	type ModuleDef = {
		key: keyof ClientModsConfig;
		name: string;
		icon: typeof import("lucide-svelte").Circle;
		category: "hud" | "visual" | "combat" | "utility";
		desc?: string;
	};

	const ALL_MODULES: ModuleDef[] = $derived([
		{ key: "armorHud", name: "Armor HUD", icon: Shield, category: "hud", desc: uiText("ui.e8ccd4e1a3359852") },
		{ key: "bossbar", name: "Bossbar", icon: Activity, category: "hud", desc: uiText("ui.5bce54f3ad57cbfd") },
		{ key: "comboDisplay", name: "Combo Display", icon: Zap, category: "combat", desc: uiText("ui.8682b53c7553faa9") },
		{ key: "coordinates", name: "Coordinates", icon: Compass, category: "hud", desc: uiText("ui.f0eb24afb81a80c3") },
		{ key: "cps", name: "CPS", icon: Activity, category: "hud", desc: uiText("ui.ff79cdf6663b96b2") },
		{ key: "directionHud", name: "Direction Hud", icon: Compass, category: "hud", desc: uiText("ui.756931d3d9ed62cc") },
		{ key: "fps", name: "FPS", icon: Gauge, category: "hud", desc: uiText("ui.21ee3811cea84acf") },
		{ key: "keystrokes", name: "Keystrokes", icon: Keyboard, category: "hud", desc: uiText("ui.b299a1ac6ff814c9") },
		{ key: "memory", name: "Memory", icon: Cpu, category: "hud", desc: uiText("ui.787c30cb364fe169") },
		{ key: "packOverlay", name: "Pack Overlay", icon: Image, category: "visual", desc: uiText("ui.a629e3f3c4ba74ea") },
		{ key: "pingDisplay", name: "Ping Display", icon: Wifi, category: "hud", desc: uiText("ui.d54f68832ddfd0de") },
		{ key: "playTime", name: "Play Time", icon: Clock, category: "hud", desc: uiText("ui.331d3b2c6705c547") },
		{ key: "potionCounter", name: "Potion Counter", icon: FlaskConical, category: "combat", desc: uiText("ui.0734cc47375acaaf") },
		{ key: "potionEffects", name: "Potion Effects", icon: FlaskConical, category: "hud", desc: uiText("ui.50c7db314324ab03") },
		{ key: "reachDisplay", name: "Reach Display", icon: Activity, category: "combat", desc: uiText("ui.f9b6139367d50159") },
		{ key: "scoreboard", name: "Scoreboard", icon: ListOrdered, category: "hud", desc: uiText("ui.5ba78ec6ab21259f") },
		{ key: "serverAddress", name: "Server Address", icon: Globe, category: "hud", desc: uiText("ui.3433dc82c2042a0e") },
		{ key: "speedometer", name: "Speedometer", icon: Gauge, category: "hud", desc: uiText("ui.5dabdaf4ab546d4f") },
		{ key: "timeDisplay", name: "Time Display", icon: Clock, category: "hud", desc: uiText("ui.58c747f43edbeb22") },
		{ key: "toggleSprint", name: "Toggle Sprint", icon: Zap, category: "combat", desc: uiText("ui.4e8d2b72cc1a4b65") },

		{ key: "twoDItems", name: "2D Items", icon: Image, category: "visual", desc: uiText("ui.96b5f78fabfab58c") },
		{ key: "threeDSkinLayers", name: "3D Skin Layers", icon: Layers, category: "visual", desc: uiText("ui.bb2bf6a169a58671") },
		{ key: "animations", name: "Animations", icon: Sparkles, category: "visual", desc: uiText("ui.cf4a7ada79941ba9") },
		{ key: "autoFriend", name: "Auto Friend", icon: Users, category: "utility", desc: uiText("ui.0132e01253796f5a") },
		{ key: "autoGG", name: "AutoGG", icon: MessageSquare, category: "utility", desc: uiText("ui.f208e2b8dc26f7c0") },
		{ key: "autoText", name: "AutoText", icon: MessageSquare, category: "utility", desc: uiText("ui.611e6c1c57cce43e") },
		{ key: "blockOverlay", name: "Block Overlay", icon: Image, category: "visual", desc: uiText("ui.82a605024c310332") },
		{ key: "chat", name: "Chat", icon: MessageSquare, category: "hud", desc: uiText("ui.28ae606466fdd054") },
		{ key: "cosmetics", name: "Cosmetics", icon: Wand2, category: "visual", desc: uiText("ui.ea612cd9825189c0") },
		{ key: "crosshair", name: "Crosshair", icon: Crosshair, category: "visual", desc: uiText("ui.8af6b37a20378cd5") },
		{ key: "damageTint", name: "Damage Tint", icon: Heart, category: "combat", desc: uiText("ui.0ba96e5681da502a") },
		{ key: "debugScreen", name: "Debug Screen", icon: Terminal, category: "hud", desc: uiText("ui.6d1c61d381261484") },
		{ key: "discordRP", name: "Discord RP", icon: MessageSquare, category: "utility", desc: uiText("ui.53274abcd8b49caf") },
		{ key: "emotes", name: "Emotes", icon: Smile, category: "visual", desc: uiText("ui.a4de1466acd2dbfc") },
		{ key: "fullbright", name: "Full Bright", icon: Sun, category: "visual", desc: uiText("ui.a2ff1120e6655028") },
		{ key: "glintColorizer", name: "Glint Colorizer", icon: Palette, category: "visual", desc: uiText("ui.8217abc2e2e17263") },
		{ key: "hitbox", name: "Hitbox", icon: Shield, category: "combat", desc: uiText("ui.48bc8874444411ea") },
		{ key: "hitColor", name: "Hit Color", icon: Palette, category: "combat", desc: uiText("ui.a4238d22f54408b8") },
		{ key: "inputFix", name: "InputFix", icon: Keyboard, category: "utility", desc: uiText("ui.b68f434fedcea986") },
		{ key: "itemPhysics", name: "Item Physics", icon: Sparkles, category: "visual", desc: uiText("ui.252570b7ddc5797d") },
		{ key: "motionBlur", name: "Motion Blur", icon: Eye, category: "visual", desc: uiText("ui.b9782c9854070520") },
		{ key: "nametags", name: "Nametags", icon: Users, category: "visual", desc: uiText("ui.075366d54a48cc7b") },
		{ key: "oldAnimations", name: "Old Animations", icon: Sparkles, category: "visual", desc: uiText("ui.93e06fd1c1d5ba61") },
		{ key: "particles", name: "Particles", icon: Sparkles, category: "visual", desc: uiText("ui.6884970ed053bdae") },
		{ key: "perspective", name: "Perspective", icon: Eye, category: "visual", desc: uiText("ui.dd9dd7e63077a81f") },
		{ key: "shinyPots", name: "Shiny Pots", icon: FlaskConical, category: "combat", desc: uiText("ui.19eba89ec72e47c2") },
		{ key: "skins", name: "Skins", icon: Layers, category: "visual", desc: uiText("ui.5309df3505d662b4") },
		{ key: "tab", name: "Tab", icon: ListOrdered, category: "hud", desc: uiText("ui.856d8319d3c91ed0") },
		{ key: "timeChanger", name: "Time Changer", icon: Clock, category: "visual", desc: uiText("ui.fb3715dddd84e8e5") },
		{ key: "tntTimer", name: "TNT Timer", icon: Bomb, category: "combat", desc: uiText("ui.0382ae6075ad8e0a") },
		{ key: "waveyCapes", name: "Wavey Capes", icon: Waves, category: "visual", desc: uiText("ui.fd4b64274351a9d6") },
		{ key: "waypoints", name: "Waypoints", icon: MapPin, category: "utility", desc: uiText("ui.53ef04d5e8b0d216") },
		{ key: "weatherChanger", name: "Weather Changer", icon: CloudRain, category: "visual", desc: uiText("ui.3905e67cd8860885") },
		{ key: "zoom", name: "Zoom", icon: ZoomIn, category: "visual", desc: uiText("ui.80dd761ca5868309") }
	]);

	const filteredModules = $derived.by(() => {
		let list = ALL_MODULES;
		if (activeSection !== "all") {
			list = list.filter((m) => m.category === activeSection);
		}
		if (!searchQuery.trim()) return list;
		const q = searchQuery.toLowerCase().trim();
		return list.filter((m) => m.name.toLowerCase().includes(q) || (m.desc && m.desc.toLowerCase().includes(q)));
	});

	async function handleClose() {
		clientMods.close();
	}

	function handleToggle(key: keyof ClientModsConfig) {
		clientMods.toggle(key);
		playSound("click");
	}

	function openModuleSettings(key: string | keyof ClientModsConfig) {
		clientMods.selectedModuleForConfig = String(key);
		playSound("click");
	}

	function handleReset() {
		clientMods.reset();
		toast(uiText("ui.45ab9100638178eb"), "info");
	}
</script>

{#if clientMods.isMenuOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-bg-overlay/75 backdrop-blur-md p-4 select-none"
		transition:fade={{ easing: quintOut, duration: 220 }}
		onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
		onkeydown={(e) => { if (e.key === "Escape") handleClose(); }}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div
			class="relative flex w-full max-w-5xl h-[680px] max-h-[92vh] rounded-3xl bg-bg/95 border border-fg/10 shadow-2xl shadow-black/90 overflow-hidden"
			transition:scale={{ easing: backOut, duration: 240, start: 0.96 }}
		>
			<!-- Left Navigation Bar -->
			<div class="flex flex-col items-center justify-between w-16 py-6 border-r border-fg/5 bg-bg">
				<div class="flex flex-col items-center gap-6">
					<button
						onclick={handleClose}
						class={launcherButton({ variant: "secondary", size: "icon", class: "flex items-center justify-center" })}
						aria-label={uiText("statusBanner.dismiss")}
						title={uiText("statusBanner.dismiss")}
					>
						<X class="w-5 h-5" />
					</button>

					<button
						onclick={() => { activeSection = "all"; }}
						class="w-10 h-10 rounded-2xl flex items-center justify-center transition-colors {activeSection === 'all' ? 'text-brand-400 bg-brand-500/10 border border-brand-500/30' : 'text-fg/50 hover:text-fg hover:bg-fg/10'}"
						title={uiText("ui.5e3aa152501dca60")}
					>
						<Home class="w-5 h-5" />
					</button>

					<button
						onclick={() => { activeSection = "hud"; }}
						class="w-10 h-10 rounded-2xl flex items-center justify-center transition-colors {activeSection === 'hud' ? 'text-brand-400 bg-brand-500/10 border border-brand-500/30' : 'text-fg/50 hover:text-fg hover:bg-fg/10'}"
						title={uiText("ui.66b842205d7c08ba")}
					>
						<PenTool class="w-5 h-5" />
					</button>

					<button
						onclick={() => { activeSection = "visual"; }}
						class="w-10 h-10 rounded-2xl flex items-center justify-center transition-colors {activeSection === 'visual' ? 'text-brand-400 bg-brand-500/10 border border-brand-500/30' : 'text-fg/50 hover:text-fg hover:bg-fg/10'}"
						title={uiText("ui.89deef1a60ab8cfb")}
					>
						<Eye class="w-5 h-5" />
					</button>

					<button
						onclick={() => { activeSection = "combat"; }}
						class="w-10 h-10 rounded-2xl flex items-center justify-center transition-colors {activeSection === 'combat' ? 'text-brand-400 bg-brand-500/10 border border-brand-500/30' : 'text-fg/50 hover:text-fg hover:bg-fg/10'}"
						title={uiText("ui.7dbe969d2def781b")}
					>
						<Shield class="w-5 h-5" />
					</button>
				</div>

				<div class="flex flex-col items-center gap-4">
					<button
						onclick={handleReset}
						class={launcherButton({ variant: "secondary", size: "icon", class: "flex items-center justify-center" })}
						title={uiText("ui.4549473edde7761e")}
					>
						<RotateCcw class="w-4 h-4" />
					</button>

					<div
						class="w-9 h-9 rounded-xl flex items-center justify-center text-fg/30"
						title={uiText("ui.f5eceed91a096e57")}
					>
						<Info class="w-4 h-4" />
					</div>
				</div>
			</div>

			<!-- Main Content -->
			<div class="flex flex-1 flex-col overflow-hidden">
				<!-- Header Bar -->
				<div class="flex items-center justify-between px-8 py-4 border-b border-fg/5 bg-bg-elevated">
					<div class="flex items-center gap-3">
						<span class="text-xs font-bold uppercase tracking-widest text-brand-400">{uiText("ui.b10636e6d873da24")}</span>
						<span class="text-fg/20">/</span>
						<span class="text-xs text-fg/60">
							{filteredModules.length} {filteredModules.length === 1 ? uiText("ui.8bcf94b370cdfed9") : uiText("ui.addce5b011fb4d85")}
						</span>
					</div>

					<div class="flex items-center gap-3">
						<button
							onclick={() => { clientMods.isPreviewingHud = true; }}
							class={launcherButton({ variant: "ghostBrand", size: "sm", class: "flex items-center gap-1.5" })}
							title={uiText("ui.544214d815001829")}
						>
							<LayoutTemplate class="w-3.5 h-3.5" />
							<span>{uiText("ui.847ef6db91a74a9a")}</span>
						</button>

						<div class="relative w-64">
							<Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-fg/40" />
							<input
								type="text"
								bind:value={searchQuery}
								placeholder={uiText("ui.6f86b3a5bfc8ca04")}
								class="w-full pl-10 pr-4 py-1.5 rounded-2xl bg-fg/5 border border-fg/10 text-sm text-fg placeholder-fg/40 focus:outline-none focus:border-brand-400/50 transition-colors"
							/>
							{#if searchQuery}
								<button
									onclick={() => searchQuery = ""}
									class={launcherButton({ variant: "ghost", size: "icon", class: "absolute right-3 top-1/2 -translate-y-1/2" })}
								>
									<X class="w-3.5 h-3.5" />
								</button>
							{/if}
						</div>
					</div>
				</div>

				<!-- Modules Grid -->
				<div class="flex-1 overflow-y-auto custom-scrollbar p-8">
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3.5">
						{#each filteredModules as mod (mod.key)}
							{@const isActive = !!clientMods.config[mod.key]}
							<div
								class="group relative flex items-center justify-between px-4 py-3 rounded-2xl border transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 cursor-pointer select-none {isActive ? 'border-fg/70 bg-fg/[0.07] shadow-lg shadow-black/40' : 'border-fg/10 bg-bg-elevated/60 hover:border-fg/25 hover:bg-fg/[0.03]'}"
								onclick={() => handleToggle(mod.key)}
								role="button"
								tabindex="0"
								onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") handleToggle(mod.key); }}
							>
								<!-- Left: Icon + Separator + Label -->
								<div class="flex items-center gap-3 min-w-0">
									<div class="flex items-center justify-center w-6 h-6 shrink-0 {isActive ? 'text-fg' : 'text-fg/50 group-hover:text-fg/80'}">
										<mod.icon class="w-5 h-5" strokeWidth={1.8} />
									</div>

									<div class="h-4 w-px bg-fg/15 shrink-0"></div>

									<span class="text-sm font-medium truncate {isActive ? 'text-fg font-semibold' : 'text-fg/70 group-hover:text-fg'}">
										{mod.name}
									</span>
								</div>

								<!-- Right: Settings Cog -->
								<div class="flex items-center gap-1.5 shrink-0 pl-2">
									<button
										type="button"
										onclick={(e) => {
											e.stopPropagation();
											openModuleSettings(mod.key);
										}}
										class={launcherButton({ variant: "secondary", size: "icon", class: "flex items-center justify-center" })}
										title="{uiText("ui.27864af39a282ed5")} {mod.name}"
									>
										<Settings class="w-3.5 h-3.5" />
									</button>
								</div>
							</div>
						{/each}
					</div>

					{#if filteredModules.length === 0}
						<div class="flex flex-col items-center justify-center h-64 text-center">
							<Search class="w-10 h-10 text-fg/20 mb-3" />
							<p class="text-fg/60 font-medium">{uiText("ui.59b7f0eb9dfbfa75")}</p>
							<p class="text-xs text-fg/40 mt-1">{uiText("ui.1937256d5943f156")}</p>
						</div>
					{/if}
				</div>

				<!-- Bottom Status Bar -->
				<div class="flex items-center justify-between px-8 py-3.5 border-t border-fg/5 bg-bg text-xs text-fg/40">
					<div class="flex items-center gap-2">
						<span class="w-2 h-2 rounded-full bg-brand-500 animate-pulse"></span>
						<span>{uiText("ui.265d068dce7aae8f")}</span>
					</div>

					<div class="flex items-center gap-4">
						<span>{uiText("ui.9bffce84d032c31e")}</span>
					</div>

				</div>
			</div>
		</div>
	</div>
{/if}

{#if clientMods.selectedModuleForConfig}
	<ClientModuleConfigModal
		moduleKey={clientMods.selectedModuleForConfig}
		onClose={() => { clientMods.selectedModuleForConfig = null; }}
	/>
{/if}

{#if clientMods.isPreviewingHud}
	<ClientHudPreview onClose={() => { clientMods.isPreviewingHud = false; }} />
{/if}
