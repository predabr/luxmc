<script lang="ts">
	import { Newspaper, ExternalLink, Tag, ArrowRight } from "lucide-svelte";
	import { onMount } from "svelte";

	type Release = {
		id: string;
		title: string;
		body: string;
		publishedAt: string;
		tag: string;
		url: string;
	};

	let releases = $state<Release[]>([]);
	let loading = $state(true);
	let error = $state(false);

	onMount(async () => {
		try {
			const res = await fetch("https://api.github.com/repos/predabr/luxmc/releases?per_page=5");
			if (!res.ok) throw new Error("Failed to fetch");
			const data = await res.json();
			releases = data.map((r: any) => ({
				id: String(r.id),
				title: r.name || r.tag_name,
				body: (r.body || "").slice(0, 160).replace(/[#*`\n]/g, " ").trim(),
				publishedAt: new Date(r.published_at).toLocaleDateString("pt-BR", {
					day: "2-digit",
					month: "short"
				}),
				tag: r.tag_name,
				url: r.html_url
			}));
		} catch {
			error = true;
		} finally {
			loading = false;
		}
	});
</script>

<section>
	<div class="flex items-center justify-between mb-3">
		<h2 class="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-2">
			<Newspaper class="w-3.5 h-3.5 text-emerald-400" />
			Notícias & Patch Notes
		</h2>
		<a
			href="https://github.com/predabr/luxmc/releases"
			target="_blank"
			rel="noopener noreferrer"
			class="text-xs text-emerald-400 hover:text-emerald-300 hover:underline font-bold flex items-center gap-1"
		>
			Ver Todas <ExternalLink class="w-3 h-3" />
		</a>
	</div>

	{#if loading}
		<div class="space-y-2">
			{#each [1, 2, 3] as _}
				<div class="h-16 rounded-2xl bg-[#18191c] border border-white/5 animate-pulse"></div>
			{/each}
		</div>
	{:else if error}
		<div class="rounded-2xl bg-[#18191c] border border-white/5 p-5 text-center">
			<p class="text-xs text-white/40">Não foi possível carregar as notícias.</p>
			<a
				href="https://github.com/predabr/luxmc/releases"
				target="_blank"
				rel="noopener noreferrer"
				class="text-[10px] text-emerald-400 hover:underline font-bold mt-1 inline-block"
			>
				Abrir no GitHub
			</a>
		</div>
	{:else}
		<div class="flex flex-col gap-2">
			{#each releases as release (release.id)}
				<a
					href={release.url}
					target="_blank"
					rel="noopener noreferrer"
					class="group rounded-2xl bg-[#18191c] border border-white/5 hover:border-emerald-500/30 p-3.5 transition-all hover:bg-[#1c1d22] cursor-pointer"
				>
					<div class="flex items-center justify-between gap-2">
						<div class="flex items-center gap-2 min-w-0">
							<span class="text-[9px] font-bold px-1.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shrink-0 font-mono">
								{release.tag}
							</span>
							<span class="text-[10px] text-white/30 shrink-0">{release.publishedAt}</span>
						</div>
						<ArrowRight class="w-3 h-3 text-white/20 group-hover:text-emerald-400 -rotate-45 transition-all shrink-0" />
					</div>
					<h3 class="text-xs font-bold text-white mt-1.5 group-hover:text-emerald-400 transition-colors truncate">
						{release.title}
					</h3>
					{#if release.body}
						<p class="text-[10px] text-white/40 mt-0.5 line-clamp-2 leading-relaxed">
							{release.body}
						</p>
					{/if}
				</a>
			{/each}
		</div>
	{/if}
</section>
