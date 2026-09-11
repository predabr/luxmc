<script lang="ts">
	import { fade } from "svelte/transition";
	import Card from "$lib/components/ui/Card.svelte";
	import InstanceCard from "./InstanceCard.svelte";
	import { Boxes } from "lucide-svelte";
	import type { Profile } from "$lib/stores/profiles.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	interface Props {
		instances: Profile[];
		viewMode?: "grid" | "list";
		searchQuery?: string;
		selectionMode?: boolean;
		selectedIds?: Set<string>;
		instanceColors?: Record<string, string>;
		launchingInstanceId?: string | null;
		colorOptions?: Array<{ value: string; color: string }>;
		getIconSrc: (icon?: string) => string;
		formatTimeAgo: (ts: number) => string;
		formatBytes: (bytes: number) => string;
		activeId?: string | null;
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
	}

	let {
		instances,
		viewMode = "grid",
		searchQuery = "",
		selectionMode = false,
		selectedIds = new Set(),
		instanceColors = {},
		launchingInstanceId = null,
		colorOptions = [],
		getIconSrc,
		formatTimeAgo,
		formatBytes,
		activeId = null,
		onSelect,
		onToggleSelect,
		onQuickPlay,
		onEdit,
		onOpenFolder,
		onDelete,
		onDuplicate,
		onScreenshots,
		onNotes,
		onHealthCheck
	}: Props = $props();
</script>

{#if instances.length === 0}
	<Card>
		<div class="flex flex-col items-center gap-3 py-8 text-center">
			<div class="relative mb-2">
				<div class="mc-creeper-face h-16 w-16"></div>
			</div>
			<p class="text-sm" style="color: rgb(var(--fg-muted));">
				{#if searchQuery}
					{t("instances.noInstancesMatch", { query: searchQuery })}
				{:else}
					{t("instances.createFirst")}
				{/if}
			</p>
		</div>
	</Card>
{:else if viewMode === "grid"}
	<div in:fade={{ duration: 150 }} class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
		{#each instances as p (p.id)}
			<InstanceCard
				profile={p}
				isActive={activeId === p.id}
				tagColor={instanceColors[p.id]}
				{colorOptions}
				{getIconSrc}
				{formatTimeAgo}
				{formatBytes}
				{selectionMode}
				isSelected={selectedIds.has(p.id)}
				{launchingInstanceId}
				{onSelect}
				{onToggleSelect}
				{onQuickPlay}
				{onEdit}
				{onOpenFolder}
				{onDelete}
				{onDuplicate}
				{onScreenshots}
				{onNotes}
				{onHealthCheck}
				viewMode="grid"
			/>
		{/each}
	</div>
{:else}
	<div in:fade={{ duration: 150 }} class="flex flex-col gap-1">
		<div class="grid grid-cols-[auto_1fr_8rem_6rem_5.5rem] gap-3 border-b px-4 py-2 text-xs font-medium" style="border-color: rgb(var(--border)); color: rgb(var(--fg-subtle));">
			<span></span>
			<span>{t("instances.sortName")}</span>
			<span>{t("instances.sortVersion")}</span>
			<span>{t("instances.loader")}</span>
			<span class="text-right">{t("instances.actions")}</span>
		</div>
		{#each instances as p (p.id)}
			<InstanceCard
				profile={p}
				isActive={activeId === p.id}
				tagColor={instanceColors[p.id]}
				{colorOptions}
				{getIconSrc}
				{formatTimeAgo}
				{formatBytes}
				{selectionMode}
				isSelected={selectedIds.has(p.id)}
				{launchingInstanceId}
				{onSelect}
				{onToggleSelect}
				{onQuickPlay}
				{onEdit}
				{onOpenFolder}
				{onDelete}
				{onDuplicate}
				{onScreenshots}
				{onNotes}
				{onHealthCheck}
				viewMode="list"
			/>
		{/each}
	</div>
{/if}
