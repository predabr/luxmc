<script lang="ts">
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
		HardDrive,
		Download,
		Play,
		Pencil,
		Loader2,
		Square
	} from "lucide-svelte";
	import { preloadData } from "$app/navigation";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { instanceSetFavorite, stopGame } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { getIconSrc as defaultGetIconSrc } from "$lib/utils/icons";
	import { resolveProfileBanner } from "$lib/utils/curatedBanners";

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
			toast("Falha ao salvar favorito: " + String(err), "error");
		});
	}

    let now = $state(Date.now());
    onMount(() => { const timer = setInterval(() => now = Date.now(), 60000); return () => clearInterval(timer); });
    const isRunningThis = $derived(appState.isGameRunning && appState.activeGameDetails?.profileId === profile.id);
    const isLaunchingThis = $derived(
        launchingInstanceId === profile.id ||
        (appState.isLaunching && (appState.launchingProfileId === profile.id || appState.activeGameDetails?.profileId === profile.id))
    );
    const isOpening = $derived(navigatingInstanceId === profile.id);
    const lastPlayed = $derived.by(() => { now; return isRunningThis ? "Em execução agora" : (profile.lastPlayed ? formatTimeAgo(profile.lastPlayed) : 'Ainda não jogada'); });
    const minutes = $derived(gamingStats.profileMinutes(profile.id));
    const playtime = $derived(minutes <= 0 ? (isRunningThis ? '< 1 min' : '0 min') : (minutes >= 60 ? `${Math.floor(minutes / 60)}h ${minutes % 60}m` : `${minutes} min`));
    const banner = $derived(bannerFailed ? '/bg_day.jpg' : resolveProfileBanner(profile));
    const resolvedIcon = $derived(getIconSrc(profile.icon));
    const showAvatarIcon = $derived(Boolean(resolvedIcon && resolvedIcon !== '/grass_block.png' && resolvedIcon !== banner));
    const actions = $derived([
        { label: 'Configurar instância', icon: Pencil, onClick: () => onEdit?.(profile) },
        { label: 'Abrir pasta', icon: FolderOpen, onClick: () => onOpenFolder?.(profile.id) },
        { label: 'Duplicar instância', icon: Copy, onClick: () => onDuplicate?.(profile.id) },
        { label: 'Capturas de tela', icon: Image, onClick: () => onScreenshots?.(profile.id) },
        { label: 'Anotações', icon: StickyNote, onClick: () => onNotes?.(profile.id) },
        { label: 'Diagnóstico', icon: HeartPulse, onClick: () => onHealthCheck?.(profile.id) },
        { label: 'Excluir instância', icon: Trash2, variant: 'danger' as const, divider: true, onClick: () => onDelete?.(profile) }
    ]);
</script>

