<script lang="ts">
    import { toast } from "$lib/stores/toasts.svelte";
    import { openPortal } from "$lib/api/deepLinks";
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import {
		Play,
		Compass,
		Shirt,
		Layers,
		Plus,
		Settings as SettingsIcon,
		Github,
		Globe
	} from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { appState } from "$lib/stores/app.svelte";

	let { notificationCount = 0 }: { notificationCount?: number } = $props();
	const { t } = useTranslation();

	type Item = { href: string; labelKey: string; icon: typeof import("lucide-svelte").Circle; title: string };

	const items: Item[] = [
		{ href: "/mods", labelKey: "nav.mods", title: "Explorar Mods", icon: Compass },
		{ href: "/skins", labelKey: "nav.skins", title: "Skins & Capas", icon: Shirt },
		{ href: "/instances", labelKey: "nav.instances", title: "Biblioteca", icon: Layers },
	];

	const isHomeActive = $derived(page.url.pathname === "/");
	const settingsActive = $derived(page.url.pathname.startsWith("/settings"));

	function getAccountStatus(acc: typeof account.value) {
		if (!acc) return { type: "none", label: "Desconectado", dotColor: "bg-zinc-500" };
		if (!acc.minecraftToken || acc.id.startsWith("offline_") || acc.id.startsWith("offline-")) {
			return { type: "offline", label: "Conta Offline", dotColor: "bg-sky-400" };
		}
		return { type: "online", label: "Microsoft Online", dotColor: "bg-[#1bd96a]" };
	}

	const accountStatus = $derived(getAccountStatus(account.value));

	const iconPalette = ["bg-blue-600", "bg-purple-600", "bg-violet-600", "bg-emerald-600", "bg-amber-600", "bg-rose-600", "bg-cyan-600"];
	function getDeterministicColor(str: string): string {
		let hash = 0;
		for (let i = 0; i < str.length; i++) hash = str.charCodeAt(i) + ((hash << 5) - hash);
		return iconPalette[Math.abs(hash) % iconPalette.length];
	}
</script>

