<script lang="ts">
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import ChangelogPanel from "$lib/components/ui/ChangelogPanel.svelte";
	import { Newspaper, MessageSquare, Rss } from "lucide-svelte";
	import { onMount } from "svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	type FeedItem = { title: string; link: string; pubDate: string; source: string };

	let items = $state<FeedItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const feeds: Array<{ name: string; url: string }> = [
		{ name: "Minecraft.net News", url: "https://www.minecraft.net/en-us/feeds/news" },
	];

	async function loadFeeds() {
		loading = true;
		try {
			const collected: FeedItem[] = [];
			for (const feed of feeds) {
				try {
					const rss = await fetch(feed.url).then((r) => r.text());
					const parsed = parseRss(rss, feed.name);
					collected.push(...parsed);
				} catch (e) {
					console.warn("feed failed:", feed.name, e);
				}
			}
			items = collected;
			if (collected.length === 0) {
				error = t("news.noFeedsError");
			}
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	function parseRss(xml: string, source: string): FeedItem[] {
		const items: FeedItem[] = [];
		const itemRe = /<item[\s>]([\s\S]*?)<\/item>/g;
		let m: RegExpExecArray | null;
		while ((m = itemRe.exec(xml)) !== null) {
			const block = m[1];
			const title = (block.match(/<title>([\s\S]*?)<\/title>/) ?? [])[1] ?? "";
			const link = (block.match(/<link>([\s\S]*?)<\/link>/) ?? [])[1] ?? "";
			const pub = (block.match(/<pubDate>([\s\S]*?)<\/pubDate>/) ?? [])[1] ?? "";
			if (title) {
				items.push({
					title: title.replace(/<!\[CDATA\[(.*?)\]\]>/g, "$1").trim(),
					link: link.trim(),
					pubDate: pub.trim(),
					source,
				});
			}
		}
		return items;
	}

	function openLink(item: FeedItem) {
		if (!item.link) return;
		window.open(item.link, "_blank", "noopener,noreferrer");
		toast(t("news.openedInBrowser", { title: item.title }), "info");
	}

	onMount(loadFeeds);
</script>

<div class="mx-auto flex h-full max-w-4xl flex-col gap-6">
	<header class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<Newspaper class="h-5 w-5" style="color: rgb(45, 212, 191);" />
			<Heading>{t("news.title")}</Heading>
		</div>
		<button
			class="flex items-center gap-1.5 rounded-md px-2 py-1 text-xs"
			style="border: 1px solid rgb(var(--border)); color: rgb(var(--fg-muted));"
			onclick={loadFeeds}
		>
			<Rss class="h-3 w-3" />
			{t("news.refresh")}
		</button>
	</header>

	<Card>
		{#snippet header()}
			<div class="flex items-center gap-2">
				<MessageSquare class="h-4 w-4" style="color: rgb(45, 212, 191);" />
				<span class="font-medium">{t("news.minecraftNews")}</span>
			</div>
		{/snippet}
		{#if loading}
			<p class="text-xs" style="color: rgb(var(--fg-subtle));">{t("news.loading")}</p>
		{:else if error}
			<p class="text-xs" style="color: rgb(250, 204, 21);">{error}</p>
		{:else if items.length === 0}
			<p class="text-xs" style="color: rgb(var(--fg-subtle));">{t("news.noItems")}</p>
		{:else}
			<ul class="flex flex-col gap-2">
				{#each items.slice(0, 20) as item, i (i)}
					<li>
						<button
							class="block w-full rounded-md p-2 text-left transition-colors hover:bg-bg-subtle"
							style="border: 1px solid rgb(var(--border));"
							onclick={() => openLink(item)}
						>
							<p class="text-sm font-medium" style="color: rgb(var(--fg));">{item.title}</p>
							<p class="mt-0.5 text-[10px]" style="color: rgb(var(--fg-subtle));">
								{item.source}{item.pubDate ? ` · ${new Date(item.pubDate).toLocaleString()}` : ""}
							</p>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</Card>

	<Card>
		{#snippet header()}
			<div class="flex items-center gap-2">
				<Rss class="h-4 w-4" style="color: rgb(45, 212, 191);" />
				<span class="font-medium">{t("news.changelog")}</span>
			</div>
		{/snippet}
		<ChangelogPanel />
	</Card>
</div>
