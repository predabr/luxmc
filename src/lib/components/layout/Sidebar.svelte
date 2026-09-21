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
		return { type: "online", label: "Microsoft Online", dotColor: "bg-brand-500" };
	}

	const accountStatus = $derived(getAccountStatus(account.value));
</script>

<aside class="flex h-full min-h-screen w-[66px] shrink-0 flex-col items-center py-3 bg-transparent backdrop-blur-md border-r border-fg/5 transition-all duration-300 z-40 relative select-none">
	
	<div class="relative group w-full flex justify-center mb-3">
		<button
			type="button"
			onclick={() => goto("/")}
			class="relative h-11 w-11 rounded-2xl flex items-center justify-center transition-all duration-200 cursor-pointer {isHomeActive ? 'bg-brand-500 text-brand-foreground shadow-lg shadow-brand-500/25 scale-[1.04]' : 'bg-fg/5 text-fg/60 hover:text-fg hover:bg-fg/10 active:scale-[0.98]'}"
			title="Início"
		>
			<Play class="h-5 w-5 fill-current ml-0.5" />
		</button>
		<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl">
			{t("nav.home") || "Início"}
		</div>
	</div>

	<nav class="flex-1 w-full flex flex-col items-center gap-2">
		{#each items as item}
			{@const active = page.url.pathname.startsWith(item.href)}
			<div class="relative group w-full flex justify-center">
				{#if active}
					<span class="absolute left-0.5 top-1/2 -translate-y-1/2 w-1 h-6 bg-brand-500 rounded-r-full shadow-glow"></span>
				{/if}
				<a
					href={item.href}
					onclick={(e) => {
						e.preventDefault();
						goto(item.href);
					}}
					class="relative h-11 w-11 rounded-2xl flex items-center justify-center transition-all duration-200 cursor-pointer {active ? 'bg-brand-500/15 text-brand-400 border border-brand-500/30 scale-[1.02]' : 'bg-fg/[0.02] border border-transparent text-fg/50 hover:text-fg hover:bg-fg/[0.06] active:scale-[0.98]'}"
				>
					<item.icon class="h-5 w-5" strokeWidth={active ? 2.2 : 1.8} />
				</a>

				<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl">
					{t(item.labelKey) || item.title}
				</div>
			</div>
		{/each}
		
		<div class="w-7 h-[1px] bg-fg/10 my-1"></div>
		
		{#if profiles.list.length > 0}
			<div class="w-full flex flex-col items-center gap-2 overflow-y-auto overflow-x-hidden max-h-[30vh] custom-scrollbar px-1 py-0.5">
				{#each profiles.list as prof}
					{@const active = page.url.pathname === `/instances/${prof.id}` || (page.url.pathname === "/instances" && profiles.activeId === prof.id)}
					<div class="relative group w-full flex justify-center">
						{#if active}
							<span class="absolute left-0.5 top-1/2 -translate-y-1/2 w-1 h-5 bg-brand-500 rounded-r-full shadow-glow"></span>
						{/if}
						<a
							href={`/instances/${prof.id}`}
							onclick={(e) => {
								e.preventDefault();
								goto(`/instances/${prof.id}`);
							}}
							class="relative h-10 w-10 rounded-xl overflow-hidden flex items-center justify-center transition-all duration-200 active:scale-[0.98] border cursor-pointer {active ? 'ring-2 ring-brand-500 border-transparent shadow-lg scale-105' : 'border-fg/10 bg-fg/[0.03] hover:bg-fg/[0.08] hover:border-fg/30'}"
						>
							{#if prof.icon && prof.icon !== "grass_block" && prof.icon !== "/grass_block" && (prof.icon.startsWith("http") || (prof.icon.startsWith("/") && prof.icon.includes(".")) || prof.icon.startsWith("data:"))}
								<img src={prof.icon} alt={prof.name} class="w-full h-full object-cover" onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }} />
							{:else}
								<div class="w-full h-full bg-fg/[0.04] border border-fg/[0.08] flex items-center justify-center">
									<img src="/grass_block.png" alt={prof.name} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
								</div>
							{/if}
						</a>

						<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-2 rounded-xl border border-fg/10 shadow-2xl flex flex-col gap-0.5">
							<span class="text-fg font-extrabold">{prof.name}</span>
							<span class="text-[10px] text-fg/50">{prof.mcVersion} • {prof.loader}</span>
						</div>
					</div>
				{/each}
			</div>
			<div class="w-7 h-[1px] bg-fg/10 my-0.5"></div>
		{/if}

		<div class="relative group w-full flex justify-center mt-1">
			<a
				href="/instances?new=true"
				onclick={(e) => {
					e.preventDefault();
					goto("/instances?new=true");
				}}
				class="h-10 w-10 rounded-xl bg-fg/[0.03] border border-fg/10 hover:border-brand-500/50 hover:bg-brand-500/10 hover:text-brand-400 flex items-center justify-center text-fg/50 transition-all duration-200 active:scale-[0.98] shadow-sm cursor-pointer"
				title="Nova Instância"
			>
				<Plus class="h-4 w-4" />
			</a>
			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl">
				{t("home.createInstance") || "Nova Instância"}
			</div>
		</div>
	</nav>

	<div class="mt-auto w-full flex flex-col items-center gap-2 pt-2 border-t border-fg/10">
		<div class="relative group w-full flex justify-center">
			<button
				type="button"
				onclick={() => { void openPortal().catch(error => toast(String(error), "error")); }}
				class="relative h-10 w-10 rounded-xl flex items-center justify-center transition-all duration-200 cursor-pointer bg-fg/[0.02] text-fg/50 hover:text-brand-400 hover:bg-fg/[0.06] active:scale-[0.98]"
				aria-label="Portal Web & Studio 3D"
			>
				<Globe class="h-4 w-4" strokeWidth={1.8} />
			</button>
			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl flex items-center gap-1.5">
				<span>Portal Web & Studio 3D</span>
				<span class="text-[10px] text-brand-400 font-mono">pages.dev</span>
			</div>
		</div>

		<div class="relative group w-full flex justify-center">
			<button
				type="button"
				onclick={() => openUrl("https://github.com/predabr/luxmc")}
				class="relative h-10 w-10 rounded-xl flex items-center justify-center transition-all duration-200 cursor-pointer bg-fg/[0.02] text-fg/50 hover:text-fg hover:bg-fg/[0.06] active:scale-[0.98]"
				aria-label="Repositório GitHub"
			>
				<Github class="h-4 w-4" strokeWidth={1.8} />
			</button>
			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl flex items-center gap-1.5">
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
				class="relative h-10 w-10 rounded-xl flex items-center justify-center transition-all duration-200 cursor-pointer {settingsActive ? 'bg-brand-500/15 text-brand-400 border border-brand-500/30' : 'bg-fg/[0.02] text-fg/50 hover:text-fg hover:bg-fg/[0.06] active:scale-[0.98]'}"
			>
				<SettingsIcon class="h-4 w-4 transition-transform duration-300 {settingsActive ? 'rotate-90' : 'group-hover:rotate-45'}" strokeWidth={1.8} />
			</a>
			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl">
				{t("nav.settings") || "Configurações"}
			</div>
		</div>

		<div class="w-7 h-[1px] bg-fg/10 my-0.5"></div>

		<button 
			type="button" 
			title="Meu Perfil ({accountStatus.label})" 
			class="relative group transition-transform duration-200 active:scale-[0.98] cursor-pointer"
			onclick={() => appState.showProfileModal = true}
		>
			<div class="h-10 w-10 rounded-full overflow-hidden bg-fg/[0.04] border border-fg/10 group-hover:border-brand-500 transition-all duration-200 shadow-sm flex items-center justify-center p-0.5">
				<img 
					src={activeSkinStore.current.avatarUrl || (account.value ? "https://mc-heads.net/avatar/" + account.value.uuid + "/100" : "/logo.png")} 
					alt="Avatar" 
					class="w-full h-full object-cover rounded-full" 
				/>
			</div>

			<span 
				class="absolute bottom-0 right-0 w-2.5 h-2.5 rounded-full border-2 border-bg {accountStatus.dotColor} shadow-sm"
				title={accountStatus.label}
			></span>

			<div class="pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-150 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl flex items-center gap-2">
				<span>{account.value?.username || "Perfil de Jogador"}</span>
				<span class="text-[10px] font-normal text-fg/50">({accountStatus.label})</span>
			</div>
		</button>
	</div>
</aside>