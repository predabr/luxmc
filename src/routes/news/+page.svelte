<script lang="ts">
import { translateUi as uiText, currentUiLocale } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import { onMount } from "svelte";
    import { newsState } from "$lib/stores/news.svelte";
    onMount(() => { void newsState.load(); });
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import ChangelogPanel from "$lib/components/ui/ChangelogPanel.svelte";
	import { 
		Newspaper, 
		Sparkles, 
		ExternalLink, 
		Radio, 
		Tag, 
		Calendar, 
		Flame, 
		ShieldCheck, 
		Cpu, 
		Globe, 
		CheckCircle2,
		Sliders
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";

	type LauncherArticle = {
		id: string;
		title: string;
		tag: string;
		tagColor: string;
		date: string;
		summary: string;
		highlights: string[];
		version?: string;
		link?: string;
		image: string;
	};

	const launcherNews: LauncherArticle[] = $derived([
        {
            id: "v3.0.0",
            title: uiText("release3.title"),
            tag: "3.0",
            tagColor: "text-brand-400 bg-brand-500/10 border-brand-500/30",
            date: "2026-10-05",
            version: "v3.0.0",
            image: "/news_1.jpg",
            summary: uiText("release3.summary"),
            highlights: ["appearance", "skins", "content", "social", "updates"].map(key => uiText(`release3.${key}`)),
            link: "https://github.com/predabr/luxmc/releases/tag/v3.0.0"
        },
		{
			id: "v2.0.2",
			title: uiText("ui.d05d6076df593be8"),
			tag: "Hotfix",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",
			date: "28 de Setembro, 2026",
			version: "v2.0.2",
			image: "/news_1.jpg",
			summary: uiText("ui.d9dd7981b343ed2b"),
			highlights: [
				uiText("ui.406b155788ea227f"),
				uiText("ui.4f9ae0da40497eaa"),
				uiText("ui.0090fb5de03ddb62"),
				uiText("ui.fabd1dc579c5aa69"),
				uiText("ui.cd9d119904babd9d"),
				uiText("ui.a0ff55212743dab4")
			],
			link: "https://luxmc-r92.pages.dev"
		},
		{
			id: "v2.0.1",
			title: uiText("ui.2e815bd562b6418c"),
			tag: "Oficial",
			tagColor: "text-amber-400 bg-amber-500/10 border-amber-500/30",
			date: "28 de Setembro, 2026",
			version: "v2.0.1",
			image: "/news_1.jpg",
			summary: uiText("ui.89e8aafdb7738026"),
			highlights: [
				uiText("ui.952e6935c7823d18"),
				"Suporte a Feral GameMode, MangoHud, GPU Dedicada e Gamescope no Linux e Windows",
				uiText("ui.0fda044922482302"),
				uiText("ui.bf90e88095e4ecd8"),
				uiText("ui.8a402f9eaeba58d2")
			],
			link: "https://luxmc-r92.pages.dev"
		},
		{
			id: "v2.0.0",
			title: uiText("ui.862b234fda958684"),
			tag: "Oficial",
			tagColor: "text-amber-400 bg-amber-500/10 border-amber-500/30",
			date: "27 de Setembro, 2026",
			version: "v2.0.0",
			image: "/news_1.jpg",
			summary: uiText("ui.1ac3832574384277"),
			highlights: [
				uiText("ui.a55ba9203f33a180"),
				uiText("ui.d052a2ae110003ce"),
				uiText("ui.a16708645bc7060c"),
				uiText("ui.bb4ee284878ded9b"),
				uiText("ui.19be1747280647d3")
			],
			link: "https://luxmc-r92.pages.dev/#destaques"
		},
		{
			id: "v1.9.2",
			title: uiText("ui.b0ec68f51f5ceac6"),
			tag: "Oficial",
			tagColor: "text-amber-400 bg-amber-500/10 border-amber-500/30",
			date: "20 de Setembro, 2026",
			version: "v1.9.2",
			image: "/news_1.jpg",
			summary: uiText("ui.b945108fc4ed55d8"),
			highlights: [
				uiText("ui.ab8debedbcd9234f"),
				uiText("ui.ffec3b78fca7316d"),
				uiText("ui.170ee654508e01af"),
				uiText("ui.8bbb8dcfb9e00775"),
				uiText("ui.2fd10527b9912607")
			],
			link: "https://luxmc-r92.pages.dev/#destaques"
		},
		{
			id: "v1.7.7",
			title: uiText("ui.285a4a6dc4b26f5d"),
			tag: "Oficial",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",
			date: "20 de Setembro, 2026",
			version: "v1.7.7",
			image: "/news_1.jpg",
			summary: uiText("ui.0425ec60f7de9623"),
			highlights: [
				uiText("ui.3c48838a82c3dde4"),
				uiText("ui.868680ab3adfd364"),
				uiText("ui.bdeb127f2c6e6e38"),
				uiText("ui.c8c8b2f76ad805aa"),
				uiText("ui.252cbc324c42d602")
			],
			link: "https://luxmc-r92.pages.dev/#destaques"
		},
		{
			id: "v1.7.6",
			title: uiText("ui.1c6695f79ab5078a"),
			tag: "Oficial",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",

			date: "20 de Setembro, 2026",
			version: "v1.7.6",
			image: "/news_1.jpg",
			summary: uiText("ui.096d2e6121729927"),
			highlights: [
				uiText("ui.369498434fdd39f4"),
				uiText("ui.0b98e28831100ee5"),
				uiText("ui.08f3db03d3231c8a"),
				uiText("ui.dc34523bec6cd82f"),
				"Consumo reduzido para menos de 85 MB de RAM em repouso"
			],
			link: "https://luxmc-r92.pages.dev/#destaques"
		},
		{
			id: "v1.7.5",
			title: uiText("ui.78a4a3c2094f520e"),
			tag: "Oficial",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",
			date: "19 de Setembro, 2026",
			version: "v1.7.5",
			image: "/news_1.jpg",
			summary: uiText("ui.cbc2ab335e51d833"),
			highlights: [
				uiText("ui.76f2b171999ace50"),
				uiText("ui.94d9592a021f722a"),
				uiText("ui.f7c8688222a21ebe"),
				uiText("ui.a9bed1602167fdcd"),
				uiText("ui.db03c8d72fc71e64"),
				uiText("ui.19da4f5cb0f4f908")
			],
			link: "https://luxmc-r92.pages.dev/#releases"
		},
		{
			id: "v1.7.4",
			title: uiText("ui.6aefc3733d8ff056"),
			tag: "Oficial",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",
			date: "18 de Setembro, 2026",
			version: "v1.7.4",
			image: "/news_1.jpg",
			summary: uiText("ui.2a1f490346077e93"),
			highlights: [
				uiText("ui.10d19756db00943d"),
				"Download concorrente com pool de 6 workers e espelhos de CDN resilientes (Edge, Mediafilez)",
				uiText("ui.c22f058a9b4e0a10"),
				uiText("ui.6ffca4c4426897c7"),
				uiText("ui.fdb88223360e88a2")
			],
			link: "https://luxmc-r92.pages.dev/#releases"
		},
		{
			id: "v1.7.3",
			title: uiText("ui.fa5205d0d40662c9"),
			tag: "Oficial",
			tagColor: "text-blue-400 bg-blue-500/10 border-blue-500/30",
			date: "18 de Setembro, 2026",
			version: "v1.7.3",
			image: "/news_1.jpg",
			summary: uiText("ui.85a9ff3016b236b4"),
			highlights: [
				uiText("ui.0ebe37a05e777add"),
				uiText("ui.1bc603cc01017a00"),
				uiText("ui.8fd495197fa8e1cc"),
				uiText("ui.1ddb12a965b032dc"),
				uiText("ui.0a47ae1305f6e136")
			],
			link: "https://luxmc-r92.pages.dev/#releases"
		},
		{
			id: "v1.7.2",
			title: uiText("ui.23de5f9b1fac72b8"),
			tag: "Ecossistema",
			tagColor: "text-sky-400 bg-sky-500/10 border-sky-500/30",
			date: "17 de Setembro, 2026",
			version: "v1.7.2",
			image: "/news_2.jpg",
			summary: uiText("ui.a333a28110216834"),
			highlights: [
				uiText("ui.915ae50070789650"),
				uiText("ui.422e2777c699f4ce"),
				uiText("ui.2febcf68a5a44202"),
				uiText("ui.a7a647bcdd203080")
			],
			link: "https://luxmc-r92.pages.dev"
		},
		{
			id: "v1.7.0",
			title: uiText("ui.2634d53dad9751d9"),
			tag: "Recurso",
			tagColor: "text-purple-400 bg-purple-500/10 border-purple-500/30",
			date: "14 de Setembro, 2026",
			version: "v1.7.0",
			image: "/news_3.jpg",
			summary: uiText("ui.aaff198025de76ab"),
			highlights: [
				uiText("ui.9c490ae8163316d8"),
				uiText("ui.942166f7fc3901ee"),
				uiText("ui.5158d73a8ba1db05")
			],
			link: "https://luxmc-r92.pages.dev"
		},
		{
			id: "v1.6.5",
			title: uiText("ui.f4c1660f63f6fb62"),
			tag: uiText("ui.50289b98d5904394"),
			tagColor: "text-amber-400 bg-amber-500/10 border-amber-500/30",
			date: "10 de Setembro, 2026",
			version: "v1.6.5",
			image: "/news_4.jpg",
			summary: uiText("ui.d53f3e061bd5f0e6"),
			highlights: [
				uiText("ui.35134f7e37f446dc"),
				uiText("ui.f549caaa9f79b580"),
				uiText("ui.bb37364bf8f81835")
			],
			link: "https://luxmc-r92.pages.dev"
		}
	]);

	let selectedTab = $state<"news" | "launcher" | "changelog">("news");
    const officialNews: LauncherArticle[] = $derived(selectedTab === "launcher" ? launcherNews : newsState.items.map(item => ({
        ...item, tag: item.category, tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30", highlights: [],
        date: new Date(`${item.date}T12:00:00Z`).toLocaleDateString(currentUiLocale(), { timeZone: "UTC" })
    })));

	async function openWebsite(url?: string) {
		const target = url || "https://luxmc-r92.pages.dev";
		try {
			await openUrl(target);
		} catch {
			window.open(target, "_blank");
		}
		toast("Abrindo no navegador...", "info");
	}
</script>

<div class="mx-auto flex h-full max-w-5xl flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-12">
	<!-- Header -->
	<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-fg/10 pb-5">
		<div class="flex items-center gap-3">
			<div class="p-2.5 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400">
				<Newspaper class="h-6 w-6" />
			</div>
			<div>
				<h1 class="text-2xl font-black text-fg tracking-tight flex items-center gap-2.5">
					{uiText("ui.a698fb88fcb732dd")}
					<span class="text-xs font-bold px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
						{uiText("ui.301b911d50083056")}
					</span>
				</h1>
				<p class="text-xs text-fg/50 mt-0.5">{uiText("ui.3ad808a60d02d8dc")}</p>
			</div>
		</div>

		<div class="flex items-center gap-3">
			<button
				type="button"
				onclick={() => openWebsite("https://luxmc-r92.pages.dev")}
				class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
			>
				<Globe class="w-4 h-4 text-emerald-400" />
				<span>{uiText("ui.969d6043dbac376f")}</span>
				<ExternalLink class="w-3 h-3 text-fg/40" />
			</button>
		</div>
	</header>

	<!-- Tabs -->
	<div class="flex items-center gap-2 p-1.5 rounded-2xl bg-bg-elevated border border-fg/10 w-fit">
		<button
			type="button"
			onclick={() => selectedTab = "news"}
			class="px-5 py-2 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {selectedTab === 'news' ? 'bg-fg/15 text-fg shadow-sm' : 'text-fg/60 hover:text-fg'}"
		>
			{uiText("ui.71651d2d36a58ca8")}{newsState.items.length})
		</button>
        <button type="button" onclick={() => selectedTab = "launcher"} class="px-5 py-2 rounded-xl text-xs font-bold transition-colors {selectedTab === 'launcher' ? 'bg-fg/15 text-fg' : 'text-fg/60 hover:text-fg'}">{uiText("ui.c95fb566afc11204")}{launcherNews.length})</button>
		<button
			type="button"
			onclick={() => selectedTab = "changelog"}
			class="px-5 py-2 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {selectedTab === 'changelog' ? 'bg-fg/15 text-fg shadow-sm' : 'text-fg/60 hover:text-fg'}"
		>
			{uiText("ui.287076c525481ecf")}
		</button>
	</div>

	{#if selectedTab !== "changelog"}
        {#if selectedTab === "news"}
            <div class="flex flex-wrap items-center justify-between gap-3 text-xs text-fg-muted" role="status">
                <span>{newsState.loading ? uiText("ui.d449a2691da0e880") : newsState.error || (newsState.offline ? uiText("ui.9198ccb48ed7bd1c") : uiText("ui.72ac8ad3be3dd73b", {arg0: (new Date(newsState.updatedAt).toLocaleTimeString(currentUiLocale()))}))}</span>
                <button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "luxmc-control" })} disabled={newsState.loading} onclick={() => newsState.load(true)}>{uiText("ui.8f1e47af1f2c6d7f")}</button>
            </div>
        {/if}
		<!-- Featured Article -->
		{#if officialNews.length > 0}
			{@const featured = officialNews[0]}
			<div class="rounded-3xl bg-bg-elevated border border-blue-500/30 shadow-2xl relative overflow-hidden group flex flex-col lg:flex-row">
				<div class="lg:w-1/2 h-56 lg:h-auto relative overflow-hidden">
					<img loading="lazy" decoding="async" 
						src={featured.image} 
						alt={featured.title} 
						class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
					/>
					<div class="absolute inset-0 bg-gradient-to-t lg:bg-gradient-to-r from-transparent via-bg-elevated/60 to-bg-elevated"></div>
				</div>

				<div class="lg:w-1/2 p-6 md:p-8 relative z-10 space-y-4 flex flex-col justify-between">
					<div class="space-y-3">
						<div class="flex items-center gap-2.5 flex-wrap">
							<span class="text-xs font-extrabold px-3 py-1 rounded-full {featured.tagColor}">
								{featured.tag}
							</span>
							<span class="text-xs font-mono font-bold bg-fg/10 px-2.5 py-0.5 rounded-full text-fg/90">
								{featured.version}
							</span>
							<span class="text-xs text-fg/40 flex items-center gap-1">
								<Calendar class="w-3.5 h-3.5" />
								{featured.date}
							</span>
						</div>

						<h2 class="text-xl md:text-2xl font-black text-fg group-hover:text-blue-300 transition-colors">
							{featured.title}
						</h2>

						<p class="text-xs md:text-sm text-fg/70 leading-relaxed">
							{featured.summary}
						</p>

						<div class="pt-2">
							<h3 class="text-xs font-bold uppercase tracking-wider text-blue-400 mb-2.5 flex items-center gap-1.5">
								<Sparkles class="w-3.5 h-3.5" /> {uiText("ui.f3bec1b7ad6df31a")}
							</h3>
							<div class="grid grid-cols-1 gap-2">
								{#each featured.highlights.slice(0, 3) as hl}
									<div class="flex items-start gap-2 text-xs text-fg/80 bg-fg/[0.03] border border-fg/5 p-2.5 rounded-xl">
										<CheckCircle2 class="w-3.5 h-3.5 text-blue-400 shrink-0 mt-0.5" />
										<span>{hl}</span>
									</div>
								{/each}
							</div>
						</div>
					</div>

					<div class="pt-3 flex items-center justify-end">
						<button
							type="button"
							onclick={() => openWebsite(featured.link)}
							class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-2" })}
						>
							<span>{uiText("ui.664e7082f85b56f5")}</span>
							<ExternalLink class="w-3.5 h-3.5" />
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- Other Updates Grid -->
		<div class="space-y-3 pt-2">
			<h3 class="text-xs font-bold uppercase tracking-wider text-fg/50">{uiText("ui.f1bca52a404d541e")}</h3>

			<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
				{#each officialNews.slice(1) as article (article.id)}
					<div class="rounded-3xl bg-bg-elevated border border-fg/10 hover:border-blue-500/30 transition-[color,background-color,border-color,box-shadow,transform,opacity] flex flex-col justify-between overflow-hidden shadow-lg group">
						<div class="w-full h-36 relative overflow-hidden bg-bg">
							<img loading="lazy" decoding="async" 
								src={article.image} 
								alt={article.title} 
								class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
							/>
							<div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-transparent to-transparent"></div>
							<div class="absolute top-3 left-3">
								<span class="text-[10px] font-bold px-2.5 py-0.5 rounded-full {article.tagColor}">
									{article.tag}
								</span>
							</div>
						</div>

						<div class="p-5 flex flex-col justify-between flex-1 gap-4">
							<div class="space-y-2">
								<div class="flex items-center justify-between text-[10px] text-fg/40">
									<span class="font-mono font-bold text-fg/50">{article.version}</span>
									<span>{article.date}</span>
								</div>

								<h4 class="text-xs font-bold text-fg group-hover:text-blue-300 transition-colors leading-snug line-clamp-2">
									{article.title}
								</h4>

								<p class="text-[11px] text-fg/50 leading-relaxed line-clamp-3">
									{article.summary}
								</p>
							</div>

							<div class="pt-2 border-t border-fg/5 flex items-center justify-between">
								<span class="text-[10px] text-fg/40">{uiText("ui.d3094029aa08ec2c")}</span>
								<button
									type="button"
									onclick={() => openWebsite(article.link)}
									class={launcherButton({ variant: "ghost", size: "sm", class: "flex items-center gap-1" })}
								>
									<span>{uiText("ui.62c95076dbeb3160")}</span>
									<ExternalLink class="w-3 h-3" />
								</button>
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>

	{:else}
		<Card>
			{#snippet header()}
				<div class="flex items-center gap-2">
					<Sliders class="h-4 w-4 text-emerald-400" />
					<span class="font-medium text-fg">{uiText("ui.dfd2f97727f4e2e8")}</span>
				</div>
			{/snippet}
			<ChangelogPanel />
		</Card>
	{/if}
</div>
