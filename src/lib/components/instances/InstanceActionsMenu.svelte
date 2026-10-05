<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { DropdownMenu } from "bits-ui";
	import { MoreVertical, type Icon } from "lucide-svelte";

	interface ActionItem {
		label: string;
		icon?: typeof Icon;
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
		isOpen?: boolean;
	} = $props();
</script>

<DropdownMenu.Root bind:open={isOpen}>
	<DropdownMenu.Trigger
		class="h-8 w-8 rounded-xl flex items-center justify-center text-fg/50 hover:text-fg bg-fg/[0.03] hover:bg-fg/10 border border-fg/5 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer shadow-sm active:scale-[0.98] data-[state=open]:bg-brand-400/20 data-[state=open]:text-brand-400 data-[state=open]:border-brand-400/40"
		title={uiText("ui.98a492c508e9c984")}
	>
		<MoreVertical class="h-3.5 w-3.5" />
	</DropdownMenu.Trigger>

	<DropdownMenu.Portal>
		<DropdownMenu.Content
			align="end"
			sideOffset={6}
			class="z-50 min-w-48 overflow-hidden rounded-2xl bg-bg-elevated/95 p-1.5 shadow-2xl backdrop-blur-2xl border border-fg/10 space-y-0.5 text-xs font-semibold text-fg/80 focus:outline-none"
		>
			{#each actions as action (action.label)}
				{#if action.divider}
					<DropdownMenu.Separator class="border-t border-fg/5 my-1" />
				{/if}
				<DropdownMenu.Item
					onSelect={() => action.onClick()}
					class="w-full flex items-center gap-2 px-3 py-2 rounded-xl transition-colors cursor-pointer text-left outline-none {action.variant === 'danger' ? 'hover:bg-red-500/15 text-red-400 hover:text-red-300 focus:bg-red-500/15 focus:text-red-300' : 'hover:bg-fg/5 hover:text-fg focus:bg-fg/5 focus:text-fg'}"
				>
					{#if action.icon}
						{@const IconComponent = action.icon}
						<IconComponent class="w-3.5 h-3.5 {action.iconClass ?? ''}" />
					{/if}
					<span>{action.label}</span>
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Portal>
</DropdownMenu.Root>
