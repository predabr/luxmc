<script lang="ts">
	import { page } from "$app/stores";
	import { goto } from "$app/navigation";
	import {
		LayoutGrid,
		Package,
		Shirt,
		Boxes,
		Plus,
		Settings as SettingsIcon,
		Github,
		Sliders
	} from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { clientMods } from "$lib/stores/clientMods.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import ProfileModal from "$lib/components/profile/ProfileModal.svelte";

	let { notificationCount = 0 }: { notificationCount?: number } = $props();
	const { t } = useTranslation();

	let showProfileModal = $state(false);

	type Item = { href: string; labelKey: string; icon: any; title: string };

	const items: Item[] = [
		{ href: "/", labelKey: "nav.home", title: "Início", icon: LayoutGrid },
		{ href: "/mods", labelKey: "nav.mods", title: "Central de Conteúdo", icon: Package },
		{ href: "/skins", labelKey: "nav.skins", title: "Personalização", icon: Shirt },
		{ href: "/instances", labelKey: "nav.instances", title: "Biblioteca", icon: Boxes },
	];

	const settingsActive = $derived($page.url.pathname.startsWith("/settings"));

	function getAccountStatus(acc: typeof account.value) {
		if (!acc) return { type: "none", label: "Desconectado", dotColor: "bg-zinc-500" };
		if (!acc.minecraftToken || acc.id.startsWith("offline_") || acc.id.startsWith("offline-")) {
			return { type: "offline", label: "Conta Offline", dotColor: "bg-sky-400" };
		}
		return { type: "online", label: "Microsoft Online", dotColor: "bg-emerald-400" };
	}

	const accountStatus = $derived(getAccountStatus(account.value));
</script>

