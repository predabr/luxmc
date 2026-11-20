<script lang="ts">
	import LoaderLogo from "$lib/components/mods/LoaderLogo.svelte";

	let { loader }: { loader: string } = $props();
	const color = $derived(({ fabric: "bg-sky-500/15 border-sky-500/30 text-sky-300", forge: "bg-amber-500/15 border-amber-500/30 text-amber-300", neoforge: "bg-orange-500/15 border-orange-500/30 text-orange-300", quilt: "bg-purple-500/15 border-purple-500/30 text-purple-300" } as Record<string, string>)[loader.toLowerCase()] || "bg-success/15 border-success/30 text-success");
</script>

<span class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[10px] font-bold uppercase tracking-wide {color}">
	<LoaderLogo {loader} size={11} />
	{loader}
</span>
