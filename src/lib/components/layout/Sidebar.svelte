<script lang="ts">
	import { page } from "$app/stores";
	import { goto } from "$app/navigation";
	import {
		LayoutGrid,
		Package,
		Globe,
		Shirt,
		Boxes,
		Plus,
		Users,
		Settings as SettingsIcon,
	} from "lucide-svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import ProfileModal from "$lib/components/profile/ProfileModal.svelte";

	let { notificationCount = 0 }: { notificationCount?: number } = $props();
	const { t } = useTranslation();

	let showProfileModal = $state(false);

	type Item = { href: string; labelKey: string; icon: any; title: string };

	const items: Item[] = [
		{ href: "/", labelKey: "nav.home", title: "Início", icon: LayoutGrid },
		{ href: "/mods", labelKey: "nav.mods", title: "Central de Conteúdo", icon: Package },
		{ href: "/servers", labelKey: "nav.servers", title: "Servidores", icon: Globe },
		{ href: "/skins", labelKey: "nav.skins", title: "Personalização", icon: Shirt },
		{ href: "/instances", labelKey: "nav.instances", title: "Biblioteca", icon: Boxes },
	];

	const friendsActive = $derived($page.url.pathname.startsWith("/friends"));
	const settingsActive = $derived($page.url.pathname.startsWith("/settings"));

	function getAccountStatus(acc: typeof account.value) {
		if (!acc) return { type: "none", label: "Desconectado", dotColor: "bg-zinc-500" };
		if (!acc.minecraftToken || acc.id.startsWith("offline_")) {
			return { type: "offline", label: "Conta Offline", dotColor: "bg-sky-400" };
		}
		const expMs = acc.expiresAt > 0 && acc.expiresAt < 1e11 ? acc.expiresAt * 1000 : acc.expiresAt;
		const nowMs = Date.now();
		if (expMs > 0 && expMs < nowMs) {
			return { type: "expired", label: "Sessão Expirada", dotColor: "bg-rose-500" };
		}
		if (expMs > 0 && (expMs - nowMs) < 86400 * 1000) {
			return { type: "expiring", label: "Sessão Expirando", dotColor: "bg-amber-400" };
		}
		return { type: "online", label: "Microsoft Online", dotColor: "bg-emerald-400" };
	}

	const accountStatus = $derived(getAccountStatus(account.value));
</script>

