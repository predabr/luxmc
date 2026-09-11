<script lang="ts">
	import { Signal } from "lucide-svelte";

	type Props = {
		ping: number;
		loading?: boolean;
	};

	let { ping, loading = false }: Props = $props();
</script>

<span class="text-emerald-400 font-mono font-bold text-xs flex items-center gap-1 justify-end">
	<Signal class="w-3.5 h-3.5 {loading ? 'animate-pulse' : ''}" />
	{ping}ms
</span>
