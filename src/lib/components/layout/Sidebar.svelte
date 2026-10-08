<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import { onMount } from "svelte";
    import { PanelLeftClose, PanelLeftOpen, Users } from "lucide-svelte";
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
	import { friendsState } from "$lib/stores/friends.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { getIconSrc } from "$lib/utils/icons";

	let { notificationCount = 0 }: { notificationCount?: number } = $props();
	const { t } = useTranslation();
    let expanded = $state(false);
    onMount(() => { try { expanded = localStorage.getItem("luxmc_sidebar_expanded") === "true"; } catch {} const media=matchMedia("(max-width: 700px)");const adapt=()=>{if(media.matches)expanded=false;};adapt();media.addEventListener("change",adapt);return()=>media.removeEventListener("change",adapt); });
    function toggleSidebar() { expanded = !expanded; try { localStorage.setItem("luxmc_sidebar_expanded", String(expanded)); } catch {} }

	type Item = { href: string; labelKey: string; icon: typeof import("lucide-svelte").Circle; title: string };

	const items: Item[] = $derived([
		{ href: "/mods", labelKey: "nav.mods", title: uiText("home.browseMods"), icon: Compass },
		{ href: "/skins", labelKey: "nav.skins", title: uiText("ui.786fa4f02a1a0dfc"), icon: Shirt },
		{ href: "/friends", labelKey: "nav.friends", title: uiText("ui.30b21c1ed5a0967b"), icon: Users },
		{ href: "/instances", labelKey: "nav.instances", title: uiText("library.title"), icon: Layers },
	]);

	const isHomeActive = $derived(page.url.pathname === "/");
	const settingsActive = $derived(page.url.pathname.startsWith("/settings"));

	function getAccountStatus(acc: typeof account.value) {
		if (!acc) return { type: "none", label: uiText("ui.879b012449c1fd87"), dotColor: "bg-zinc-500" };
		if (!acc.minecraftToken || acc.id.startsWith("offline_") || acc.id.startsWith("offline-")) {
			return { type: "offline", label: uiText("ui.2793472a35db2b80"), dotColor: "bg-sky-400" };
		}
		return { type: "online", label: uiText("ui.8b89c55dfc443d48"), dotColor: "bg-brand-500" };
	}

	const accountStatus = $derived(getAccountStatus(account.value));
</script>