<aside class="flex h-screen w-[70px] shrink-0 flex-col items-center py-4 bg-[#111215] border-r border-white/5 z-40 relative select-none shadow-[4px_0_24px_rgba(0,0,0,0.5)]">
	
	<!-- Profile Avatar / Brand at Top (Squircle Frame matching reference) -->
	<button 
		type="button"
		title="Meu Perfil ({accountStatus.label})" 
		class="relative mb-4 group transition-transform duration-200 active:scale-95 cursor-pointer"
		onclick={() => showProfileModal = true}
	>
		<div class="h-11 w-11 rounded-[16px] overflow-hidden bg-[#1c1d22] border border-white/10 group-hover:border-[#d8bc98]/70 transition-all duration-200 shadow-md flex items-center justify-center p-0.5">
			<img 
				src={activeSkinStore.current.avatarUrl || (account.value ? "https://mc-heads.net/avatar/" + account.value.uuid + "/100" : "/logo.png")} 
				alt="Avatar" 
				class="w-full h-full object-cover rounded-[14px]" 
			/>
		</div>

		<!-- Status Indicator Dot -->
		<span 
			class="absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-[#111215] {accountStatus.dotColor} shadow-sm"
			title={accountStatus.label}
		></span>

		<!-- Hover Floating Tooltip -->
		<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl flex items-center gap-2">
			<span>{account.value?.username || "Perfil de Jogador"}</span>
			<span class="text-[10px] font-normal text-white/50">({accountStatus.label})</span>
		</div>
	</button>

	<!-- Main Navigation Icons (Exact squircle & champagne gradient from Reference Image 3) -->
	<nav class="flex-1 w-full flex flex-col items-center gap-2">
		{#each items as item}
			{@const active = item.href === "/" ? $page.url.pathname === "/" : $page.url.pathname.startsWith(item.href)}
			<div class="relative group w-full flex justify-center">
				<a
					href={item.href}
					onclick={(e) => {
						e.preventDefault();
						goto(item.href);
					}}
					class="relative h-11 w-11 rounded-[16px] flex items-center justify-center transition-all duration-200 cursor-pointer {active ? 'nav-pill-active scale-[1.02]' : 'nav-pill-inactive active:scale-95'}"
				>
					<item.icon class="h-5 w-5 {active ? 'text-[#15171c]' : 'text-[#8a8d98] group-hover:text-white transition-colors'}" strokeWidth={active ? 2.2 : 1.8} />
				</a>

				<!-- Smooth Tooltip -->
				<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl">
					{item.title}
				</div>
			</div>
		{/each}
		
		<div class="w-7 h-[1px] bg-white/10 my-1"></div>
		
		<!-- Dynamic Created Instances List on Sidebar (Squircle styling) -->
		{#if profiles.list.length > 0}
			<div class="w-full flex flex-col items-center gap-2 overflow-y-auto max-h-[28vh] custom-scrollbar px-1 py-0.5">
				{#each profiles.list as prof}
					{@const active = $page.url.pathname === `/instances/${prof.id}` || ($page.url.pathname === "/instances" && profiles.activeId === prof.id)}
					<div class="relative group w-full flex justify-center">
						<a
							href={`/instances/${prof.id}`}
							onclick={(e) => {
								e.preventDefault();
								goto(`/instances/${prof.id}`);
							}}
							class="relative h-10 w-10 rounded-[14px] overflow-hidden flex items-center justify-center transition-all duration-200 active:scale-95 border cursor-pointer {active ? 'ring-2 ring-[#d8bc98] border-transparent shadow-lg scale-105' : 'border-white/10 bg-[#1c1d22] hover:border-white/30'}"
						>
							{#if prof.icon && (prof.icon.startsWith("http") || prof.icon.startsWith("/") || prof.icon.startsWith("data:"))}
								<img src={prof.icon} alt={prof.name} class="w-full h-full object-cover" />
							{:else}
								<img src="/grass_block.png" alt={prof.name} class="w-7 h-7 object-contain [image-rendering:pixelated]" />
							{/if}
						</a>

						<!-- Instance Tooltip with Name, Version & Loader -->
						<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-2 rounded-xl border border-white/10 shadow-2xl flex flex-col gap-0.5">
							<span class="text-white font-extrabold">{prof.name}</span>
							<span class="text-[10px] text-white/50">{prof.mcVersion} • {prof.loader}</span>
						</div>
					</div>
				{/each}
			</div>
			<div class="w-7 h-[1px] bg-white/5 my-0.5"></div>
		{/if}

		<!-- Add Instance Button with Plus -->
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

	<!-- Bottom Section: Friends, Settings -->
	<div class="mt-auto w-full flex flex-col items-center gap-2 pt-2 border-t border-white/5">
		<!-- Friends Route with Tooltip -->
		<div class="relative group w-full flex justify-center">
			<a
				href="/friends"
				onclick={(e) => {
					e.preventDefault();
					goto("/friends");
				}}
				class="relative h-11 w-11 rounded-[16px] flex items-center justify-center transition-all duration-200 cursor-pointer {friendsActive ? 'nav-pill-active scale-[1.02]' : 'nav-pill-inactive active:scale-95'}"
			>
				<Users class="h-5 w-5 {friendsActive ? 'text-[#15171c]' : 'text-[#8a8d98] group-hover:text-white transition-colors'}" strokeWidth={friendsActive ? 2.2 : 1.8} />
			</a>
			<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-xl border border-white/10 shadow-2xl">
				Amigos
			</div>
		</div>

		<!-- Settings Route with Tooltip -->
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

	<!-- Profile Modal -->
	{#if showProfileModal}
		<ProfileModal onClose={() => showProfileModal = false} />
	{/if}
</aside>