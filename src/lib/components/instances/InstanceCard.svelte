<script lang="ts">
	import { fade } from "svelte/transition";
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
		profiles.toggleFavorite(profile.id);
		instanceSetFavorite(profile.id, !profile.favorite).catch(() => {});
	}
</script>

{#if viewMode === "grid"}
	<div class="relative group hover:-translate-y-1 hover:shadow-xl hover:shadow-black/20 transition-all duration-300 ease-out">
		{#if selectionMode}
			<button
				class="absolute left-2 top-2 z-10 grid h-5 w-5 place-items-center rounded transition-colors"
				style="border: 1px solid {isSelected ? 'rgb(45, 212, 191)' : 'rgb(var(--border))'}; background: {isSelected ? 'rgba(45, 212, 191, 0.2)' : 'rgb(var(--bg-elevated))'};"
				onclick={(e) => { e.stopPropagation(); onToggleSelect?.(profile.id); }}
				role="checkbox"
				aria-checked={isSelected}
				aria-label={`${t("instances.select")}: ${profile.name}`}
			>
				{#if isSelected}
					<Check class="h-3 w-3" style="color: rgb(45, 212, 191);" />
				{/if}
			</button>
		{/if}
		<Card interactive onclick={() => onSelect?.(profile.id)}>
			<div class="h-36 -mx-4 -mt-4 mb-3 rounded-t-2xl overflow-hidden relative bg-[#1c1d22]">
				<img
					src={profile.loader === 'fabric' ? '/modpack_fo.webp' : profile.loader === 'forge' ? '/modpack_better_mc.webp' : profile.loader === 'neoforge' ? '/modpack_cobblemon.webp' : '/vanilla_banner.png'}
					alt="Minecraft Artwork"
					class="w-full h-full object-cover opacity-85 group-hover:scale-105 transition-transform duration-500"
				/>
				<div class="absolute inset-0 bg-gradient-to-t from-[#141518] via-transparent to-transparent"></div>
				<div class="absolute top-2.5 right-2.5 bg-black/70 backdrop-blur-md text-[#caa97c] text-[10px] font-black uppercase px-2.5 py-1 rounded-full border border-white/10 shadow-lg flex items-center gap-1.5">
					<span class="w-1.5 h-1.5 rounded-full bg-[#caa97c]"></span>
					<span>{profile.loader.toUpperCase()} · {profile.mcVersion}</span>
				</div>
				<div class="absolute bottom-2.5 left-3 h-10 w-10 rounded-xl bg-[#14151a] border-2 border-white/15 p-0.5 shadow-xl flex items-center justify-center overflow-hidden">
					<img src={getIconSrc(profile.icon)} alt="Minecraft" class="w-full h-full object-cover rounded-lg [image-rendering:pixelated]" />
				</div>
			</div>

			{#if tagColor}
				<div class="absolute left-0 top-0 h-full w-1 rounded-l-xl" style="background: {colorOptions.find((c) => c.value === tagColor)?.color ?? 'rgb(45, 212, 191)'};"></div>
			{/if}
			<div class="flex items-start justify-between">
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-2">
						<button
							class="grid h-5 w-5 shrink-0 place-items-center rounded transition-colors cursor-pointer"
							style="color: {profile.favorite ? 'rgb(250, 204, 21)' : 'rgb(var(--fg-subtle))'};"
							onclick={handleFavorite}
							aria-label={t("servers.favorite")}
						>
							<Star class="h-3.5 w-3.5 {profile.favorite ? 'fill-current' : ''}" />
						</button>
						<p class="truncate font-black text-white text-sm">{profile.name}</p>
						{#if isActive}
							<span class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-[9px] font-black uppercase flex items-center gap-1">
								<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> Ativa
							</span>
						{:else}
							<span class="px-2 py-0.5 rounded-full bg-white/5 text-white/50 text-[9px] font-bold">
								Pronta
							</span>
						{/if}
					</div>
					<p class="mt-0.5 flex items-center gap-1.5 text-xs text-white/50">
						<span class="inline-block mc-diamond-shape" style="width: 6px; height: 6px;"></span>
						{profile.mcVersion} · {profile.loader.toUpperCase()}
						{#if profile.loaderVersion}<span class="text-white/30">({profile.loaderVersion})</span>{/if}
					</p>
				</div>
				<button
					class="grid h-7 w-7 shrink-0 place-items-center rounded-md transition-all duration-150 text-white/40 hover:text-red-400 hover:bg-red-500/10 cursor-pointer"
					onclick={(e) => { e.stopPropagation(); onDelete?.(profile); }}
					aria-label={t("common.delete")}
					title="Excluir Instância"
				>
					<Trash2 class="h-3.5 w-3.5" />
				</button>
			</div>

			<div class="mt-2 flex flex-wrap gap-2 text-[11px]" style="color: rgb(var(--fg-subtle));">
				{#if profile.lastPlayed}
					<span class="flex items-center gap-1">
						<Clock class="h-3 w-3" />
						{formatTimeAgo(profile.lastPlayed)}
					</span>
				{/if}
				{#if profile.modCount !== undefined && profile.modCount > 0}
					<span class="flex items-center gap-1">
						<Box class="h-3 w-3" />
						{t("instances.modsCount", { count: profile.modCount })}
					</span>
				{/if}
				{#if profile.diskUsage}
					<span class="flex items-center gap-1">
						<HardDrive class="h-3 w-3" />
						{formatBytes(profile.diskUsage)}
					</span>
				{/if}
				<span class="flex items-center gap-1">
					<Download class="h-3 w-3" />
					{t("instances.ramCount", { ram: (profile.ramMb || 4096) / 1024 })}
				</span>
			</div>

			{#if profile.notes}
				<p class="mt-1 text-[11px] italic line-clamp-2" style="color: rgb(var(--fg-subtle));">{profile.notes}</p>
			{/if}

			<div class="mt-4 flex items-center justify-between border-t border-white/5 pt-3 relative">
				<div class="flex items-center gap-1.5">
					<button
						type="button"
						class="h-8 w-8 rounded-xl flex items-center justify-center text-white/50 hover:text-white bg-white/[0.03] hover:bg-white/10 border border-white/5 transition-all cursor-pointer shadow-sm active:scale-95"
						onclick={(e) => { e.stopPropagation(); onEdit?.(profile); }}
						aria-label={t("instances.edit")}
						title="Configurar Instância"
					>
						<Pencil class="h-3.5 w-3.5" />
					</button>
					<button
						type="button"
						class="h-8 w-8 rounded-xl flex items-center justify-center text-white/50 hover:text-white bg-white/[0.03] hover:bg-white/10 border border-white/5 transition-all cursor-pointer shadow-sm active:scale-95"
						onclick={(e) => { e.stopPropagation(); onOpenFolder?.(profile.id); }}
						aria-label={t("instances.openFolder")}
						title="Abrir Pasta de Arquivos"
					>
						<FolderOpen class="h-3.5 w-3.5" />
					</button>
					<InstanceActionsMenu
						bind:isOpen={menuOpen}
						actions={[
							{ label: "Duplicar Instância", icon: Copy, iconClass: "text-[#caa97c]", onClick: () => onDuplicate?.(profile.id) },
							{ label: "Capturas de Tela", icon: Image, iconClass: "text-sky-400", onClick: () => onScreenshots?.(profile.id) },
							{ label: "Anotações", icon: StickyNote, iconClass: "text-amber-400", onClick: () => onNotes?.(profile.id) },
							{ label: "Diagnóstico", icon: HeartPulse, iconClass: "text-emerald-400", onClick: () => onHealthCheck?.(profile.id) },
							{ divider: true, label: "", onClick: () => {} },
							{ label: "Excluir Instância", icon: Trash2, variant: "danger", onClick: () => onDelete?.(profile) },
						]}
					/>
				</div>

				<button
					type="button"
					class="flex items-center gap-2 rounded-xl px-6 py-2.5 text-xs font-black text-black transition-all hover:brightness-105 active:scale-95 shadow-[0_4px_20px_rgba(202,169,124,0.35)] cursor-pointer bg-gradient-to-r from-[#caa97c] via-[#ddbe93] to-[#ebd095] disabled:opacity-60"
					onclick={(e) => { e.stopPropagation(); onQuickPlay?.(profile); }}
					disabled={launchingInstanceId === profile.id}
				>
					{#if launchingInstanceId === profile.id}
						<Loader2 class="h-4 w-4 animate-spin text-black" />
						<span>Iniciando...</span>
					{:else}
						<Play class="h-4 w-4 fill-current stroke-[2.5]" />
						<span class="tracking-wide uppercase">JOGAR</span>
					{/if}
				</button>
			</div>
		</Card>
	</div>
{:else}
	<div
		class="grid grid-cols-[auto_1fr_8rem_6rem_5.5rem] items-center gap-3 rounded-md px-4 py-2.5 text-left transition-colors cursor-pointer"
		style="{isActive ? 'background: rgba(45, 212, 191, 0.05);' : ''}{tagColor ? ' border-left: 3px solid ' + (colorOptions.find((c) => c.value === tagColor)?.color ?? 'rgb(45, 212, 191)') + ';' : ''}"
		onclick={() => onSelect?.(profile.id)}
		onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelect?.(profile.id); }}
		role="button"
		tabindex="0"
	>
		{#if selectionMode}
			<button
				class="grid h-5 w-5 shrink-0 place-items-center rounded transition-colors"
				style="border: 1px solid {isSelected ? 'rgb(45, 212, 191)' : 'rgb(var(--border))'}; background: {isSelected ? 'rgba(45, 212, 191, 0.2)' : 'rgb(var(--bg-elevated))'};"
				onclick={(e) => { e.stopPropagation(); onToggleSelect?.(profile.id); }}
				role="checkbox"
				aria-checked={isSelected}
				aria-label={`${t("instances.select")}: ${profile.name}`}
			>
				{#if isSelected}
					<Check class="h-3 w-3" style="color: rgb(45, 212, 191);" />
				{/if}
			</button>
		{:else}
			<button
				class="grid h-5 w-5 shrink-0 place-items-center rounded transition-colors"
				style="color: {profile.favorite ? 'rgb(250, 204, 21)' : 'rgb(var(--fg-subtle))'};"
				onclick={handleFavorite}
				aria-label={t("servers.favorite")}
			>
				<Star class="h-3.5 w-3.5 {profile.favorite ? 'fill-current' : ''}" />
			</button>
		{/if}
		<div class="flex min-w-0 items-center gap-3">
			<img src={getIconSrc(profile.icon)} alt="Minecraft" class="w-5 h-5 rounded object-contain [image-rendering:pixelated] drop-shadow-xs shrink-0 bg-black/40 border border-white/10" />
			<span class="truncate text-sm font-medium">{profile.name}</span>
			{#if isActive}
				<span class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-[9px] font-black uppercase flex items-center gap-1">
					<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> Ativa
				</span>
			{/if}
		</div>
		<span class="flex items-center gap-1 truncate font-mono text-xs" style="color: rgb(var(--fg-muted));">
			<span class="inline-block mc-diamond-shape" style="width: 5px; height: 5px;"></span>
			{profile.mcVersion}
		</span>
		<span class="truncate text-xs font-bold uppercase" style="color: rgb(var(--fg-muted));">{profile.loader}</span>
		<div class="flex items-center justify-end gap-1.5">
			<button
				class="flex items-center gap-1.5 rounded-xl px-4 py-1.5 text-xs font-black text-black transition-all hover:brightness-105 active:scale-95 shadow-[0_2px_10px_rgba(202,169,124,0.3)] cursor-pointer bg-gradient-to-r from-[#caa97c] via-[#ddbe93] to-[#ebd095] disabled:opacity-75"
				onclick={(e) => { e.stopPropagation(); onQuickPlay?.(profile); }}
				disabled={launchingInstanceId === profile.id}
			>
				{#if launchingInstanceId === profile.id}
					<Loader2 class="h-3 w-3 animate-spin text-black" />
					<span>Iniciando...</span>
				{:else}
					<Play class="h-3 w-3 fill-current stroke-[2.5]" />
					<span class="tracking-wide uppercase">JOGAR</span>
				{/if}
			</button>
			<button
				class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-white/10 text-white/50 hover:text-white transition-all cursor-pointer"
				onclick={(e) => { e.stopPropagation(); onEdit?.(profile); }}
				aria-label={t("instances.edit")}
				title={t("instances.editTooltip")}
			>
				<Pencil class="h-3.5 w-3.5" />
			</button>
			<button
				class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-white/10 text-white/50 hover:text-white transition-all cursor-pointer"
				onclick={(e) => { e.stopPropagation(); onOpenFolder?.(profile.id); }}
				aria-label={t("instances.openFolder")}
			>
				<FolderOpen class="h-3.5 w-3.5" />
			</button>
			<button
				class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-red-500/20 text-white/50 hover:text-red-400 transition-all cursor-pointer"
				onclick={(e) => { e.stopPropagation(); onDelete?.(profile); }}
				aria-label={t("common.delete")}
			>
				<Trash2 class="h-3.5 w-3.5" />
			</button>
		</div>
	</div>
{/if}
