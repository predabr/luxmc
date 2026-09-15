<script lang="ts">
	import { Camera, Image, ExternalLink, ChevronLeft, ChevronRight } from "lucide-svelte";
	import { goto } from "$app/navigation";
	import { onMount } from "svelte";

	type Screenshot = {
		id: string;
		url: string;
		timestamp: string;
	};

	let screenshots = $state<Screenshot[]>([]);
	let loading = $state(true);
	let scrollEl = $state<HTMLDivElement | null>(null);

	onMount(async () => {
		try {
			const saved = localStorage.getItem("luxmc_screenshots");
			if (saved) {
				const data = JSON.parse(saved);
				if (Array.isArray(data)) {
					screenshots = data.slice(0, 10).map((s: any) => ({
						id: String(s.id || Date.now()),
						url: s.url || s.path || "",
						timestamp: s.timestamp ? new Date(s.timestamp).toLocaleDateString("pt-BR", { day: "2-digit", month: "short" }) : ""
					}));
				}
			}
		} catch {
			// ignore
		} finally {
			loading = false;
		}
	});

	function scroll(dir: "left" | "right") {
		if (!scrollEl) return;
		const amount = 220;
		scrollEl.scrollBy({ left: dir === "left" ? -amount : amount, behavior: "smooth" });
	}
</script>

<section>
	<div class="flex items-center justify-between mb-3">
		<h2 class="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-2">
			<Camera class="w-3.5 h-3.5 text-[#caa97c]" />
			Capturas de Tela
		</h2>
		{#if screenshots.length > 0}
			<div class="flex items-center gap-1">
				<button
					type="button"
					class="p-1 rounded-lg bg-white/5 hover:bg-white/10 text-white/40 hover:text-white transition-all cursor-pointer"
					onclick={() => scroll("left")}
				>
					<ChevronLeft class="w-3 h-3" />
				</button>
				<button
					type="button"
					class="p-1 rounded-lg bg-white/5 hover:bg-white/10 text-white/40 hover:text-white transition-all cursor-pointer"
					onclick={() => scroll("right")}
				>
					<ChevronRight class="w-3 h-3" />
				</button>
				<a
					href="/screenshots"
					class="text-xs text-[#caa97c] hover:underline font-bold ml-1"
				>
					Ver Todas
				</a>
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="flex gap-3 overflow-hidden">
			{#each [1, 2, 3] as _}
				<div class="h-28 w-40 rounded-2xl bg-[#18191c] border border-white/5 animate-pulse shrink-0"></div>
			{/each}
		</div>
	{:else if screenshots.length === 0}
		<div class="rounded-2xl bg-[#18191c] border border-white/5 p-6 flex flex-col items-center justify-center text-center">
			<div class="w-10 h-10 rounded-xl bg-white/5 border border-white/10 flex items-center justify-center mb-3">
				<Image class="w-5 h-5 text-white/30" />
			</div>
			<p class="text-xs font-semibold text-white/50 mb-1">Nenhuma captura ainda</p>
			<p class="text-[10px] text-white/35">Suas screenshots aparecerão aqui automaticamente</p>
		</div>
	{:else}
		<div
			bind:this={scrollEl}
			class="flex gap-3 overflow-x-auto custom-scrollbar pb-1 scroll-smooth"
			style="scrollbar-width: none; -ms-overflow-style: none;"
		>
			{#each screenshots as shot (shot.id)}
				<div class="relative h-28 w-40 rounded-2xl overflow-hidden bg-[#18191c] border border-white/5 hover:border-[#caa97c]/30 transition-all shrink-0 group cursor-pointer">
					{#if shot.url}
						<img src={shot.url} alt="Screenshot" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" />
					{:else}
						<div class="w-full h-full flex items-center justify-center">
							<Camera class="w-6 h-6 text-white/15" />
						</div>
					{/if}
					<div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
					{#if shot.timestamp}
						<span class="absolute bottom-1.5 left-2 text-[9px] font-bold text-white/70 bg-black/40 px-1.5 py-0.5 rounded-md backdrop-blur-sm">
							{shot.timestamp}
						</span>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</section>