<aside class="launcher-sidebar flex h-dvh min-h-0 {expanded ? 'w-60' : 'w-[66px]'} shrink-0 flex-col items-center py-3 bg-bg-elevated/80 backdrop-blur-2xl border-r border-fg/[0.08] transition-[width] duration-200 z-40 relative select-none">
	
    <button class={launcherButton({ variant: "secondary", size: "sm", class: `mb-3 flex items-center gap-3 ${expanded ? 'w-[calc(100%_-_1.5rem)] justify-between' : ''}` })} aria-label={uiText(expanded ? "design.collapseMenu" : "design.expandMenu")} aria-expanded={expanded} onclick={toggleSidebar}>
        {#if expanded}<PanelLeftClose class="h-4 w-4" /><span class="font-semibold tracking-wide text-fg">LUXMC</span>{:else}<PanelLeftOpen class="h-4 w-4" />{/if}
    </button>
	<div class="relative group w-full flex justify-center mb-3">
		<button
			type="button"
			onclick={() => goto("/")}
			class="launcher-nav-link relative h-11 {expanded ? 'w-full mx-3 gap-3 px-3 justify-start' : 'w-11 justify-center'} rounded-2xl flex items-center transition-[background-color,color,transform] duration-150 cursor-pointer {isHomeActive ? 'bg-brand-500 text-brand-foreground shadow-sm' : 'bg-fg/5 text-fg/60 hover:text-fg hover:bg-fg/10'}"
			aria-current={isHomeActive ? "page" : undefined}
			title={t("nav.home") || uiText("nav.home")}
		>
			<Play class="h-5 w-5 fill-current ml-0.5 shrink-0" />
			{#if expanded}<span class="text-sm font-medium">{t("nav.home") || uiText("nav.home")}</span>{/if}
		</button>
		<div class="{expanded ? 'hidden' : ''} pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-[opacity,transform] duration-100 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl">
			{t("nav.home") || uiText("nav.home")}
		</div>
	</div>

	<nav class="min-h-0 overflow-y-auto custom-scrollbar flex-1 w-full flex flex-col items-center gap-2">
		{#each items as item}
			{@const active = page.url.pathname.startsWith(item.href) || (item.href === "/friends" && page.url.pathname === "/hosting")}
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
					aria-label={t(item.labelKey) || item.title}
                    aria-current={active ? "page" : undefined}
                    class="launcher-nav-link relative h-11 {expanded ? 'w-full mx-3 gap-3 px-3 justify-start' : 'w-11 justify-center'} rounded-2xl flex items-center transition-[background-color,color,border-color,transform] duration-150 cursor-pointer {active ? 'bg-brand-500/15 text-brand-400 border border-brand-500/30' : 'bg-fg/[0.02] border border-transparent text-fg/50 hover:text-fg hover:bg-fg/[0.06]'}"
				>
					<item.icon class="h-5 w-5 shrink-0" strokeWidth={active ? 2.2 : 1.8} />
                    {#if expanded}<span class="text-sm font-medium">{t(item.labelKey) || item.title}</span>{/if}
				</a>

				<div class="{expanded ? 'hidden' : ''} pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-[opacity,transform] duration-100 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl">
					{t(item.labelKey) || item.title}
				</div>
			</div>
		{/each}
		
		<div class="w-7 h-[1px] bg-fg/10 my-1"></div>
		
		{#if profiles.list.length > 0}
			<div class="w-full flex flex-col items-center gap-2 overflow-y-auto overflow-x-hidden max-h-[30vh] custom-scrollbar px-1 py-0.5">
				{#each profiles.list as prof (prof.id)}
					{@const active = page.url.pathname === `/instances/${prof.id}` || (page.url.pathname === "/instances" && profiles.activeId === prof.id)}
					{@const iconSrc = getIconSrc(prof.icon)}
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
							class="launcher-nav-link relative {expanded ? 'h-14 w-full mx-2 px-2 gap-3 justify-start' : 'h-10 w-10 justify-center'} rounded-xl overflow-hidden flex items-center transition-[background-color,border-color] duration-150 border cursor-pointer {active ? 'border-brand-500/40 bg-brand-500/10' : 'border-fg/10 bg-fg/[0.03] hover:bg-fg/[0.08] hover:border-fg/30'}"
							aria-current={active ? "page" : undefined}
							aria-label={prof.name}
						>
							<div class="h-9 w-9 shrink-0 rounded-lg overflow-hidden flex items-center justify-center">
							{#if iconSrc !== "/grass_block.png"}
								<img loading="lazy" decoding="async" src={iconSrc} alt={prof.name} class="w-full h-full object-cover" onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }} />
							{:else}
								<div class="w-full h-full bg-fg/[0.04] border border-fg/[0.08] flex items-center justify-center">
									<img loading="lazy" decoding="async" src="/grass_block.png" alt={prof.name} class="w-8 h-8 object-contain [image-rendering:pixelated]" />
								</div>
							{/if}
							</div>
							{#if expanded}<span class="min-w-0 flex-1 text-left"><span class="block truncate text-xs font-semibold text-fg">{prof.name}</span><span class="mt-1 block truncate text-[10px] text-fg-muted">{prof.mcVersion} · {prof.loader}</span></span>{/if}
						</a>

						<div class="{expanded ? 'hidden' : ''} pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-[opacity,transform] duration-100 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-2 rounded-xl border border-fg/10 shadow-2xl flex flex-col gap-0.5">
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
				class={launcherButton({ variant: "outline", size: expanded ? "md" : "icon", class: expanded ? "w-[calc(100%_-_1.5rem)] justify-start" : "" })}
				title={uiText("instances.newInstance")}
			>
				<Plus class="h-4 w-4" />
				{#if expanded}<span class="truncate">{uiText("instances.newInstance")}</span>{/if}
			</a>
			<div class="{expanded ? 'hidden' : ''} pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-[opacity,transform] duration-100 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl">
				{t("home.createInstance") || uiText("instances.newInstance")}
			</div>
		</div>
	</nav>

	<div class="mt-auto shrink-0 pb-1 w-full flex flex-col items-center gap-2 pt-2 border-t border-fg/10">
		<div class="relative group w-full flex justify-center">
			<button
				type="button"
				onclick={() => { void openPortal().catch(error => toast(String(error), "error")); }}
				class={launcherButton({ variant: "ghost", size: expanded ? "md" : "icon", class: expanded ? "w-[calc(100%_-_1.5rem)] justify-start" : "" })}
				aria-label={uiText("ui.bccbcc41cb5cb8a9")}
			>
				<Globe class="h-4 w-4" strokeWidth={1.8} />
				{#if expanded}<span class="truncate">{uiText("design.webPortal")}</span>{/if}
			</button>
			<div class="{expanded ? 'hidden' : ''} pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-[opacity,transform] duration-100 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl flex items-center gap-1.5">
				<span>{uiText("ui.bccbcc41cb5cb8a9")}</span>
				<span class="text-[10px] text-brand-400 font-mono">{uiText("ui.2c47c5da0bb95417")}</span>
			</div>
		</div>

		<div class="relative group w-full flex justify-center">
			<button
				type="button"
				onclick={() => openUrl("https://github.com/predabr/luxmc")}
				class={launcherButton({ variant: "ghost", size: expanded ? "md" : "icon", class: expanded ? "w-[calc(100%_-_1.5rem)] justify-start" : "" })}
				aria-label={uiText("ui.a1db67212cb86502")}
			>
				<Github class="h-4 w-4" strokeWidth={1.8} />
				{#if expanded}<span class="truncate">GitHub</span>{/if}
			</button>
			<div class="{expanded ? 'hidden' : ''} pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-[opacity,transform] duration-100 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl flex items-center gap-1.5">
				<span>{uiText("ui.9f7dd2ab68dc081b")}</span>
			</div>
		</div>

		<div class="relative group w-full flex justify-center">
			<a
				href="/settings"
				onclick={(e) => {
					e.preventDefault();
					goto("/settings");
				}}
				class="launcher-nav-link relative h-11 {expanded ? 'w-full mx-3 gap-3 px-3 justify-start' : 'w-10 justify-center'} rounded-xl flex items-center transition-[background-color,color,border-color] duration-150 cursor-pointer {settingsActive ? 'bg-brand-500/15 text-brand-400 border border-brand-500/30' : 'bg-fg/[0.02] text-fg/50 hover:text-fg hover:bg-fg/[0.06]'}"
				aria-current={settingsActive ? "page" : undefined}
			>
				<SettingsIcon class="h-4 w-4 shrink-0 transition-transform duration-200 {settingsActive ? 'rotate-90' : 'group-hover:rotate-45'}" strokeWidth={1.8} />
				{#if expanded}<span class="text-sm font-medium">{t("nav.settings") || uiText("ui.76b0fb6ad18939ac")}</span>{/if}
			</a>
			<div class="{expanded ? 'hidden' : ''} pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-[opacity,transform] duration-100 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl">
				{t("nav.settings") || uiText("ui.76b0fb6ad18939ac")}
			</div>
		</div>

		<div class="w-7 h-[1px] bg-fg/10 my-0.5"></div>

		<button 
			type="button" 
			title={`${uiText("ui.93e27c44e7cbee87")}${accountStatus.label})`}
			class={launcherButton({ variant: "ghost", size: "lg", class: `relative group ${expanded ? 'w-[calc(100%_-_1.5rem)] justify-start px-2' : 'px-0'}` })}
			onclick={() => appState.showProfileModal = true}
		>
			<div class="h-10 w-10 rounded-full overflow-hidden bg-fg/[0.04] border border-fg/10 group-hover:border-brand-500 transition-[border-color] duration-150 shadow-sm flex items-center justify-center p-0.5">
				<img loading="lazy" decoding="async" 
					src={friendsState.ownProfile?.portrait || activeSkinStore.current.avatarUrl || account.value?.avatarUrl || (account.value ? "https://mc-heads.net/avatar/" + (account.value.username || account.value.uuid) + "/100" : "/logo.png")}
					alt={uiText("ui.ca8e826d9c2ec401")} 
					class="w-full h-full object-cover rounded-full" 
					onerror={(e) => {
						const img = e.currentTarget as HTMLImageElement;
						const fallback = account.value?.username ? `https://mc-heads.net/avatar/${account.value.username}/100` : "/logo.png";
						if (img.src !== fallback) img.src = fallback;
						else img.src = "/logo.png";
					}}
				/>
			</div>
			{#if expanded}<span class="min-w-0 text-left"><span class="block truncate text-xs font-semibold text-fg">{friendsState.ownProfile?.displayName || account.value?.username || uiText("ui.bab0dd5d766ccc9a")}</span><span class="mt-1 block text-[10px] text-fg-muted">{friendsState.ownProfile?.role === "owner" ? uiText("ownerTools.badge") : accountStatus.label}</span></span>{/if}

			<span 
				class="absolute bottom-0 right-0 w-2.5 h-2.5 rounded-full border-2 border-bg {accountStatus.dotColor} shadow-sm"
				title={accountStatus.label}
			></span>

			<div class="{expanded ? 'hidden' : ''} pointer-events-none absolute left-[70px] top-1/2 -translate-y-1/2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-[opacity,transform] duration-100 z-[9999] whitespace-nowrap bg-bg-elevated text-fg text-xs font-bold px-3 py-1.5 rounded-xl border border-fg/10 shadow-2xl flex items-center gap-2">
				<span>{friendsState.ownProfile?.displayName || account.value?.username || uiText("ui.bab0dd5d766ccc9a")}</span>
				<span class="text-[10px] font-normal text-fg/50">({accountStatus.label})</span>
			</div>
		</button>
	</div>
</aside>