<aside class="flex h-full min-h-screen w-[66px] shrink-0 flex-col items-center py-3 bg-[#0d0f12]/60 backdrop-blur-2xl border-r border-white/[0.06] z-40 relative select-none">
	
	<div class="relative group w-full flex justify-center mb-3">
		<button
			type="button"
			onclick={() => goto("/")}
			class="relative h-11 w-11 rounded-2xl flex items-center justify-center transition-all duration-200 cursor-pointer {isHomeActive ? 'bg-[#1bd96a] text-[#090a0f] shadow-lg shadow-[#1bd96a]/25 scale-[1.04]' : 'bg-white/[0.04] text-white/60 hover:text-white hover:bg-white/[0.08] active:scale-[0.98]'}"
			title="Início"
		>
			<Play class="h-5 w-5 fill-current ml-0.5" />
		</button>
		<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#161920] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl">
			{t("nav.home") || "Início"}
		</div>
	</div>

	<nav class="flex-1 w-full flex flex-col items-center gap-2">
		{#each items as item}
			{@const active = page.url.pathname.startsWith(item.href)}
			<div class="relative group w-full flex justify-center">
				{#if active}
					<span class="absolute left-0.5 top-1/2 -translate-y-1/2 w-1 h-6 bg-[#1bd96a] rounded-r-full shadow-glow"></span>
				{/if}
				<a
					href={item.href}
					onclick={(e) => {
						e.preventDefault();
						goto(item.href);
					}}
					class="relative h-11 w-11 rounded-2xl flex items-center justify-center transition-all duration-200 cursor-pointer {active ? 'bg-[#1bd96a]/15 text-[#1bd96a] border border-[#1bd96a]/30 scale-[1.02]' : 'bg-white/[0.02] border border-transparent text-white/50 hover:text-white hover:bg-white/[0.06] active:scale-[0.98]'}"
				>
					<item.icon class="h-5 w-5" strokeWidth={active ? 2.2 : 1.8} />
				</a>

				<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#161920] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl">
					{t(item.labelKey) || item.title}
				</div>
			</div>
		{/each}
		
		<div class="w-7 h-[1px] bg-white/10 my-1"></div>
		
		{#if profiles.list.length > 0}
			<div class="w-full flex flex-col items-center gap-2 overflow-y-auto overflow-x-hidden max-h-[30vh] custom-scrollbar px-1 py-0.5">
				{#each profiles.list as prof}
					{@const active = page.url.pathname === `/instances/${prof.id}` || (page.url.pathname === "/instances" && profiles.activeId === prof.id)}
					{@const bgColor = getDeterministicColor(prof.id || prof.name)}
					<div class="relative group w-full flex justify-center">
						{#if active}
							<span class="absolute left-0.5 top-1/2 -translate-y-1/2 w-1 h-5 bg-[#1bd96a] rounded-r-full shadow-glow"></span>
						{/if}
						<a
							href={`/instances/${prof.id}`}
							onclick={(e) => {
								e.preventDefault();
								goto(`/instances/${prof.id}`);
							}}
							class="relative h-10 w-10 rounded-xl overflow-hidden flex items-center justify-center transition-all duration-200 active:scale-[0.98] border cursor-pointer {active ? 'ring-2 ring-[#1bd96a] border-transparent shadow-lg scale-105' : 'border-white/10 bg-white/[0.03] hover:bg-white/[0.08] hover:border-white/30'}"
						>
							{#if prof.icon && (prof.icon.startsWith("http") || prof.icon.startsWith("/") || prof.icon.startsWith("data:"))}
								<img src={prof.icon} alt={prof.name} class="w-full h-full object-cover" />
							{:else}
								<div class="w-full h-full {bgColor} flex items-center justify-center">
									<img src="/grass_block.png" alt={prof.name} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
								</div>
							{/if}
						</a>

						<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#161920] text-white text-xs font-bold px-3 py-2 rounded-xl border border-white/10 shadow-2xl flex flex-col gap-0.5">
							<span class="text-white font-extrabold">{prof.name}</span>
							<span class="text-[10px] text-white/50">{prof.mcVersion} • {prof.loader}</span>
						</div>
					</div>
				{/each}
			</div>
			<div class="w-7 h-[1px] bg-white/10 my-0.5"></div>
		{/if}

		<div class="relative group w-full flex justify-center mt-1">
			<a
				href="/instances?new=true"
				onclick={(e) => {
					e.preventDefault();
					goto("/instances?new=true");
				}}
				class="h-10 w-10 rounded-xl bg-white/[0.03] border border-white/10 hover:border-[#1bd96a]/50 hover:bg-[#1bd96a]/10 hover:text-[#1bd96a] flex items-center justify-center text-white/50 transition-all duration-200 active:scale-[0.98] shadow-sm cursor-pointer"
				title="Nova Instância"
			>
				<Plus class="h-4 w-4" />
			</a>
			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#161920] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl">
				{t("home.createInstance") || "Nova Instância"}
			</div>
		</div>
	</nav>

	<div class="mt-auto w-full flex flex-col items-center gap-2 pt-2 border-t border-white/10">
		<div class="relative group w-full flex justify-center">
			<button
				type="button"
				onclick={() => { void openPortal().catch(error => toast(String(error), "error")); }}
				class="relative h-10 w-10 rounded-xl flex items-center justify-center transition-all duration-200 cursor-pointer bg-white/[0.02] text-white/50 hover:text-[#1bd96a] hover:bg-white/[0.06] active:scale-[0.98]"
				aria-label="Portal Web & Studio 3D"
			>
				<Globe class="h-4 w-4" strokeWidth={1.8} />
			</button>
			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#161920] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl flex items-center gap-1.5">
				<span>Portal Web & Studio 3D</span>
				<span class="text-[10px] text-[#1bd96a] font-mono">pages.dev</span>
			</div>
		</div>

		<div class="relative group w-full flex justify-center">
			<button
				type="button"
				onclick={() => openUrl("https://github.com/predabr/luxmc")}
				class="relative h-10 w-10 rounded-xl flex items-center justify-center transition-all duration-200 cursor-pointer bg-white/[0.02] text-white/50 hover:text-white hover:bg-white/[0.06] active:scale-[0.98]"
				aria-label="Repositório GitHub"
			>
				<Github class="h-4 w-4" strokeWidth={1.8} />
			</button>
			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#161920] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl flex items-center gap-1.5">
				<span>GitHub Oficial</span>
			</div>
		</div>

		<div class="relative group w-full flex justify-center">
			<a
				href="/settings"
				onclick={(e) => {
					e.preventDefault();
					goto("/settings");
				}}
				class="relative h-10 w-10 rounded-xl flex items-center justify-center transition-all duration-200 cursor-pointer {settingsActive ? 'bg-[#1bd96a]/15 text-[#1bd96a] border border-[#1bd96a]/30' : 'bg-white/[0.02] text-white/50 hover:text-white hover:bg-white/[0.06] active:scale-[0.98]'}"
			>
				<SettingsIcon class="h-4 w-4 transition-transform duration-300 {settingsActive ? 'rotate-90' : 'group-hover:rotate-45'}" strokeWidth={1.8} />
			</a>
			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#161920] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl">
				{t("nav.settings") || "Configurações"}
			</div>
		</div>

		<div class="w-7 h-[1px] bg-white/10 my-0.5"></div>

		<button 
			type="button"
			title="Meu Perfil ({accountStatus.label})" 
			class="relative group transition-transform duration-200 active:scale-[0.98] cursor-pointer"
			onclick={() => appState.showProfileModal = true}
		>
			<div class="h-10 w-10 rounded-full overflow-hidden bg-white/[0.04] border border-white/10 group-hover:border-[#1bd96a] transition-all duration-200 shadow-sm flex items-center justify-center p-0.5">
				<img 
					src={activeSkinStore.current.avatarUrl || (account.value ? "https://mc-heads.net/avatar/" + account.value.uuid + "/100" : "/logo.png")} 
					alt="Avatar" 
					class="w-full h-full object-cover rounded-full" 
				/>
			</div>

			<span 
				class="absolute bottom-0 right-0 w-2.5 h-2.5 rounded-full border-2 border-[#0d0f12] {accountStatus.dotColor} shadow-sm"
				title={accountStatus.label}
			></span>

			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#161920] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl flex items-center gap-2">
				<span>{account.value?.username || "Perfil de Jogador"}</span>
				<span class="text-[10px] font-normal text-white/50">({accountStatus.label})</span>
			</div>
		</button>
	</div>
</aside>