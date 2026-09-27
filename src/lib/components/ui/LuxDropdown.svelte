<script lang="ts">
	import { DropdownMenu } from "bits-ui";
	import type { Snippet } from "svelte";

	export interface DropdownMenuItem {
		id: string;
		label: string;
		icon?: any;
		danger?: boolean;
		disabled?: boolean;
		separator?: boolean;
		action: () => void;
	}

	let {
		items,
		align = "end",
		class: klass = "",
		trigger
	}: {
		items: DropdownMenuItem[];
		align?: "start" | "center" | "end";
		class?: string;
		trigger: Snippet;
	} = $props();
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger class="outline-none {klass}">
		{@render trigger()}
	</DropdownMenu.Trigger>
	<DropdownMenu.Portal>
		<DropdownMenu.Content
			{align}
			sideOffset={6}
			class="z-50 min-w-[180px] overflow-hidden rounded-2xl bg-bg-elevated/95 p-1.5 shadow-2xl backdrop-blur-2xl border border-fg/10 focus:outline-none"
		>
			{#each items as item (item.id)}
				{#if item.separator}
					<DropdownMenu.Separator class="my-1 h-px bg-fg/10" />
				{/if}
				<DropdownMenu.Item
					disabled={item.disabled}
					onSelect={() => item.action()}
					class="relative flex cursor-pointer select-none items-center gap-2 rounded-xl px-3 py-2 text-xs font-semibold outline-none transition-colors data-[disabled]:pointer-events-none data-[disabled]:opacity-40 {item.danger ? 'text-red-400 hover:bg-red-500/10 focus:bg-red-500/10' : 'text-fg/80 hover:bg-fg/5 hover:text-fg focus:bg-fg/5 focus:text-fg'}"
				>
					{#if item.icon}
						{@const IconComponent = item.icon}
						<IconComponent class="h-3.5 w-3.5 shrink-0" />
					{/if}
					<span>{item.label}</span>
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Portal>
</DropdownMenu.Root>
