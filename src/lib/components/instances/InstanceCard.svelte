<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { onMount } from "svelte";
	import LoaderBadge from "./LoaderBadge.svelte";
	import { button } from "$lib/components/ui/button";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import InstanceActionsMenu from "./InstanceActionsMenu.svelte";
	import LuxTooltip from "$lib/components/ui/LuxTooltip.svelte";
	import {
		Trash2,
		Check,
		Box,
		Copy,
		FolderOpen,
		Image,
		HeartPulse,
		Star,
		StickyNote,
		Clock,
        Cpu,
		HardDrive,
		Download,
		Play,
		Pencil,
		Loader2,
		Square
	} from "lucide-svelte";
	import { preloadRoute } from "$lib/utils/preloadRoute";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { instanceSetFavorite, stopGame } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { getIconSrc as defaultGetIconSrc } from "$lib/utils/icons";
	import { resolveProfileBanner } from "$lib/utils/curatedBanners";
	import { startTicker, stopTicker, tickerNow } from "$lib/stores/ticker.svelte";

	const { t } = useTranslation();

	interface Props {
		profile: Profile;
		isActive?: boolean;
		tagColor?: string;
		colorOptions?: Array<{ value: string; color: string }>;
		getIconSrc?: (icon?: string) => string;
		formatTimeAgo: (ts: number) => string;
		formatBytes: (bytes: number) => string;
		selectionMode?: boolean;
		isSelected?: boolean;
		launchingInstanceId?: string | null;
		navigatingInstanceId?: string | null;
		onSelect?: (id: string) => void;
		onToggleSelect?: (id: string) => void;
		onQuickPlay?: (p: Profile) => void;
		onEdit?: (p: Profile) => void;
		onOpenFolder?: (id: string) => void;
		onDelete?: (p: Profile) => void;
		onDuplicate?: (id: string) => void;
		onScreenshots?: (id: string) => void;
		onNotes?: (id: string) => void;
		onHealthCheck?: (id: string) => void;
		viewMode?: "grid" | "list";
	}

	let {
		profile,
		isActive = false,
		tagColor,
		colorOptions = [],
		getIconSrc = defaultGetIconSrc,
		formatTimeAgo,
		formatBytes,
		selectionMode = false,
		isSelected = false,
		launchingInstanceId = null,
		navigatingInstanceId = null,
		onSelect,
		onToggleSelect,
		onQuickPlay,
		onEdit,
		onOpenFolder,
		onDelete,
		onDuplicate,
		onScreenshots,
		onNotes,
		onHealthCheck,
		viewMode = "grid"
	}: Props = $props();

	let menuOpen = $state(false);
	let bannerFailed = $state(false);

	$effect(() => {
		profile.id;
		profile.banner;
		bannerFailed = false;
	});

	function handleFavorite(e: MouseEvent) {
		e.stopPropagation();
		const nextFav = !profile.favorite;
		profiles.toggleFavorite(profile.id);
		instanceSetFavorite(profile.id, nextFav).catch((err) => {
			profiles.toggleFavorite(profile.id);
			toast(uiText("ui.350ccac3fb6c3c2b") + String(err), "error");
		});
	}

    onMount(() => { startTicker(); return () => stopTicker(); });
    const isRunningThis = $derived(appState.isGameRunning && appState.activeGameDetails?.profileId === profile.id);
    const isLaunchingThis = $derived(
        launchingInstanceId === profile.id ||
        (appState.isLaunching && (appState.launchingProfileId === profile.id || appState.activeGameDetails?.profileId === profile.id))
    );
    const isOpening = $derived(navigatingInstanceId === profile.id);
    const lastPlayed = $derived.by(() => { tickerNow(); return isRunningThis ? uiText("ui.d1122f2d310f1106") : (profile.lastPlayed ? formatTimeAgo(profile.lastPlayed) : uiText("ui.307a3b6fb9276530")); });
    const minutes = $derived(gamingStats.profileMinutes(profile.id));
    const playtime = $derived(minutes <= 0 ? (isRunningThis ? '< 1 min' : '0 min') : (minutes >= 60 ? `${Math.floor(minutes / 60)}h ${minutes % 60}m` : `${minutes} min`));
    const banner = $derived(bannerFailed ? '/bg_day.jpg' : resolveProfileBanner(profile));
    const resolvedIcon = $derived(getIconSrc(profile.icon));
    const showAvatarIcon = $derived(Boolean(resolvedIcon && resolvedIcon !== '/grass_block.png' && resolvedIcon !== banner));
    const actions = $derived([
        { label: uiText("ui.4ea489ea6c7e5d1e"), icon: Pencil, onClick: () => onEdit?.(profile) },
        { label: uiText("screenshots.openFolderBtn"), icon: FolderOpen, onClick: () => onOpenFolder?.(profile.id) },
        { label: uiText("ui.ac95c6bd19634d35"), icon: Copy, onClick: () => onDuplicate?.(profile.id) },
        { label: uiText("nav.screenshots"), icon: Image, onClick: () => onScreenshots?.(profile.id) },
        { label: uiText("ui.9306b6186e54f518"), icon: StickyNote, onClick: () => onNotes?.(profile.id) },
        { label: uiText("ui.e695eb16e81f3704"), icon: HeartPulse, onClick: () => onHealthCheck?.(profile.id) },
        { label: uiText("ui.79ef883cba2c401e"), icon: Trash2, variant: 'danger' as const, divider: true, onClick: () => onDelete?.(profile) }
    ]);
