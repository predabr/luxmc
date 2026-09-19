<script lang="ts">
	import { onMount } from "svelte";
	import LoaderBadge from "./LoaderBadge.svelte";
	import { button } from "$lib/components/ui/button";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import InstanceActionsMenu from "./InstanceActionsMenu.svelte";
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
		Loader2
	} from "lucide-svelte";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { instanceSetFavorite } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

	interface Props {
		profile: Profile;
		isActive?: boolean;
		tagColor?: string;
		colorOptions?: Array<{ value: string; color: string }>;
		getIconSrc: (icon?: string) => string;
		formatTimeAgo: (ts: number) => string;
		formatBytes: (bytes: number) => string;
		selectionMode?: boolean;
		isSelected?: boolean;
		launchingInstanceId?: string | null;
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
		getIconSrc,
		formatTimeAgo,
		formatBytes,
		selectionMode = false,
		isSelected = false,
		launchingInstanceId = null,
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
    const lastPlayed = $derived.by(() => { now; return profile.lastPlayed ? formatTimeAgo(profile.lastPlayed) : 'Ainda não jogada'; });
    const minutes = $derived(gamingStats.profileMinutes(profile.id));
    const playtime = $derived(minutes >= 60 ? `${Math.floor(minutes / 60)}h ${minutes % 60}m` : `${minutes} min`);
    const banner = $derived(profile.banner || (profile.icon?.startsWith('http') || profile.icon?.startsWith('/modpack_') ? profile.icon : (({ modpack_better_mc: '/modpack_better_mc.webp', modpack_cobblemon: '/modpack_cobblemon.webp', modpack_fo: '/modpack_fo.webp' } as Record<string, string>)[profile.icon] || '/bg_day.jpg')));
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
	class="surface-glass group relative cursor-pointer transition-all duration-200 hover:border-brand-500/30 hover:shadow-elevated {viewMode === 'grid' ? 'flex flex-col hover:-translate-y-1' : 'flex flex-wrap items-center gap-4 p-4'}"
	style:box-shadow={isActive ? "0 0 0 1px rgb(var(--brand-500) / 0.3)" : undefined}
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
    {#if tagColor}<span class="absolute bottom-5 left-0 top-5 w-0.5 rounded-full" style:background={colorOptions.find(color => color.value === tagColor)?.color || 'rgb(var(--brand-500))'}></span>{/if}
    {#if viewMode === 'grid'}
        <div class="relative h-40 overflow-hidden rounded-t-2xl">
            <img src={banner} alt="" class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105" loading="lazy" />
            <div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-bg-elevated/10 to-bg-overlay/15"></div>
            <div class="absolute left-4 top-4"><LoaderBadge loader={profile.loader} /></div>
            <div class="absolute right-4 top-4">{@render selection()}</div>
            <img src={getIconSrc(profile.icon)} alt="" class="absolute bottom-2 left-5 h-14 w-14 rounded-2xl border border-fg/15 bg-bg-elevated p-1.5 object-cover shadow-elevated" />
        </div>
    {:else}
        {@render selection()}
        <img src={getIconSrc(profile.icon)} alt="" class="h-12 w-12 rounded-xl border border-fg/10 bg-bg-subtle p-1 object-cover" />
    {/if}
    <div class={viewMode === 'grid' ? 'flex flex-1 flex-col p-5 pt-1' : 'min-w-32 flex-1'}>
        <div class="flex items-center gap-2"><h3 class="min-w-0 flex-1 truncate text-base font-semibold text-fg"><button type="button" class="text-left hover:text-brand-400 transition-colors focus-visible:outline-none focus-visible:underline" onclick={() => selectionMode ? onToggleSelect?.(profile.id) : onSelect?.(profile.id)}>{profile.name}</button></h3>{#if isActive}<span class="shrink-0 rounded-full bg-brand-500/10 px-2 py-1 text-[9px] font-bold text-brand-300">SELECIONADA</span>{/if}</div>
        <p class="mt-1 text-xs text-fg-subtle">Minecraft {profile.mcVersion}{#if profile.loaderVersion} · {profile.loaderVersion}{/if}</p>
        {#if profile.notes}<p class="mt-2 line-clamp-1 text-xs text-fg-muted">{profile.notes}</p>{/if}
        {#if viewMode === 'grid'}
            <div class="mt-5 grid grid-cols-3 gap-2 border-y border-fg/5 py-3">
                <div><p class="text-[10px] text-fg-subtle">Tempo jogado</p><p class="mt-1 text-xs font-semibold text-fg">{playtime}</p></div>
                <div><p class="text-[10px] text-fg-subtle">Mods</p><p class="mt-1 text-xs font-semibold text-fg">{profile.modCount || 0}</p></div>
                <div><p class="text-[10px] text-fg-subtle">Memória</p><p class="mt-1 text-xs font-semibold text-fg">{((profile.ramMb || 4096) / 1024).toFixed(1)} GB</p></div>
            </div>
            <div class="mt-3 flex items-center justify-between gap-2 text-[10px] text-fg-subtle"><span class="flex items-center gap-1.5"><Clock class="h-3 w-3" />{lastPlayed}</span>{#if profile.diskUsage}<span>{formatBytes(profile.diskUsage)}</span>{/if}</div>
            <div class="relative z-10 mt-4 flex items-center justify-between gap-2">{@render controls()}</div>
        {/if}
    </div>
    {#if viewMode === 'list'}
        <div class="hidden text-right text-xs text-fg-muted lg:block"><p>{playtime} · {profile.modCount || 0} mods</p><p class="mt-1 text-[10px] text-fg-subtle">{lastPlayed}</p></div>
        <div class="hidden sm:block"><LoaderBadge loader={profile.loader} /></div>
        <div class="relative z-10 flex items-center gap-2">{@render controls()}</div>
    {/if}
</div>

{#snippet selection()}
    {#if selectionMode}<button type="button" role="checkbox" aria-checked={isSelected} aria-label={`Selecionar ${profile.name}`} class="relative z-10 grid h-8 w-8 place-items-center rounded-xl border border-fg/15 bg-bg-elevated text-brand-400" onclick={() => onToggleSelect?.(profile.id)}>{#if isSelected}<Check class="h-4 w-4" />{/if}</button>
    {:else}<button type="button" class="relative z-10 grid h-8 w-8 place-items-center rounded-xl border border-fg/10 bg-bg-elevated text-fg-muted hover:text-warning" aria-label={`Favoritar ${profile.name}`} aria-pressed={!!profile.favorite} onclick={handleFavorite}><Star class="h-4 w-4 {profile.favorite ? 'fill-warning text-warning' : ''}" /></button>{/if}
{/snippet}
{#snippet controls()}
    <div class="flex items-center gap-1"><button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={`Configurar ${profile.name}`} onclick={() => onEdit?.(profile)}><Pencil class="h-4 w-4" /></button><InstanceActionsMenu bind:isOpen={menuOpen} {actions} /></div>
    <button type="button" class={button({ variant: 'play', size: 'md', class: viewMode === 'grid' ? 'min-w-28' : '' })} onclick={() => onQuickPlay?.(profile)} disabled={launchingInstanceId === profile.id} aria-label={`Jogar ${profile.name}`}>
        {#if launchingInstanceId === profile.id}<Loader2 class="h-4 w-4 animate-spin" />Iniciando{:else}<Play class="h-4 w-4 fill-current" />JOGAR{/if}
    </button>
{/snippet}
