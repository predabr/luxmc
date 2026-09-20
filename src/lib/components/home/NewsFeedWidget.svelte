<script lang="ts">
    import { z } from "zod";
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

	const defaultNews: Release[] = [
		{
			id: "pale-garden-1-21-5",
			title: "Minecraft Drop: The Pale Garden & The Creaking",
			body: "Novo bioma Pale Garden com madeira Pale Oak, mob Creaking e novos blocos de resina.",
			publishedAt: "Set 2026",
			tag: "Minecraft 1.21.5",
			url: "/news"
		},
		{
			id: "modrinth-api-v3",
			title: "Modrinth v3: Servidores CDN no Brasil",
			body: "Downloads de modpacks e shaders até 4x mais rápidos com novo nó em São Paulo.",
			publishedAt: "Set 2026",
			tag: "Modrinth",
			url: "https://modrinth.com"
		},
		{
			id: "sodium-iris-update",
			title: "Sodium 0.6: OpenGL Multi-Draw & Vulkan",
			body: "Salto de taxa de quadros e renderização suave a 144+ FPS no Linux e Windows.",
			publishedAt: "Set 2026",
			tag: "Desempenho",
			url: "https://modrinth.com/mod/sodium"
		},
		{
			id: "luxmc-v1-7-6",
			title: "Luxmc v1.7.6: Barra Sincronizada & NeoForge ATM10",
			body: "Barra lateral translúcida conectada ao wallpaper, correções gráficas e performance extrema.",
			publishedAt: "20 Set",
			tag: "v1.7.6",
			url: "/news"
		},
		{
			id: "luxmc-v1-7-5",
			title: "Luxmc v1.7.5: Aceleração GPU & Modo Streamer",
			body: "Interface ultra smooth com WebKit GPU, suporte a Espanhol e privacidade avançada.",
			publishedAt: "19 Set",
			tag: "v1.7.5",
			url: "/news"
		}
	];

	let releases = $state<Release[]>(defaultNews);
	let loading = $state(false);
	let error = $state(false);

	onMount(async () => {
		try {
			const res = await fetch("https://api.github.com/repos/predabr/luxmc/releases?per_page=5");
			if (!res.ok) return;
			const data = z.array(z.object({ id: z.number(), name: z.string().nullable(), tag_name: z.string(), body: z.string().nullable(), published_at: z.string(), html_url: z.string().url() })).parse(await res.json());
			if (data.length > 0) {
				const fetched = data.map((r) => ({
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
				releases = [...fetched.slice(0, 2), ...defaultNews.slice(0, 2)];
			}
		} catch {}
	});
</script>

<section>
	<div class="flex items-center justify-between mb-3">
		<h2 class="text-xs font-bold text-fg uppercase tracking-wider flex items-center gap-2">
			<Newspaper class="w-3.5 h-3.5 text-emerald-400" />
			Notícias & Atualizações
		</h2>
		<a
			href="/news"
			class="text-xs text-emerald-400 hover:text-emerald-300 hover:underline font-bold flex items-center gap-1"
		>
			Ver Central <ArrowRight class="w-3 h-3" />
		</a>
	</div>

	{#if loading}
		<div class="space-y-2">
			{#each [1, 2, 3] as _}
				<div class="h-16 rounded-2xl bg-bg-elevated border border-fg/5 animate-pulse"></div>
			{/each}
		</div>
	{:else if error}
		<div class="rounded-2xl bg-bg-elevated border border-fg/5 p-5 text-center">
			<p class="text-xs text-fg/40">Não foi possível carregar as notícias.</p>
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
					class="group rounded-2xl bg-bg-elevated border border-fg/5 hover:border-emerald-500/30 p-3.5 transition-all hover:bg-bg-subtle cursor-pointer"
				>
					<div class="flex items-center justify-between gap-2">
						<div class="flex items-center gap-2 min-w-0">
							<span class="text-[9px] font-bold px-1.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shrink-0 font-mono">
								{release.tag}
							</span>
							<span class="text-[10px] text-fg/30 shrink-0">{release.publishedAt}</span>
						</div>
						<ArrowRight class="w-3 h-3 text-fg/20 group-hover:text-emerald-400 -rotate-45 transition-all shrink-0" />
					</div>
					<h3 class="text-xs font-bold text-fg mt-1.5 group-hover:text-emerald-400 transition-colors truncate">
						{release.title}
					</h3>
					{#if release.body}
						<p class="text-[10px] text-fg/40 mt-0.5 line-clamp-2 leading-relaxed">
							{release.body}
						</p>
					{/if}
				</a>
			{/each}
		</div>
	{/if}
</section>
