<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";

	onMount(() => {
		goto("/");
	});
</script>

<div class="flex items-center justify-center min-h-[60vh]">
	<div class="h-8 w-8 border-2 border-emerald-400 border-t-transparent rounded-full animate-spin"></div>
</div>

