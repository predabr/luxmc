<script lang="ts">
import { translateUi as uiText, currentUiLocale } from "$lib/i18n/useTranslation.svelte";
    import { z } from "zod";
	import { Newspaper, ExternalLink, Tag, ArrowRight } from "lucide-svelte";
	import { onMount, untrack } from "svelte";

	type Release = {
		id: string;
		title: string;
		body: string;
		publishedAt: string;
		tag: string;
		url: string;
	};

	const defaultNews: Release[] = $derived([
        {
            id: "luxmc-v3-0-0",
            title: uiText("release3.title"),
            body: uiText("release3.summary"),
            publishedAt: "2026-10-05",
            tag: "v3.0.0",
            url: "/news"
        },
		{
			id: "pale-garden-1-21-5",
			title: uiText("ui.a0dc4bdd46f44954"),
			body: uiText("ui.e4e24ba39a1d2b8c"),
			publishedAt: "Set 2026",
			tag: "Minecraft 1.21.5",
			url: "/news"
		},
		{
			id: "modrinth-api-v3",
			title: uiText("ui.0b175611a05abdd1"),
			body: uiText("ui.741f8d7308bc3f93"),
			publishedAt: "Set 2026",
			tag: "Modrinth",
			url: "https://modrinth.com"
		},
		{
			id: "sodium-iris-update",
			title: uiText("ui.755904314ac4e964"),
			body: uiText("ui.b3ce699de94c41a1"),
			publishedAt: "Set 2026",
			tag: uiText("ui.ce274629c3f2e125"),
			url: "https://modrinth.com/mod/sodium"
		},
		{
			id: "luxmc-v2-0-2",
			title: uiText("ui.08cc6505298cb2a2"),
			body: uiText("ui.1c55c5b0af246549"),
			publishedAt: "Hoje",
			tag: "v2.0.2",
			url: "/news"
		},
		{
			id: "luxmc-v2-0-1",
			title: uiText("ui.c8f273aadcdff865"),
			body: uiText("ui.18e4119c21f0f77e"),
			publishedAt: "Hoje",
			tag: "v2.0.1",
			url: "/news"
		},
		{
			id: "luxmc-v2-0-0",
			title: uiText("ui.18136eecf74dab47"),
			body: uiText("ui.f5429eaba599b0a1"),
			publishedAt: "Ontem",
			tag: "v2.0.0",
			url: "/news"
		},
		{
			id: "luxmc-v1-7-6",
			title: uiText("ui.5b50668b7ba4294b"),
			body: uiText("ui.beb0db02e674fe92"),
			publishedAt: "20 Set",
			tag: "v1.7.6",
			url: "/news"
		},

		{
			id: "luxmc-v1-7-5",
			title: uiText("ui.baf3262cf3b7e92c"),
			body: uiText("ui.c9346a5d54ec7869"),
			publishedAt: "19 Set",
			tag: "v1.7.5",
			url: "/news"
		}
	]);

	let releases = $state<Release[]>(untrack(() => defaultNews));
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
					publishedAt: new Date(r.published_at).toLocaleDateString(currentUiLocale(), {
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
			{uiText("ui.27c583af31e6c157")}
		</h2>
		<a
			href="/news"
			class="text-xs text-emerald-400 hover:text-emerald-300 hover:underline font-bold flex items-center gap-1"
		>
			{uiText("ui.0e5eca22a3ced66b")} <ArrowRight class="w-3 h-3" />
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
			<p class="text-xs text-fg/40">{uiText("ui.0c3ed25494067a4f")}</p>
			<a
				href="https://github.com/predabr/luxmc/releases"
				target="_blank"
				rel="noopener noreferrer"
				class="text-[10px] text-emerald-400 hover:underline font-bold mt-1 inline-block"
			>
				{uiText("ui.5c3f24bca8de14ee")}
			</a>
		</div>
	{:else}
		<div class="flex flex-col gap-2">
			{#each releases as release (release.id)}
				<a
					href={release.url}
					target="_blank"
					rel="noopener noreferrer"
					class="group rounded-2xl bg-bg-elevated border border-fg/5 hover:border-emerald-500/30 p-3.5 transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:bg-bg-subtle cursor-pointer"
				>
					<div class="flex items-center justify-between gap-2">
						<div class="flex items-center gap-2 min-w-0">
							<span class="text-[9px] font-bold px-1.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shrink-0 font-mono">
								{release.tag}
							</span>
							<span class="text-[10px] text-fg/30 shrink-0">{release.publishedAt}</span>
						</div>
						<ArrowRight class="w-3 h-3 text-fg/20 group-hover:text-emerald-400 -rotate-45 transition-[color,background-color,border-color,box-shadow,transform,opacity] shrink-0" />
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
