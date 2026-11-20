<script lang="ts">
	const LOGOS: Record<string, string> = {
		fabric: "/branding/loaders/fabric.png",
		quilt: "/branding/loaders/quilt.png",
		forge: "/branding/loaders/forge.png",
		neoforge: "/branding/loaders/neoforge.png"
	};

	let { loader, size = 14 }: { loader: string; size?: number } = $props();

	const src = $derived(LOGOS[loader.trim().toLowerCase().replace(/[\s-]/g, "")] ?? "");
</script>

{#if src}
	<img
		src={src}
		alt=""
		width={size}
		height={size}
		loading="lazy"
		decoding="async"
		class="loader-logo"
		style="width: {size}px; height: {size}px;"
	/>
{/if}

<style>
	.loader-logo {
		flex: none;
		border-radius: 4px;
		object-fit: cover;
		box-shadow: 0 0 0 1px rgb(255 255 255 / 0.08);
	}
</style>
