<script lang="ts">
	import { onMount } from "svelte";
	import {
		AlertTriangle,
		Check,
		ExternalLink,
		FolderOpen,
		Link2,
		Loader2,
		Pencil,
		Plus,
		Puzzle,
		RefreshCw,
		ShieldCheck,
		ShieldX,
		Trash2,
		Power,
		PowerOff,
		Users
	} from "lucide-svelte";
	import { goto } from "$app/navigation";
	import { button as launcherButton } from "$lib/components/ui/button";
	import { plugins } from "$lib/stores/plugins.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	let { showHeader = true, catalogOnly = false }: { showHeader?: boolean; catalogOnly?: boolean } = $props();
	const { t } = useTranslation();

	let manualUrl = $state("");
	let showManual = $state(false);

	onMount(() => {
		plugins.notify = (message, kind) => toast(message, kind ?? "info");
		void plugins.init();
	});

	const busy = $derived(plugins.busyId);

	function bytes(value: number) {
		if (value > 1024 * 1024) return `${(value / (1024 * 1024)).toFixed(1)} MB`;
		if (value > 1024) return `${Math.round(value / 1024)} KB`;
		return `${value} B`;
	}

	function scanLabel(record: (typeof plugins.records)[number]) {
		if (!record.scan.clean) return t("plugins.shieldBlocked");
		if (record.scan.findings.length > 0) return t("plugins.shieldWarn");
		return t("plugins.shieldClean");
	}
</script>

