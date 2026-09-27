<script lang="ts">
	import { Tooltip } from "bits-ui";
	import type { Snippet } from "svelte";

	let {
		text,
		side = "top",
		delayDuration = 200,
		class: klass = "",
		children
	}: {
		text: string;
		side?: "top" | "bottom" | "left" | "right";
		delayDuration?: number;
		class?: string;
		children: Snippet;
	} = $props();
</script>

<Tooltip.Provider {delayDuration}>
	<Tooltip.Root>
		<Tooltip.Trigger>
			{#snippet child({ props })}
				<span {...props} class="inline-flex {klass}">
					{@render children()}
				</span>
			{/snippet}
		</Tooltip.Trigger>
		<Tooltip.Portal>
			<Tooltip.Content
				{side}
				sideOffset={6}
				class="z-50 overflow-hidden rounded-xl bg-bg-elevated/95 px-3 py-1.5 text-xs font-semibold text-fg shadow-xl backdrop-blur-xl border border-fg/10"
			>
				{text}
				<Tooltip.Arrow class="fill-bg-elevated/95" />
			</Tooltip.Content>
		</Tooltip.Portal>
	</Tooltip.Root>
</Tooltip.Provider>
