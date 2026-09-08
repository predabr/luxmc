<script lang="ts">
	import { onMount } from "svelte";
	import { changelogGet, type ChangelogEntry } from "$lib/api";
	import { Sparkles } from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

	let entries = $state<ChangelogEntry[]>([]);
	let loading = $state(true);

	onMount(async () => {
		try {
			entries = await changelogGet();
		} catch (e) {
			toast(t("changelog.failed", { error: String(e) }), "error");
		} finally {
			loading = false;
		}
	});
</script>

<div class="flex flex-col gap-3">
	<header class="flex items-center gap-2">
		<Sparkles class="h-4 w-4" style="color: rgb(var(--brand-400));" />
		<h2 class="text-sm font-medium" style="color: rgb(var(--fg));">{t("changelog.title")}</h2>
	</header>

	{#if loading}
		<p class="text-xs" style="color: rgb(var(--fg-subtle));">{t("changelog.loading")}</p>
	{:else}
		<ol class="flex flex-col gap-3">
			{#each entries as e (e.version)}
				<li
					class="rounded-lg p-3"
					style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg-subtle));"
				>
					<div class="mb-1 flex items-center justify-between">
						<h3 class="text-sm font-semibold" style="color: rgb(var(--fg));">
							v{e.version} — {e.title}
						</h3>
						<span class="text-[10px]" style="color: rgb(var(--fg-subtle));">{e.date}</span>
					</div>
					<ul class="flex flex-col gap-1 text-xs" style="color: rgb(var(--fg-muted));">
						{#each e.highlights as h, i (i)}
							<li class="flex items-start gap-1.5">
								<span style="color: rgb(var(--brand-400));">›</span>
								<span>{h}</span>
							</li>
						{/each}
					</ul>
				</li>
			{/each}
		</ol>
	{/if}
</div>
