<script lang="ts">
	import { page } from "$app/stores";
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
</script>

<aside class="flex h-screen w-[70px] shrink-0 flex-col items-center py-3 bg-[#111215] border-r border-white/5 z-40 relative select-none shadow-[4px_0_24px_rgba(0,0,0,0.6)]">
	
	<!-- Profile Avatar at Top (stretches to top cleanly) -->
	<button 
		type="button"
		title="Meu Perfil" 
		class="relative mb-5 group hover:scale-105 active:scale-95 transition-all duration-300 cursor-pointer"
		onclick={() => showProfileModal = true}
	>
		<div class="h-11 w-11 rounded-full overflow-hidden bg-[#1c1d22] border-2 border-white/10 group-hover:border-amber-400/80 transition-all duration-300 shadow-md flex items-center justify-center">
			<img 
				src={activeSkinStore.current.avatarUrl || (account.value ? "https://mc-heads.net/avatar/" + account.value.uuid + "/100" : "https://mc-heads.net/avatar/MHF_Steve/100")} 
				alt="Avatar" 
				class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-110" 
			/>
		</div>

		<!-- Hover Floating Tooltip -->
		<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-200 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-full border border-white/10 shadow-2xl">
			{account.value?.username || "Perfil de Jogador"}
		</div>
	</button>

	<!-- Main Navigation Icons with Fluid Hover Animations -->
	<nav class="flex-1 w-full flex flex-col items-center gap-2.5">
		{#each items as item}
			{@const active = item.href === "/" ? $page.url.pathname === "/" : $page.url.pathname.startsWith(item.href)}
			<div class="relative group w-full flex justify-center">
				<!-- Animated Pill Indicator -->
				<div 
					class="absolute left-0 top-1/2 -translate-y-1/2 w-1 rounded-r-full transition-all duration-300 ease-out {active ? 'h-7 shadow-sm' : 'h-0 bg-transparent'}"
					style={active ? 'background-color: var(--accent-color, #e2b86b);' : ''}
				></div>
				
				<a
					href={item.href}
					class="relative h-11 w-11 rounded-full flex items-center justify-center transition-all duration-300 ease-out hover:scale-110 active:scale-95 {active ? 'bg-white/10 shadow-inner' : 'text-white/40 hover:text-white hover:bg-white/5'}"
					style={active ? 'color: var(--accent-color, #e2b86b);' : ''}
				>
					<item.icon class="h-5 w-5 transition-transform duration-300 group-hover:rotate-3" />
				</a>

				<!-- Smooth Tooltip -->
				<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-200 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-full border border-white/10 shadow-2xl">
					{item.title}
				</div>
			</div>
		{/each}
		
		<div class="w-8 h-[1px] bg-white/10 my-1"></div>
		
		<!-- Dynamic Created Instances List on Sidebar -->
		{#if profiles.list.length > 0}
			<div class="w-full flex flex-col items-center gap-2 overflow-y-auto max-h-[30vh] custom-scrollbar px-1 py-1">
				{#each profiles.list as prof}
					{@const active = $page.url.pathname === `/instances/${prof.id}` || ($page.url.pathname === "/instances" && profiles.activeId === prof.id)}
					<div class="relative group w-full flex justify-center">
						<div 
							class="absolute left-0 top-1/2 -translate-y-1/2 w-1 rounded-r-full transition-all duration-300 ease-out {active ? 'h-6 shadow-sm' : 'h-0 bg-transparent'}"
							style={active ? 'background-color: var(--accent-color, #e2b86b);' : ''}
						></div>
						
						<a
							href={`/instances/${prof.id}`}
							class="relative h-10 w-10 rounded-full overflow-hidden flex items-center justify-center transition-all duration-300 ease-out hover:scale-110 active:scale-95 border {active ? 'bg-white/10 shadow-md ring-2 ring-white/20' : 'border-white/10 bg-[#1c1d22] hover:border-white/30'}"
							style={active ? 'border-color: var(--accent-color, #e2b86b);' : ''}
						>
							{#if prof.icon && (prof.icon.startsWith("http") || prof.icon.startsWith("/") || prof.icon.startsWith("data:"))}
								<img src={prof.icon} alt={prof.name} class="w-full h-full object-cover" />
							{:else}
								<img src="/grass_block.png" alt={prof.name} class="w-6 h-6 object-contain" />
							{/if}
						</a>

						<!-- Instance Tooltip with Name, Version & Loader -->
						<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-200 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-2 rounded-2xl border border-white/10 shadow-2xl flex flex-col gap-0.5">
							<div class="flex items-center gap-1.5">
								<span class="text-white font-extrabold">{prof.name}</span>
								<span class="text-[9px] font-mono font-bold bg-white/10 text-white px-1.5 py-0.2 rounded uppercase border border-white/20">{prof.loader}</span>
							</div>
							<div class="text-[10px] text-white/50 font-mono">Minecraft {prof.mcVersion}</div>
						</div>
					</div>
				{/each}
			</div>
			<div class="w-6 h-[1px] bg-white/5 my-0.5"></div>
		{/if}

		<!-- Add Instance Button with Plus Animation -->
		<div class="relative group w-full flex justify-center">
			<a
				href="/instances"
				class="h-10 w-10 rounded-full flex items-center justify-center text-white/40 hover:text-white hover:bg-white/5 hover:scale-110 active:scale-95 transition-all duration-300 border border-dashed border-white/10 hover:border-white/40"
			>
				<Plus class="h-4 w-4 transition-transform duration-300 group-hover:rotate-90 text-white/60 group-hover:text-white" />
			</a>
			<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-200 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-full border border-white/10 shadow-2xl">
				Criar Nova Instância
			</div>
		</div>
	</nav>

	<!-- Bottom Section: Friends, Settings -->
	<div class="mt-auto w-full flex flex-col items-center gap-2.5 pt-2">
		<!-- Friends Route with Tooltip -->
		<div class="relative group w-full flex justify-center">
			<div 
				class="absolute left-0 top-1/2 -translate-y-1/2 w-1 rounded-r-full transition-all duration-300 ease-out {$page.url.pathname.startsWith('/friends') ? 'h-7 shadow-sm' : 'h-0 bg-transparent'}"
				style={$page.url.pathname.startsWith('/friends') ? 'background-color: var(--accent-color, #e2b86b);' : ''}
			></div>
			<a
				href="/friends"
				class="relative h-11 w-11 rounded-full flex items-center justify-center transition-all duration-300 ease-out hover:scale-110 active:scale-95 {$page.url.pathname.startsWith('/friends') ? 'bg-white/10 shadow-inner' : 'text-white/40 hover:text-white hover:bg-white/5'}"
				style={$page.url.pathname.startsWith('/friends') ? 'color: var(--accent-color, #e2b86b);' : ''}
			>
				<Users class="h-5 w-5" />
			</a>
			<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-200 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-full border border-white/10 shadow-2xl">
				Amigos & Chat
			</div>
		</div>

		<!-- Settings Route with Tooltip -->
		<div class="relative group w-full flex justify-center">
			<div 
				class="absolute left-0 top-1/2 -translate-y-1/2 w-1 rounded-r-full transition-all duration-300 ease-out {$page.url.pathname.startsWith('/settings') ? 'h-7 shadow-sm' : 'h-0 bg-transparent'}"
				style={$page.url.pathname.startsWith('/settings') ? 'background-color: var(--accent-color, #e2b86b);' : ''}
			></div>
			<a
				href="/settings"
				class="relative h-11 w-11 rounded-full flex items-center justify-center transition-all duration-300 ease-out hover:scale-110 active:scale-95 {$page.url.pathname.startsWith('/settings') ? 'bg-white/10 shadow-inner' : 'text-white/40 hover:text-white hover:bg-white/5'}"
				style={$page.url.pathname.startsWith('/settings') ? 'color: var(--accent-color, #e2b86b);' : ''}
			>
				<SettingsIcon class="h-5 w-5 transition-transform duration-500 {$page.url.pathname.startsWith('/settings') ? 'rotate-90' : 'group-hover:rotate-45'}" />
			</a>
			<div class="pointer-events-none absolute left-[74px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-200 z-[9999] whitespace-nowrap bg-[#1e1f24] text-white text-xs font-bold px-3 py-1.5 rounded-full border border-white/10 shadow-2xl">
				Configurações
			</div>
		</div>
	</div>

	<!-- Profile Modal -->
	{#if showProfileModal}
		<ProfileModal onClose={() => showProfileModal = false} />
	{/if}
</aside>