</script>

<div
	role="button"
	tabindex="0"
	class="surface-glass group relative cursor-pointer active:scale-[0.98] hover:border-brand-500/30 hover:shadow-elevated {viewMode === 'grid' ? 'flex flex-col' : 'flex flex-wrap items-center gap-4 p-4'} {isOpening ? 'scale-[0.98] ring-2 ring-brand-400 border-brand-400 shadow-2xl brightness-105' : ''}"
	style:box-shadow={isActive && !isOpening ? "0 0 0 1px rgb(var(--brand-500) / 0.3)" : undefined}
	onpointerenter={() => preloadRoute("/instances/" + profile.id)}
	onpointerdown={() => preloadRoute("/instances/" + profile.id, true)}
	onclick={(e) => {
		const target = e.target as HTMLElement | null;
		if (target?.closest('button, a, input, select, textarea, [role="checkbox"], [role="menu"]')) return;
		if (selectionMode) onToggleSelect?.(profile.id);
		else onSelect?.(profile.id);
	}}
	onkeydown={(e) => {
		if (e.key === "Enter" || e.key === " ") {
			const target = e.target as HTMLElement | null;
			if (target?.closest('button, a, input, select, textarea, [role="checkbox"], [role="menu"]')) return;
			e.preventDefault();
			if (selectionMode) onToggleSelect?.(profile.id);
			else onSelect?.(profile.id);
		}
	}}