<div
	role="button"
	tabindex="0"
	class="surface-glass group relative cursor-pointer transition-all duration-200 active:scale-[0.98] hover:border-brand-500/30 hover:shadow-elevated {viewMode === 'grid' ? 'flex flex-col hover:-translate-y-1' : 'flex flex-wrap items-center gap-4 p-4'} {isOpening ? 'scale-[0.98] ring-2 ring-brand-400 border-brand-400 shadow-2xl brightness-105' : ''}"
	style:box-shadow={isActive && !isOpening ? "0 0 0 1px rgb(var(--brand-500) / 0.3)" : undefined}
	onpointerenter={() => { void preloadData("/instances/" + profile.id); }}
	onpointerdown={() => { void preloadData("/instances/" + profile.id); }}
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
                <Loader2 class="w-3.5 h-3.5 animate-spin text-brand-400" /> Entrando...
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
            onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
        />
    {/if}
    <div class={viewMode === 'grid' ? 'flex flex-1 flex-col p-5 pt-1' : 'min-w-32 flex-1'}>
        <div class="flex items-center gap-2"><h3 class="min-w-0 flex-1 truncate text-base font-semibold text-fg"><button type="button" class="text-left hover:text-brand-400 transition-colors focus-visible:outline-none focus-visible:underline" onclick={() => selectionMode ? onToggleSelect?.(profile.id) : onSelect?.(profile.id)}>{profile.name}</button></h3>{#if isActive}<span class="shrink-0 rounded-full bg-brand-500/10 px-2 py-1 text-[9px] font-bold text-brand-300">SELECIONADA</span>{/if}</div>
        <p class="mt-1 text-xs text-fg-subtle">Minecraft {profile.mcVersion}{#if profile.loaderVersion} · {profile.loaderVersion}{/if}</p>
        {#if profile.notes}<p class="mt-2 line-clamp-1 text-xs text-fg-muted">{profile.notes}</p>{/if}
        {#if viewMode === 'grid'}
            <div class="mt-4 rounded-2xl bg-bg-overlay/40 border border-fg/5 p-2.5 flex items-center justify-between gap-3">
                <div class="flex items-center gap-2.5 min-w-0">
                    <div class="h-8 w-8 rounded-xl bg-brand-500/15 border border-brand-500/30 flex items-center justify-center text-brand-400 shrink-0 shadow-sm">
                        <Clock class="h-4 w-4" />
                    </div>
                    <div class="min-w-0">
                        <span class="text-[9px] font-black uppercase tracking-wider text-fg-subtle block">Tempo Jogado</span>
                        <span class="text-xs font-black font-mono text-brand-300 drop-shadow-sm truncate block">{playtime}</span>
                    </div>
                </div>
                <div class="flex items-center gap-3 text-right shrink-0">
                    <div class="border-l border-fg/10 pl-3">
                        <span class="text-[9px] font-black uppercase tracking-wider text-fg-subtle block">Mods</span>
                        <span class="text-xs font-extrabold text-fg font-mono">{profile.modCount || 0}</span>
                    </div>
                    <div class="border-l border-fg/10 pl-3">
                        <span class="text-[9px] font-black uppercase tracking-wider text-fg-subtle block">RAM</span>
                        <span class="text-xs font-extrabold text-fg font-mono">{((profile.ramMb || 4096) / 1024).toFixed(1)}G</span>
                    </div>
                </div>
            </div>
            <div class="mt-3 flex items-center justify-between gap-2 text-[10px] text-fg-subtle px-0.5">
                <span class="flex items-center gap-1.5 font-medium"><span class="h-1.5 w-1.5 rounded-full {isRunningThis ? 'bg-emerald-400 animate-pulse' : 'bg-brand-400'}"></span>Última sessão: {lastPlayed}</span>
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
            <span class="mt-1 text-[10px] text-fg-subtle font-medium">{profile.modCount || 0} mods · {lastPlayed}</span>
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
    {#if selectionMode}<button type="button" role="checkbox" aria-checked={isSelected} aria-label={`Selecionar ${profile.name}`} class="relative z-10 grid h-8 w-8 place-items-center rounded-xl border border-fg/15 bg-bg-elevated text-brand-400" onclick={() => onToggleSelect?.(profile.id)}>{#if isSelected}<Check class="h-4 w-4" />{/if}</button>
    {:else}<LuxTooltip text={profile.favorite ? "Remover dos favoritos" : "Adicionar aos favoritos"} side="left"><button type="button" class="relative z-10 grid h-8 w-8 place-items-center rounded-xl border border-fg/10 bg-bg-elevated text-fg-muted hover:text-warning" aria-label={`Favoritar ${profile.name}`} aria-pressed={!!profile.favorite} onclick={handleFavorite}><Star class="h-4 w-4 {profile.favorite ? 'fill-warning text-warning' : ''}" /></button></LuxTooltip>{/if}
{/snippet}
{#snippet controls()}
    <div class="flex items-center gap-1"><LuxTooltip text="Configurações da instância"><button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={`Configurar ${profile.name}`} onclick={() => onEdit?.(profile)}><Pencil class="h-4 w-4" /></button></LuxTooltip><InstanceActionsMenu bind:isOpen={menuOpen} {actions} /></div>
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
                    toast("Minecraft encerrado com sucesso.", "info");
                } catch (e) {
                    toast("Erro ao tentar encerrar o jogo: " + String(e), "error");
                } finally {
                    appState.isStopping = false;
                }
            }}
            aria-label={`Parar ${profile.name}`}
        >
            {#if appState.isStopping}
                <Loader2 class="h-4 w-4 animate-spin" />PARANDO...
            {:else}
                <Square class="h-4 w-4 fill-current" />PARAR
            {/if}
        </button>
    {:else}
        <button
            type="button"
            class={button({ variant: 'play', size: 'md', class: viewMode === 'grid' ? 'min-w-32' : '' })}
            onclick={() => onQuickPlay?.(profile)}
            disabled={isLaunchingThis || appState.isLaunching}
            aria-label={`Jogar ${profile.name}`}
        >
            {#if isLaunchingThis}
                <Loader2 class="h-4 w-4 animate-spin" />
                <span class="truncate">{appState.launchStatusText || "Preparando..."}</span>
            {:else}
                <Play class="h-4 w-4 fill-current" />
                <span>JOGAR</span>
            {/if}
        </button>
    {/if}
{/snippet}
