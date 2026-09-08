<script lang="ts">
	import type { HTMLInputAttributes } from "svelte/elements";
	import type { Snippet } from "svelte";

	type Props = {
		value?: string;
		placeholder?: string;
		label?: string;
		hint?: string;
		error?: string;
		leadingIcon?: Snippet;
		trailingIcon?: Snippet;
		class?: string;
	} & Omit<HTMLInputAttributes, "value" | "class" | "children">;

	let {
		value = $bindable(""),
		placeholder,
		label,
		hint,
		error,
		leadingIcon,
		trailingIcon,
		class: klass = "",
		...rest
	}: Props = $props();
</script>

<label class="flex flex-col gap-1.5 {klass}">
	{#if label}
		<span class="text-xs font-medium text-fg-muted">{label}</span>
	{/if}
	<span
		class="flex h-9 items-center gap-2 rounded-md border border-border bg-bg-subtle px-3 text-sm transition-colors focus-within:border-brand-500 focus-within:ring-1 focus-within:ring-brand-500"
	>
		{#if leadingIcon}
			<span class="text-fg-muted">{@render leadingIcon()}</span>
		{/if}
		<input
			class="h-full w-full bg-transparent outline-none placeholder:text-fg-subtle"
			bind:value
			{placeholder}
			{...rest}
		/>
		{#if trailingIcon}
			<span class="text-fg-muted">{@render trailingIcon()}</span>
		{/if}
	</span>
	{#if error}
		<span class="text-xs text-danger">{error}</span>
	{:else if hint}
		<span class="text-xs text-fg-subtle">{hint}</span>
	{/if}
</label>