<section class="plugins-manager flex flex-col gap-5">
	{#if showHeader}
		<header class="flex flex-wrap items-start gap-4">
			<span class="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border border-brand-400/25 bg-brand-500/10 text-brand-400"><Puzzle class="h-5 w-5" /></span>
			<div class="min-w-0 flex-1">
				<p class="page-eyebrow">Luxmc</p>
				<h2 class="mt-1 text-xl font-semibold tracking-tight text-fg">{t("plugins.title")}</h2>
				<p class="mt-1 max-w-2xl text-[13px] leading-relaxed text-fg-muted">{t("plugins.subtitle")}</p>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				<button type="button" class={launcherButton({ variant: "primary", size: "sm", class: "gap-2" })} onclick={() => goto("/plugins/editor")}>
					<Plus class="h-3.5 w-3.5" /> {t("plugins.create")}
				</button>
				<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "gap-2" })} onclick={() => plugins.openFolder()}>
					<FolderOpen class="h-3.5 w-3.5" /> {t("plugins.openFolder")}
				</button>
				<button type="button" class={launcherButton({ variant: "tinted", size: "sm", class: "gap-2" })} disabled={plugins.scanning} onclick={() => plugins.scanAll()}>
					{#if plugins.scanning}<Loader2 class="h-3.5 w-3.5 animate-spin" />{:else}<RefreshCw class="h-3.5 w-3.5" />{/if}
					{t("plugins.rescanAll")}
				</button>
			</div>
		</header>
	{/if}

	<div class="plugin-shield-band">
		<ShieldCheck class="h-4 w-4 shrink-0 text-brand-400" />
		<p>{t("plugins.providerNote")}</p>
	</div>

	{#if !catalogOnly}
		<div class="plugin-block">
			<div class="plugin-block-head">
				<h3>{t("plugins.installed")}</h3>
				<span class="plugin-count">{plugins.records.length}</span>
			</div>

			{#if plugins.records.length === 0}
				<p class="plugin-empty">{t("plugins.empty")}</p>
			{:else}
				<ul class="plugin-list">
					{#each plugins.records as record (record.manifest.id)}
						{@const blocked = !record.scan.clean}
						<li class="plugin-row" class:is-blocked={blocked}>
							<span class="plugin-icon" class:is-blocked={blocked}>
								{#if blocked}<ShieldX class="h-4 w-4" />{:else}<Puzzle class="h-4 w-4" />{/if}
							</span>
							<div class="plugin-info">
								<p class="plugin-name">
									{record.manifest.name}
									<span class="plugin-version">v{record.manifest.version || "1.0.0"}</span>
									<span class="plugin-badge" class:is-blocked={blocked} class:is-off={!record.enabled && !blocked}>
										{blocked ? t("plugins.shieldBlocked") : record.enabled ? t("plugins.enabled") : t("plugins.disabled")}
									</span>
								</p>
								<p class="plugin-meta">
									<span>{t("plugins.author")} {record.manifest.author || "—"}</span>
									<span class="plugin-dot">·</span>
									<span>{record.scan.files} {t("plugins.files")}</span>
									<span class="plugin-dot">·</span>
									<span>{bytes(record.sizeBytes)}</span>
									{#if record.manifest.permissions.length}
										<span class="plugin-dot">·</span>
										<span class="inline-flex items-center gap-1"><Users class="h-3 w-3" />{record.manifest.permissions.join(", ")}</span>
									{/if}
								</p>
								{#if record.scan.findings.length}
									<ul class="plugin-findings">
										{#each record.scan.findings.slice(0, 3) as finding}
											<li class={finding.severity === "critical" ? "is-critical" : "is-warning"}>
												<AlertTriangle class="h-3 w-3" /> {finding.message}
											</li>
										{/each}
									</ul>
								{/if}
							</div>
							<div class="plugin-actions">
								{#if !blocked && record.enabled}
									<button type="button" class={launcherButton({ variant: "ghostBrand", size: "iconSm" })} title={t("plugins.run")} aria-label={t("plugins.run")} onclick={() => goto(`/plugins/${record.manifest.id}`)}>
										<ExternalLink class="h-3.5 w-3.5" />
									</button>
								{/if}
								<button type="button" class={launcherButton({ variant: "ghost", size: "iconSm" })} title={t("plugins.edit")} aria-label={t("plugins.edit")} onclick={() => goto(`/plugins/editor?id=${record.manifest.id}`)}>
									<Pencil class="h-3.5 w-3.5" />
								</button>
								<button type="button" class={launcherButton({ variant: "ghost", size: "iconSm" })} title={t("plugins.rescan")} aria-label={t("plugins.rescan")} disabled={busy === record.manifest.id} onclick={() => plugins.rescan(record.manifest.id)}>
									<RefreshCw class={busy === record.manifest.id ? "h-3.5 w-3.5 animate-spin" : "h-3.5 w-3.5"} />
								</button>
								<button type="button" class={launcherButton({ variant: blocked || !record.enabled ? "secondary" : "ghost", size: "iconSm" })} title={record.enabled ? t("plugins.disable") : t("plugins.enable")} aria-label={record.enabled ? t("plugins.disable") : t("plugins.enable")} disabled={busy === record.manifest.id} onclick={() => plugins.toggle(record.manifest.id, !record.enabled)}>
									{#if record.enabled}<PowerOff class="h-3.5 w-3.5" />{:else}<Power class="h-3.5 w-3.5" />{/if}
								</button>
								<button type="button" class={launcherButton({ variant: "ghostDanger", size: "iconSm" })} title={t("plugins.remove")} aria-label={t("plugins.remove")} disabled={busy === record.manifest.id} onclick={() => plugins.remove(record.manifest.id)}>
									<Trash2 class="h-3.5 w-3.5" />
								</button>
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	{/if}

	<div class="plugin-block">
		<div class="plugin-block-head">
			<h3>{t("plugins.catalog")}</h3>
			{#if plugins.registry?.source}
				<span class="plugin-source">{plugins.registry.source.replace(/^https?:\/\//, "").replace(/\/.*$/, "")}</span>
			{/if}
		</div>

		{#if plugins.registryLoading}
			<p class="plugin-empty inline-flex items-center gap-2"><Loader2 class="h-3.5 w-3.5 animate-spin" /> {t("plugins.scanning")}</p>
		{:else if plugins.registry && plugins.registry.plugins.length > 0}
			<ul class="plugin-catalog">
				{#each plugins.registry.plugins as plugin (plugin.id)}
					{@const installed = plugins.installedId(plugin.id)}
					<li class="plugin-card">
						<div class="plugin-card-head">
							<span class="plugin-icon"><Puzzle class="h-4 w-4" /></span>
							<div class="min-w-0">
								<p class="plugin-name">{plugin.name}<span class="plugin-version">v{plugin.version || "1.0.0"}</span></p>
								<p class="plugin-meta"><span>{t("plugins.author")} {plugin.author || "—"}</span></p>
							</div>
						</div>
						<p class="plugin-description">{plugin.description}</p>
						<div class="plugin-card-tags">
							{#each plugin.tags.slice(0, 3) as tag}<span class="plugin-tag">{tag}</span>{/each}
						</div>
						<div class="plugin-card-foot">
							{#if installed}
								<span class="plugin-installed"><Check class="h-3.5 w-3.5" /> {t("plugins.installed")}</span>
							{:else}
								<button type="button" class={launcherButton({ variant: "primary", size: "sm", class: "gap-2" })} disabled={busy === plugin.id} onclick={() => plugins.install(plugin)}>
									{#if busy === plugin.id}<Loader2 class="h-3.5 w-3.5 animate-spin" />{/if}
									{busy === plugin.id ? t("plugins.installing") : t("plugins.install")}
								</button>
							{/if}
							{#if plugin.homepage}
								<a class="plugin-link" href={plugin.homepage} target="_blank" rel="noreferrer noopener">{t("plugins.detail")} <ExternalLink class="h-3 w-3" /></a>
							{/if}
						</div>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="plugin-empty">{plugins.error ? t("plugins.registryError") : t("plugins.emptyCatalog")}</p>
		{/if}

		<div class="plugin-manual">
			<button type="button" class="plugin-manual-toggle" aria-expanded={showManual} onclick={() => (showManual = !showManual)}>
				<Link2 class="h-3.5 w-3.5" /> {t("plugins.installUrlLabel")}
			</button>
			{#if showManual}
				<form class="plugin-manual-form" onsubmit={(event) => { event.preventDefault(); if (!manualUrl.trim()) return; void plugins.installUrl(manualUrl.trim()); manualUrl = ""; }}>
					<input class="luxmc-control" type="url" placeholder={t("plugins.installUrlPlaceholder")} bind:value={manualUrl} required />
					<button type="submit" class={launcherButton({ variant: "secondary", size: "sm" })} disabled={!manualUrl.trim() || plugins.busyId !== null}>{t("plugins.installUrlAction")}</button>
				</form>
			{/if}
		</div>
	</div>
</section>

<style>
	.plugins-manager { container-type: inline-size; }

	.plugin-shield-band {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 14px;
		border-radius: 14px;
		border: 1px solid rgb(var(--brand-500) / 0.24);
		background: linear-gradient(120deg, rgb(var(--brand-500) / 0.14), rgb(var(--brand-500) / 0.05));
		color: rgb(var(--fg) / 0.82);
		font-size: 12px;
		line-height: 1.45;
	}

	.plugin-block {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px;
		border-radius: 16px;
		border: 1px solid rgb(var(--fg) / 0.1);
		background: rgb(var(--bg-elevated) / 0.6);
	}

	.plugin-block-head { display: flex; align-items: center; gap: 8px; }
	.plugin-block-head h3 { margin: 0; font-size: 13px; font-weight: 700; color: rgb(var(--fg)); }
	.plugin-count { font-size: 11px; font-weight: 700; padding: 1px 7px; border-radius: 999px; background: rgb(var(--fg) / 0.1); color: rgb(var(--fg) / 0.7); }
	.plugin-source { margin-left: auto; font-size: 11px; color: rgb(var(--fg) / 0.45); }
	.plugin-empty { margin: 0; padding: 6px 2px; font-size: 12.5px; color: rgb(var(--fg) / 0.5); }

	.plugin-list { display: flex; flex-direction: column; gap: 8px; margin: 0; padding: 0; list-style: none; }

	.plugin-row {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 10px;
		border-radius: 12px;
		border: 1px solid rgb(var(--fg) / 0.08);
		background: rgb(var(--fg) / 0.035);
	}
	.plugin-row.is-blocked { border-color: rgb(var(--danger) / 0.35); background: rgb(var(--danger) / 0.08); }

	.plugin-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		flex-shrink: 0;
		border-radius: 9px;
		border: 1px solid rgb(var(--brand-500) / 0.3);
		background: rgb(var(--brand-500) / 0.14);
		color: rgb(var(--brand-400));
	}
	.plugin-icon.is-blocked { border-color: rgb(var(--danger) / 0.4); background: rgb(var(--danger) / 0.14); color: rgb(var(--danger)); }

	.plugin-info { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 3px; }
	.plugin-name { display: flex; align-items: center; gap: 6px; margin: 0; font-size: 12.5px; font-weight: 700; color: rgb(var(--fg)); }
	.plugin-version { font-size: 10.5px; font-weight: 600; color: rgb(var(--fg) / 0.45); }
	.plugin-badge { font-size: 10px; font-weight: 700; padding: 1px 6px; border-radius: 999px; letter-spacing: 0.02em; background: rgb(var(--brand-500) / 0.18); color: rgb(var(--brand-400)); text-transform: uppercase; }
	.plugin-badge.is-blocked { background: rgb(var(--danger) / 0.18); color: rgb(var(--danger)); }
	.plugin-badge.is-off { background: rgb(var(--fg) / 0.1); color: rgb(var(--fg) / 0.55); }

	.plugin-meta { display: flex; flex-wrap: wrap; align-items: center; gap: 5px; margin: 0; font-size: 11.5px; color: rgb(var(--fg) / 0.5); }
	.plugin-dot { color: rgb(var(--fg) / 0.25); }
	.plugin-findings { display: flex; flex-direction: column; gap: 2px; margin: 2px 0 0; padding: 0; list-style: none; font-size: 11px; }
	.plugin-findings li { display: flex; align-items: center; gap: 5px; color: rgb(var(--fg) / 0.6); }
	.plugin-findings li.is-critical { color: rgb(var(--danger)); }
	.plugin-findings li.is-warning { color: rgb(234 179 8); }

	.plugin-actions { display: flex; align-items: center; gap: 4px; flex-shrink: 0; }

	.plugin-catalog { display: grid; grid-template-columns: repeat(auto-fill, minmax(238px, 1fr)); gap: 10px; margin: 0; padding: 0; list-style: none; }

	.plugin-card {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px;
		border-radius: 14px;
		border: 1px solid rgb(var(--fg) / 0.1);
		background: rgb(var(--fg) / 0.035);
	}
	.plugin-card-head { display: flex; align-items: center; gap: 9px; }
	.plugin-description { margin: 0; font-size: 12px; line-height: 1.45; color: rgb(var(--fg) / 0.62); display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; }
	.plugin-card-tags { display: flex; flex-wrap: wrap; gap: 4px; }
	.plugin-tag { font-size: 10px; font-weight: 600; padding: 1px 7px; border-radius: 999px; background: rgb(var(--fg) / 0.07); color: rgb(var(--fg) / 0.55); }
	.plugin-card-foot { display: flex; align-items: center; gap: 8px; margin-top: auto; }
	.plugin-installed { display: inline-flex; align-items: center; gap: 4px; font-size: 11.5px; font-weight: 700; color: rgb(var(--brand-400)); }
	.plugin-link { margin-left: auto; display: inline-flex; align-items: center; gap: 3px; font-size: 11px; font-weight: 600; color: rgb(var(--fg) / 0.5); text-decoration: none; }
	.plugin-link:hover { color: rgb(var(--fg)); }

	.plugin-manual { display: flex; flex-direction: column; gap: 8px; border-top: 1px solid rgb(var(--fg) / 0.08); padding-top: 10px; }
	.plugin-manual-toggle { display: inline-flex; align-items: center; gap: 6px; align-self: flex-start; padding: 0; border: 0; background: transparent; font-size: 11.5px; font-weight: 600; color: rgb(var(--fg) / 0.55); cursor: pointer; }
	.plugin-manual-toggle:hover { color: rgb(var(--fg)); }
	.plugin-manual-form { display: flex; gap: 8px; }
	.plugin-manual-form input { flex: 1; min-width: 0; height: 34px; border-radius: 10px; padding: 0 10px; font-size: 12px; }

	@media (max-width: 620px) {
		.plugin-row { flex-wrap: wrap; }
		.plugin-actions { width: 100%; justify-content: flex-end; }
	}
</style>
