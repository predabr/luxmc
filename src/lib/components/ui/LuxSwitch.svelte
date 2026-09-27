<script lang="ts">
	import { Switch } from "bits-ui";

	let {
		checked = $bindable(false),
		disabled = false,
		id = "",
		class: klass = "",
		onchange
	}: {
		checked?: boolean;
		disabled?: boolean;
		id?: string;
		class?: string;
		onchange?: (val: boolean) => void;
	} = $props();

	function handleChange(val: boolean) {
		checked = val;
		onchange?.(val);
	}
</script>

<Switch.Root
	{id}
	{checked}
	{disabled}
	onCheckedChange={handleChange}
	class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-brand-500 data-[state=unchecked]:bg-fg/20 {klass}"
>
	<Switch.Thumb
		class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0"
	/>
</Switch.Root>