<aside class="flex h-screen w-[70px] shrink-0 flex-col items-center py-4 bg-[#111215] border-r border-white/5 z-40 relative select-none shadow-[4px_0_24px_rgba(0,0,0,0.5)]">
	
	<button 
		type="button"
		title="Meu Perfil ({accountStatus.label})" 
		class="relative mb-4 group transition-transform duration-200 active:scale-95 cursor-pointer"
		onclick={() => showProfileModal = true}
	>
		<div class="h-11 w-11 rounded-[16px] overflow-hidden bg-[#1c1d22] border border-white/10 group-hover:border-emerald-400/70 transition-all duration-200 shadow-md flex items-center justify-center p-0.5">
			<img 
				src={activeSkinStore.current.avatarUrl || (account.value ? "https://mc-heads.net/avatar/" + account.value.uuid + "/100" : "/logo.png")} 
				alt="Avatar" 
				class="w-full h-full object-cover rounded-[14px]" 
			/>
		</div>

		<span 
			class="absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-[#111215] {accountStatus.dotColor} shadow-sm"
			title={accountStatus.label}
		></span>

		<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl flex items-center gap-2">
			<span>{account.value?.username || "Perfil de Jogador"}</span>
			<span class="text-[10px] font-normal text-white/50">({accountStatus.label})</span>
		</div>
	</button>

	<nav class="flex-1 w-full flex flex-col items-center gap-2">
		{#each items as item}
			{@const active = item.href === "/" ? $page.url.pathname === "/" : $page.url.pathname.startsWith(item.href)}
			<div class="relative group w-full flex justify-center">
				{#if active}
					<span class="absolute left-0.5 top-1/2 -translate-y-1/2 w-1 h-6 bg-emerald-400 rounded-r-full shadow-[0_0_10px_#34d399]"></span>
				{/if}
				<a
					href={item.href}
					onclick={(e) => {
						e.preventDefault();
						goto(item.href);
					}}
					class="relative h-11 w-11 rounded-[16px] flex items-center justify-center transition-all duration-200 cursor-pointer {active ? 'nav-pill-active scale-[1.02]' : 'nav-pill-inactive active:scale-95'}"
				>
					<item.icon class="h-5 w-5 {active ? 'text-[#0c0d11]' : 'text-[#8a8d98] group-hover:text-white transition-colors'}" strokeWidth={active ? 2.2 : 1.8} />
				</a>

				<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl">
					{item.title}
				</div>
			</div>
		{/each}
		
		<div class="w-7 h-[1px] bg-white/10 my-1"></div>
		
		{#if profiles.list.length > 0}
			<div class="w-full flex flex-col items-center gap-2 overflow-y-auto max-h-[28vh] custom-scrollbar px-1 py-0.5">
				{#each profiles.list as prof}
					{@const active = $page.url.pathname === `/instances/${prof.id}` || ($page.url.pathname === "/instances" && profiles.activeId === prof.id)}
					<div class="relative group w-full flex justify-center">
						{#if active}
							<span class="absolute left-0.5 top-1/2 -translate-y-1/2 w-1 h-5 bg-emerald-400 rounded-r-full shadow-[0_0_8px_#34d399]"></span>
						{/if}
						<a
							href={`/instances/${prof.id}`}
							onclick={(e) => {
								e.preventDefault();
								goto(`/instances/${prof.id}`);
							}}
							class="relative h-10 w-10 rounded-[14px] overflow-hidden flex items-center justify-center transition-all duration-200 active:scale-95 border cursor-pointer {active ? 'ring-2 ring-emerald-400 border-transparent shadow-lg scale-105' : 'border-white/10 bg-[#1c1d22] hover:border-white/30'}"
						>
							{#if prof.icon && (prof.icon.startsWith("http") || prof.icon.startsWith("/") || prof.icon.startsWith("data:"))}
								<img src={prof.icon} alt={prof.name} class="w-full h-full object-cover" />
							{:else}
								<img src="/grass_block.png" alt={prof.name} class="w-7 h-7 object-contain [image-rendering:pixelated]" />
							{/if}
						</a>

						<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-2 rounded-xl border border-white/10 shadow-2xl flex flex-col gap-0.5">
							<span class="text-white font-extrabold">{prof.name}</span>
							<span class="text-[10px] text-white/50">{prof.mcVersion} • {prof.loader}</span>
						</div>
					</div>
				{/each}
			</div>
			<div class="w-7 h-[1px] bg-white/5 my-0.5"></div>
		{/if}

		<div class="relative group w-full flex justify-center mt-1">
			<a
				href="/instances?new=true"
				onclick={(e) => {
					e.preventDefault();
					goto("/instances?new=true");
				}}
				class="h-10 w-10 rounded-[14px] bg-[#1a1b20] border border-white/10 hover:border-white/30 hover:bg-[#22242a] flex items-center justify-center text-[#8a8d98] hover:text-white transition-all duration-200 active:scale-95 shadow-md cursor-pointer"
			>
				<Plus class="h-4 w-4 transition-transform duration-200 group-hover:rotate-90 text-white/60 group-hover:text-white" />
			</a>
			<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl">
				Criar Nova Instância
			</div>
		</div>
	</nav>

	<div class="mt-auto w-full flex flex-col items-center gap-2 pt-2 border-t border-white/5">
		<div class="relative group w-full flex justify-center">
			<button
				type="button"
				onclick={() => openUrl("https://github.com/predabr/luxmc")}
				class="relative h-11 w-11 rounded-[16px] flex items-center justify-center transition-all duration-200 cursor-pointer nav-pill-inactive active:scale-95 group-hover:border-white/20"
				aria-label="Repositório GitHub"
			>
				<Github class="h-5 w-5 text-[#8a8d98] group-hover:text-white transition-colors" strokeWidth={1.8} />
			</button>
			<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl flex items-center gap-1.5">
				<span>GitHub Oficial</span>
				<span class="text-[10px] text-brand-500 font-mono">predabr/luxmc</span>
			</div>
		</div>

		<div class="relative group w-full flex justify-center">
			<button
				type="button"
				onclick={() => clientMods.toggleMenu()}
				class="relative h-11 w-11 rounded-[16px] flex items-center justify-center transition-all duration-200 cursor-pointer nav-pill-inactive active:scale-95 group-hover:border-emerald-500/40 group-hover:bg-emerald-500/10 text-emerald-400/80 group-hover:text-emerald-400"
				aria-label="Luxmc Client Suite"
			>
				<Sliders class="h-5 w-5 transition-transform duration-200 group-hover:scale-110" strokeWidth={1.9} />
			</button>
			<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl flex items-center gap-2">
				<span>Client Suite</span>
				<span class="px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-mono text-[10px]">PvP Mods</span>
			</div>
		</div>

		<div class="relative group w-full flex justify-center">
			<a
				href="/settings"
				onclick={(e) => {
					e.preventDefault();
					goto("/settings");
				}}
				class="relative h-11 w-11 rounded-[16px] flex items-center justify-center transition-all duration-200 cursor-pointer {settingsActive ? 'nav-pill-active scale-[1.02]' : 'nav-pill-inactive active:scale-95'}"
			>
				<SettingsIcon class="h-5 w-5 transition-transform duration-300 {settingsActive ? 'text-[#15171c] rotate-90' : 'text-[#8a8d98] group-hover:rotate-45 group-hover:text-white'}" strokeWidth={settingsActive ? 2.2 : 1.8} />
			</a>
			<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl">
				Configurações
			</div>
		</div>
	</div>

	{#if showProfileModal}
		<ProfileModal onClose={() => showProfileModal = false} />
	{/if}
</aside>