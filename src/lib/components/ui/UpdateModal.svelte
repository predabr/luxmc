<script lang="ts">
	import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { button } from "$lib/components/ui/button";
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { quintOut } from "svelte/easing";
	import { X, Loader2, ArrowUpRight, Copy, Check } from "lucide-svelte";
	import { marked } from "marked";
	import DOMPurify from "dompurify";
	import { focusTrap } from "$lib/utils/focusTrap";
	import { updaterStore } from "$lib/stores/updater.svelte";
	import { settings } from "$lib/stores/settings.svelte";

	const notes = $derived(
		DOMPurify.sanitize(
			marked.parse(
				(updaterStore.releaseNotes || "").replace(/^#\s+Luxmc[^\r\n]*(?:\r?\n)*/i, "").replace(/<!--[\s\S]*?-->/g, ""),
				{ async: false }
			),
			{ ALLOWED_TAGS: ["p", "ul", "ol", "li", "strong", "em", "code", "h2", "h3", "h4", "br", "a"], ALLOWED_ATTR: ["href"] }
		)
	);
	const hasNotes = $derived(notes.replace(/<[^>]+>/g, "").trim().length > 0);
	const sizeLabel = $derived(updaterStore.totalBytes > 0 ? `${(updaterStore.totalBytes / 1048576).toFixed(1)} MB` : "");

	let copied = $state(false);
	function close() {
		if (!updaterStore.isUpdating) updaterStore.showModal = false;
	}
	async function copyNotes() {
		await navigator.clipboard.writeText(updaterStore.releaseNotes || "").catch(() => {});
		copied = true;
		setTimeout(() => (copied = false), 1600);
	}
	onMount(() => {
		const timer = setTimeout(() => { if (settings.value.autoCheckUpdates) void updaterStore.check(false); }, 3000);
		const interval = setInterval(() => { if (settings.value.autoCheckUpdates) void updaterStore.check(false); }, 1_800_000);
		return () => { clearTimeout(timer); clearInterval(interval); };
	});
</script>

{#if updaterStore.showModal}
	<div class="fixed inset-0 z-[100] flex items-center justify-center bg-bg-overlay/70 p-4 backdrop-blur-sm" role="dialog" aria-modal="true" aria-labelledby="update-title" use:focusTrap tabindex="-1" onkeydown={(event) => { if (event.key === "Escape") { event.stopPropagation(); close(); } }} onclick={(event) => { if (event.target === event.currentTarget) close(); }} transition:fade={{ duration: settings.value.animations ? 140 : 0 }}>
		<section class="flex max-h-[86dvh] w-full max-w-md flex-col overflow-hidden rounded-2xl border border-fg/10 bg-bg-elevated shadow-elevated" in:scale={{ start: 0.98, easing: quintOut, duration: settings.value.animations ? 160 : 0 }}>
			<header class="flex items-center gap-3 px-4 pt-4">
				<span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-xl bg-brand-500/12 text-brand-400">
					{#if updaterStore.isUpdating}<Loader2 class="h-4 w-4 animate-spin" />{:else}<ArrowUpRight class="h-4 w-4" />{/if}
				</span>
				<div class="min-w-0 flex-1">
					<h2 id="update-title" class="truncate text-[13.5px] font-semibold text-fg">
						{updaterStore.isUpdating ? uiText("updateDesign.installing") : uiText("updateDesign.title")}
					</h2>
					<p class="mt-0.5 text-[11px] tabular-nums text-fg-muted">
						v{updaterStore.currentVersion}{#if updaterStore.latestVersion} → <span class="font-semibold text-brand-400">v{updaterStore.latestVersion}</span>{/if}
						{#if sizeLabel}<span class="text-fg-muted/60"> · {sizeLabel}</span>{/if}
					</p>
				</div>
				{#if !updaterStore.isUpdating}
					<button type="button" class={button({ variant: "ghost", size: "iconSm" })} aria-label={uiText("common.close")} onclick={close}><X class="h-4 w-4" /></button>
				{/if}
			</header>

			{#if updaterStore.latestVersion === updaterStore.currentVersion && updaterStore.latestRevision > updaterStore.currentRevision}
				<p class="mx-4 mt-3 rounded-lg border border-brand-400/20 bg-brand-500/5 px-3 py-2 text-[11.5px] text-brand-300">{uiText("refinement.sameVersion")}</p>
			{/if}

			{#if updaterStore.isUpdating}
				<div class="px-4 py-4" role="status" aria-live="polite">
					<div class="flex items-center justify-between text-[11.5px] text-fg-muted">
						<span>{updaterStore.statusText}</span>
						<span class="font-semibold tabular-nums text-brand-400">{updaterStore.progressPercent}%</span>
					</div>
					<div role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={updaterStore.progressPercent} class="mt-2 h-1.5 overflow-hidden rounded-full bg-fg/10">
						<div class="h-full rounded-full bg-brand-500 transition-[width] duration-200" style:width={`${updaterStore.progressPercent}%`}></div>
					</div>
				</div>
			{:else}
				<div class="min-h-0 flex-1 overflow-y-auto px-4 py-3">
					{#if updaterStore.updateError}
						<div class="rounded-lg border border-danger/25 bg-danger/10 px-3 py-2">
							<p class="text-[11.5px] font-semibold text-danger">{uiText("ui.7cc311f43f53578b")}</p>
							<p class="mt-1 text-[11px] leading-relaxed text-danger/80">{updaterStore.updateError}</p>
						</div>
					{/if}
					{#if hasNotes}
						<div class="update-notes text-[12px] leading-relaxed text-fg-muted">{@html notes}</div>
					{:else}
						<p class="text-[11.5px] text-fg-muted">Sem notas publicadas nesta versão. Veja os detalhes no GitHub.</p>
					{/if}
					{#if updaterStore.terminalCommand}
						<code class="mt-3 block break-all rounded-lg bg-fg/5 px-2 py-1.5 text-[11px] text-fg">{updaterStore.terminalCommand}</code>
					{/if}
				</div>

				<footer class="flex items-center gap-2 border-t border-fg/10 px-4 py-3">
					<button type="button" class={button({ variant: "primary", size: "sm", class: "gap-1.5" })} disabled={!updaterStore.downloadUrl} onclick={() => updaterStore.startUpdate()}>
						<ArrowUpRight class="h-3.5 w-3.5" /> {uiText(updaterStore.updateError ? "ui.45824b20097cc608" : "updateDesign.install")}
					</button>
					<button type="button" class={button({ variant: "ghost", size: "sm", class: "gap-1.5" })} onclick={copyNotes}>
						{#if copied}<Check class="h-3.5 w-3.5" />{:else}<Copy class="h-3.5 w-3.5" />{/if}
						{uiText("ui.ffd3736638c756e3")}
					</button>
					<a class="ml-auto text-[11px] font-semibold text-fg-muted hover:text-fg" href={updaterStore.releaseUrl || "https://github.com/predabr/luxmc/releases"} target="_blank" rel="noreferrer noopener">GitHub ↗</a>
				</footer>
			{/if}
		</section>
	</div>
{/if}

<style>
	.update-notes :global(p + p), .update-notes :global(p + ul), .update-notes :global(ul + p), .update-notes :global(h2), .update-notes :global(h3) { margin-top: 0.7rem; }
	.update-notes :global(ul), .update-notes :global(ol) { padding-left: 1.1rem; }
	.update-notes :global(ul) { list-style: disc; }
	.update-notes :global(ol) { list-style: decimal; }
	.update-notes :global(li + li) { margin-top: 0.4rem; }
	.update-notes :global(strong), .update-notes :global(h2), .update-notes :global(h3), .update-notes :global(h4) { color: rgb(var(--fg)); font-weight: 600; }
	.update-notes :global(a) { color: rgb(var(--brand-400)); text-decoration: underline; }
	.update-notes :global(code) { font-size: 11px; background: rgb(var(--fg) / 0.07); padding: 1px 4px; border-radius: 4px; }
</style>
