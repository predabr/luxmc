<script lang="ts">
	import { fade } from "svelte/transition";
	import {
		Copy,
		Image,
		StickyNote,
		HeartPulse,
		FolderOpen,
		FolderTree,
		Trash2,
		Download,
		MoreVertical
	} from "lucide-svelte";

	interface ActionItem {
		label: string;
		icon?: typeof Copy;
		iconClass?: string;
		variant?: "default" | "danger";
		onClick: () => void;
		divider?: boolean;
	}

	let {
		actions,
		isOpen = $bindable(false)
	}: {
		actions: ActionItem[];
		isOpen: boolean;
	} = $props();
</script>

<div class="relative">
	<button
		type="button"
		class="h-8 w-8 rounded-xl flex items-center justify-center text-white/50 hover:text-white bg-white/[0.03] hover:bg-white/10 border border-white/5 transition-all cursor-pointer shadow-sm active:scale-95 {isOpen ? 'bg-[#caa97c]/20 text-[#caa97c] border-[#caa97c]/40' : ''}"
		onclick={(e) => { e.stopPropagation(); isOpen = !isOpen; }}
		title="Mais Opções"
	>
		<MoreVertical class="h-3.5 w-3.5" />
	</button>

	{#if isOpen}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="fixed inset-0 z-40" onclick={(e) => { e.stopPropagation(); isOpen = false; }}></div>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="absolute bottom-10 left-0 z-50 w-48 bg-[#181920] border border-white/10 rounded-2xl shadow-2xl p-1.5 space-y-0.5 text-xs font-semibold text-white/80"
			onclick={(e) => e.stopPropagation()}
			transition:fade={{ duration: 150 }}
		>
			{#each actions as action, i}
				{#if action.divider}
					<div class="border-t border-white/5 my-1"></div>
				{/if}
				<button
					type="button"
					class="w-full flex items-center gap-2 px-3 py-2 rounded-xl transition-colors cursor-pointer text-left {action.variant === 'danger' ? 'hover:bg-red-500/15 text-red-400 hover:text-red-300' : 'hover:bg-white/5 hover:text-white'}"
					onclick={() => { isOpen = false; action.onClick(); }}
				>
					{#if action.icon}
						<action.icon class="w-3.5 h-3.5 {action.iconClass ?? ''}" />
					{/if}
					<span>{action.label}</span>
				</button>
			{/each}
		</div>
	{/if}
</div>