>
    {#if isOpening}
        <div class="absolute inset-0 bg-bg-elevated/80 rounded-2xl flex items-center justify-center z-30 pointer-events-none transition-opacity duration-150">
            <div class="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-bg-elevated border border-brand-400/40 text-xs font-bold text-brand-300 shadow-lg">
                <Loader2 class="w-3.5 h-3.5 animate-spin text-brand-400" /> {uiText("ui.a89d514549530e6e")}
            </div>
        </div>
    {/if}
    {#if tagColor}<span class="absolute bottom-5 left-0 top-5 w-0.5 rounded-full" style:background={colorOptions.find(color => color.value === tagColor)?.color || 'rgb(var(--brand-500))'}></span>{/if}
    {#if viewMode === 'grid'}
        <div class="relative h-44 overflow-hidden rounded-t-2xl bg-bg-subtle">
            <img
                src={banner}
                alt=""
                class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
                loading="lazy"
                decoding="async"
                onerror={() => { bannerFailed = true; }}
            />
            <div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-bg-elevated/25 to-transparent"></div>
            <div class="absolute inset-0 bg-gradient-to-b from-black/40 via-transparent to-transparent"></div>
            <div class="absolute left-4 top-4 z-10"><LoaderBadge loader={profile.loader} /></div>
            <div class="absolute right-4 top-4 z-10">{@render selection()}</div>
            {#if showAvatarIcon}
                <img
                    src={resolvedIcon}
                    alt={profile.name}
                    class="absolute bottom-2 left-4 z-10 h-12 w-12 rounded-xl border border-fg/15 bg-bg-elevated/90 p-1 object-contain shadow-elevated"
                    loading="lazy"
                    decoding="async"
                    onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
                />
            {/if}
        </div>
    {:else}
        {@render selection()}
        <img
            src={resolvedIcon}
            alt={profile.name}
            class="h-12 w-12 rounded-xl border border-fg/10 bg-bg-elevated/90 p-1 object-contain shrink-0"
            loading="lazy"
            decoding="async"
            onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
        />
    {/if}
    <div class={viewMode === 'grid' ? 'flex flex-1 flex-col p-5 pt-1' : 'min-w-32 flex-1'}>
        <div class="flex items-center gap-2"><h3 class="min-w-0 flex-1 truncate text-base font-semibold text-fg"><button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "text-left focus-visible:outline-none focus-visible:underline" })} onclick={() => selectionMode ? onToggleSelect?.(profile.id) : onSelect?.(profile.id)}>{profile.name}</button></h3>{#if isActive}<span class="shrink-0 rounded-full bg-brand-500/10 px-2 py-1 text-[9px] font-bold text-brand-300">{uiText("ui.839d7fbe46b3b4d9")}</span>{/if}</div>
        <p class="mt-1 text-xs text-fg-subtle">Minecraft {profile.mcVersion}{#if profile.loaderVersion} · {profile.loaderVersion}{/if}</p>
        {#if profile.notes}<p class="mt-2 line-clamp-1 text-xs text-fg-muted">{profile.notes}</p>{/if}
        {#if viewMode === 'grid'}
            <div class="mt-4 grid grid-cols-3 gap-1 rounded-2xl border border-fg/10 bg-fg/[0.025] p-2">
                <div class="min-w-0 px-2 py-2">
                    <Clock class="mb-2 h-4 w-4 text-brand-400" />
                    <span class="block truncate text-[10px] font-medium text-fg-muted">{uiText("ui.173500a0335fc472")}</span>
                    <span class="mt-1 block truncate text-sm font-semibold tabular-nums text-fg">{playtime}</span>
                </div>
                <div class="min-w-0 border-l border-fg/10 px-2 py-2">
                    <Box class="mb-2 h-4 w-4 text-brand-400" />
                    <span class="block text-[10px] font-medium text-fg-muted">Mods</span>
                    <span class="mt-1 block text-sm font-semibold tabular-nums text-fg">{profile.modCount || 0}</span>
                </div>
                <div class="min-w-0 border-l border-fg/10 px-2 py-2">
                    <Cpu class="mb-2 h-4 w-4 text-brand-400" />
                    <span class="block text-[10px] font-medium text-fg-muted">RAM</span>
                    <span class="mt-1 block whitespace-nowrap text-sm font-semibold tabular-nums text-fg">{((profile.ramMb || 4096) / 1024).toFixed(1)} <span class="text-[10px] text-fg-muted">GB</span></span>
                </div>
            </div>
            <div class="mt-3 flex items-center justify-between gap-2 text-[10px] text-fg-subtle px-0.5">
                <span class="flex items-center gap-1.5 font-medium"><span class="h-1.5 w-1.5 rounded-full {isRunningThis ? 'bg-emerald-400 animate-pulse' : 'bg-brand-400'}"></span>{uiText("ui.3e38d7ca2ab89f80")} {lastPlayed}</span>
                {#if profile.diskUsage}<span class="font-mono">{formatBytes(profile.diskUsage)}</span>{/if}
            </div>
            <div class="relative z-10 mt-4 flex items-center justify-between gap-2">{@render controls()}</div>
        {/if}
    </div>
    {#if viewMode === 'list'}
        <div class="hidden text-right text-xs lg:flex flex-col items-end">
            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-brand-500/10 border border-brand-500/25 text-brand-300 font-mono font-black text-xs shadow-sm">
                <Clock class="h-3.5 w-3.5 text-brand-400" />
                {playtime}
            </span>
            <span class="mt-1 text-[10px] text-fg-subtle font-medium">{profile.modCount || 0} {uiText("ui.3ee1f063d3fea8fb")} {lastPlayed}</span>
        </div>
        <div class="hidden sm:block"><LoaderBadge loader={profile.loader} /></div>
        <div class="relative z-10 flex items-center gap-2">{@render controls()}</div>
    {/if}
</div>

<style>
	.card-mesh {
		background-image:
			linear-gradient(rgb(var(--fg) / 0.045) 1px, transparent 1px),
			linear-gradient(90deg, rgb(var(--fg) / 0.045) 1px, transparent 1px);
		background-size: 28px 28px;
	}
</style>

{#snippet selection()}
    {#if selectionMode}<button type="button" role="checkbox" aria-checked={isSelected} aria-label={uiText("ui.cdcc8c2293d065b1", {arg0: (profile.name)})} class={launcherButton({ variant: "secondary", size: "sm", class: "relative z-10 grid place-items-center" })} onclick={() => onToggleSelect?.(profile.id)}>{#if isSelected}<Check class="h-4 w-4" />{/if}</button>
    {:else}<LuxTooltip text={profile.favorite ? uiText("ui.9033705b99bcc5eb") : uiText("ui.5eb5aff51b27df3d")} side="left"><button type="button" class={launcherButton({ variant: "secondary", size: "icon", class: "relative z-10 grid place-items-center" })} aria-label={`Favoritar ${profile.name}`} aria-pressed={!!profile.favorite} onclick={handleFavorite}><Star class="h-4 w-4 {profile.favorite ? 'fill-warning text-warning' : ''}" /></button></LuxTooltip>{/if}
{/snippet}
{#snippet controls()}
    <div class="flex items-center gap-1"><LuxTooltip text={uiText("ui.6c5b8d86c6726795")}><button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={`Configurar ${profile.name}`} onclick={() => onEdit?.(profile)}><Pencil class="h-4 w-4" /></button></LuxTooltip><InstanceActionsMenu bind:isOpen={menuOpen} {actions} /></div>
    {#if isRunningThis}
        <button
            type="button"
            disabled={appState.isStopping}
            class={button({ variant: 'danger', size: 'md', class: `${viewMode === 'grid' ? 'min-w-28' : ''} ${appState.isStopping ? 'opacity-50' : ''}` })}
            onclick={async () => {
                try {
                    appState.isStopping = true;
                    appState.wasManuallyTerminated = true;
                    await stopGame();
                    appState.isGameRunning = false;
                    gamingStats.onGameExit();
                    toast(uiText("ui.69429e8fd4528906"), "info");
                } catch (e) {
                    toast(uiText("ui.f635cd0acfe6def9") + String(e), "error");
                } finally {
                    appState.isStopping = false;
                }
            }}
            aria-label={`Parar ${profile.name}`}
        >
            {#if appState.isStopping}
                <Loader2 class="h-4 w-4 animate-spin" />{uiText("ui.504e804c01b6e701")}
            {:else}
                <Square class="h-4 w-4 fill-current" />{uiText("ui.3a74114135495503")}
            {/if}
        </button>
    {:else}
        <button
            type="button"
            class={button({ variant: 'play', size: 'md', class: viewMode === 'grid' ? 'min-w-32' : '' })}
            onclick={() => onQuickPlay?.(profile)}
            disabled={isLaunchingThis || appState.isLaunching}
            aria-label={uiText("ui.1172eaaf58da8c29", {arg0: (profile.name)})}
        >
            {#if isLaunchingThis}
                <Loader2 class="h-4 w-4 animate-spin" />
                <span class="truncate">{appState.launchStatusText || uiText("ui.dc0546b3e22c8f9e")}</span>
            {:else}
                <Play class="h-4 w-4 fill-current" />
                <span>{uiText("home.play")}</span>
            {/if}
        </button>
    {/if}
{/snippet}
