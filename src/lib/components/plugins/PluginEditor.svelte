<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { page } from "$app/state";
	import {
		ArrowLeft,
		Code2,
		Eye,
		Loader2,
		Play,
		Plus,
		RefreshCw,
		Save,
		ShieldCheck,
		ShieldX,
		Sparkles
	} from "lucide-svelte";
	import { button as launcherButton } from "$lib/components/ui/button";
	import { pluginsApi, type PluginPermission } from "$lib/api/plugins";
	import { plugins } from "$lib/stores/plugins.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import {
		PLUGIN_TEMPLATE_HTML,
		PLUGIN_TEMPLATE_JS,
		buildSandboxDocument,
		slugifyPluginId
	} from "$lib/utils/pluginSandbox";

	const { t } = useTranslation();
	const editingId = $derived(page.url.searchParams.get("id"));

	let pluginId = $state("");
	let name = $state("");
	let author = $state("");
	let description = $state("");
	let permissions = $state<PluginPermission[]>([]);
	let source = $state(PLUGIN_TEMPLATE_JS);
	let dirty = $state(false);
	let loading = $state(false);
	let saving = $state(false);
	let previewFrame = $state<HTMLIFrameElement | null>(null);
	let previewTimer: ReturnType<typeof setTimeout> | null = null;

	const permissionOptions: { id: PluginPermission; label: () => string }[] = [
		{ id: "storage", label: () => t("plugins.permissionStorage") },
		{ id: "network", label: () => t("plugins.permissionNetwork") },
		{ id: "profiles", label: () => t("plugins.permissionProfiles") },
	];

	const idValid = $derived(/^[a-z0-9][a-z0-9._-]*$/.test(pluginId));
	const canSave = $derived(!saving && idValid && name.trim().length > 0 && source.trim().length > 0);

	function applyTemplate(kind: "panel" | "html" | "blank") {
		source = kind === "panel" ? PLUGIN_TEMPLATE_JS : kind === "html" ? PLUGIN_TEMPLATE_HTML : "// escreve aqui\n";
		dirty = true;
		schedulePreview();
	}

	function togglePermission(id: PluginPermission) {
		permissions = permissions.includes(id) ? permissions.filter((value) => value !== id) : [...permissions, id];
	}

	function renderPreview() {
		if (!previewFrame) return;
		previewFrame.srcdoc = buildSandboxDocument(
			{ id: pluginId || "preview", name: name || "Preview", version: "0.1.0", author, permissions },
			source
		);
	}

	function schedulePreview() {
		if (previewTimer) clearTimeout(previewTimer);
		previewTimer = setTimeout(renderPreview, 600);
	}

	function onKeydown(event: KeyboardEvent) {
		if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
			event.preventDefault();
			void save();
		}
		if (event.key === "Tab") {
			event.preventDefault();
			const target = event.target as HTMLTextAreaElement;
			const start = target.selectionStart;
			source = `${source.slice(0, start)}\t${source.slice(target.selectionEnd)}`;
			requestAnimationFrame(() => target.setSelectionRange(start + 1, start + 1));
		}
	}

	let listener: ((event: MessageEvent) => void) | null = null;

	onMount(() => {
		listener = (event: MessageEvent) => {
			const data = event.data;
			if (!data || data.source !== "luxmc-plugin") return;
			if (event.source !== previewFrame?.contentWindow) return;
			if (data.type === "toast") toast(`${name || "Plugin"}: ${data.payload}`, "info");
			if (data.type === "error") toast(`${name || "Plugin"}: ${data.payload}`, "error");
			if (data.type === "storage:get") {
				previewFrame?.contentWindow?.postMessage({ source: "luxmc-host", type: "storage:value", payload: { key: data.payload, value: null } }, "*");
			}
		};
		window.addEventListener("message", listener);
		if (editingId) void loadExisting(editingId);
		else renderPreview();
	});

	async function loadExisting(id: string) {
		loading = true;
		try {
			const record = plugins.records.find((entry) => entry.manifest.id === id) ?? (await pluginsApi.list()).find((entry) => entry.manifest.id === id);
			if (!record) {
				toast(t("plugins.unavailable"), "error");
				return;
			}
			pluginId = record.manifest.id;
			name = record.manifest.name;
			author = record.manifest.author;
			description = record.manifest.description;
			permissions = [...record.manifest.permissions];
			source = await pluginsApi.source(id);
			dirty = false;
			renderPreview();
		} catch (error) {
			toast(String(error), "error");
		} finally {
			loading = false;
		}
	}

	async function save() {
		if (!canSave) return;
		saving = true;
		try {
			const record = editingId
				? await plugins.saveSource(editingId, source)
				: await plugins.create({ id: pluginId, name, author, description, permissions, source });
			if (record) {
				dirty = false;
				if (!editingId) await goto(`/plugins/${record.manifest.id}`);
			}
		} finally {
			saving = false;
		}
	}

	onDestroy(() => {
		if (previewTimer) clearTimeout(previewTimer);
		if (listener) window.removeEventListener("message", listener);
	});
</script>

<svelte:window onkeydown={onKeydown} />

<section class="plugin-editor flex flex-col gap-4">
	<header class="flex flex-wrap items-center gap-3">
		<button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "gap-2" })} onclick={() => goto("/plugins")}>
			<ArrowLeft class="h-3.5 w-3.5" /> {t("plugins.title")}
		</button>
		<div class="min-w-0">
			<h1 class="truncate text-lg font-semibold tracking-tight text-fg">{editingId ? t("plugins.edit") : t("plugins.create")}</h1>
			<p class="text-[11.5px] text-fg-muted">{t("plugins.editorSubtitle")}</p>
		</div>
		<div class="flex items-center gap-2 ml-auto">
			{#if dirty}<span class="plugin-unsaved">{t("plugins.unsaved")}</span>{/if}
			<button type="button" class={launcherButton({ variant: "primary", size: "sm", class: "gap-2" })} disabled={!canSave} onclick={() => save()}>
				{#if saving}<Loader2 class="h-3.5 w-3.5 animate-spin" />{:else}<Save class="h-3.5 w-3.5" />{/if}
				{saving ? t("plugins.saving") : t("plugins.save")}
			</button>
			{#if editingId}
				<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "gap-2" })} onclick={() => goto(`/plugins/${editingId}`)}>
					<Play class="h-3.5 w-3.5" /> {t("plugins.runNow")}
				</button>
			{/if}
		</div>
	</header>

	<div class="plugin-editor-grid">
		<div class="plugin-editor-main">
			<div class="plugin-panel">
				<div class="plugin-panel-grid">
					<label class="plugin-field">
						<span>{t("plugins.name")}</span>
						<input class="luxmc-control" bind:value={name} placeholder="Minha Ferramenta" oninput={() => { if (!editingId) pluginId = slugifyPluginId(name); dirty = true; }} />
					</label>
					<label class="plugin-field">
						<span>{t("plugins.id")}</span>
						<input class="luxmc-control" bind:value={pluginId} readonly={Boolean(editingId)} placeholder="minha-ferramenta" />
						{#if !idValid}<small class="plugin-error">{t("plugins.badId")}</small>{/if}
					</label>
					<label class="plugin-field">
						<span>{t("plugins.author")}</span>
						<input class="luxmc-control" bind:value={author} placeholder="voce" oninput={() => (dirty = true)} />
					</label>
					<label class="plugin-field">
						<span>{t("plugins.permissions")}</span>
						<div class="plugin-permissions">
							{#each permissionOptions as permission (permission.id)}
								<button type="button" class="plugin-chip" aria-pressed={permissions.includes(permission.id)} onclick={() => { togglePermission(permission.id); dirty = true; }}>
									{permission.label()}
								</button>
							{/each}
						</div>
					</label>
					<label class="plugin-field plugin-field-wide">
						<span>{t("plugins.description")}</span>
						<input class="luxmc-control" bind:value={description} placeholder="O que esse plugin faz?" oninput={() => (dirty = true)} />
					</label>
				</div>
				<p class="plugin-hint">{t("plugins.permHint")}</p>
			</div>

			<div class="plugin-panel plugin-code-panel">
				<div class="plugin-panel-head">
					<p class="plugin-panel-title"><Code2 class="h-3.5 w-3.5" /> main.js</p>
					<div class="plugin-template-row">
						<span class="plugin-template-label"><Sparkles class="h-3 w-3" /> {t("plugins.template")}</span>
						<button type="button" class="plugin-chip" onclick={() => applyTemplate("panel")}>{t("plugins.templatePanel")}</button>
						<button type="button" class="plugin-chip" onclick={() => applyTemplate("html")}>{t("plugins.templateHtml")}</button>
						<button type="button" class="plugin-chip" onclick={() => applyTemplate("blank")}>{t("plugins.templateBlank")}</button>
					</div>
				</div>
				{#if loading}
					<p class="plugin-hint inline-flex items-center gap-2"><Loader2 class="h-3.5 w-3.5 animate-spin" /> {t("plugins.notLoaded")}</p>
				{:else}
					<textarea class="plugin-code" spellcheck="false" bind:value={source} oninput={() => { dirty = true; schedulePreview(); }} aria-label="main.js"></textarea>
				{/if}
			</div>
		</div>

		<div class="plugin-panel plugin-preview-panel">
			<div class="plugin-panel-head">
				<p class="plugin-panel-title"><Eye class="h-3.5 w-3.5" /> {t("plugins.preview")}</p>
				<button type="button" class={launcherButton({ variant: "ghost", size: "iconSm" })} title={t("plugins.refreshPreview")} aria-label={t("plugins.refreshPreview")} onclick={() => renderPreview()}>
					<RefreshCw class="h-3.5 w-3.5" />
				</button>
			</div>
			<iframe bind:this={previewFrame} title={t("plugins.preview")} class="plugin-preview-frame" sandbox="allow-scripts" referrerpolicy="no-referrer"></iframe>
			<p class="plugin-shield-note"><ShieldCheck class="h-3.5 w-3.5" /> {t("plugins.shieldNote")}</p>
			{#if plugins.blocked.length}
				<p class="plugin-shield-note is-danger"><ShieldX class="h-3.5 w-3.5" /> {t("plugins.blockedNote")}</p>
			{/if}
		</div>
	</div>
</section>

<style>
	.plugin-editor { container-type: inline-size; }

	.plugin-editor-grid {
		display: grid;
		gap: 12px;
		grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr);
		align-items: start;
	}

	.plugin-editor-main { display: flex; flex-direction: column; gap: 12px; min-width: 0; }

	.plugin-panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px;
		border-radius: 16px;
		border: 1px solid rgb(var(--fg) / 0.1);
		background: rgb(var(--bg-elevated) / 0.6);
		min-width: 0;
	}

	.plugin-panel-head { display: flex; align-items: center; gap: 8px; }
	.plugin-panel-title { display: inline-flex; align-items: center; gap: 6px; margin: 0; font-size: 12px; font-weight: 700; color: rgb(var(--fg)); }

	.plugin-panel-grid { display: grid; gap: 10px; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); }
	.plugin-field { display: flex; flex-direction: column; gap: 4px; font-size: 11.5px; font-weight: 600; color: rgb(var(--fg) / 0.6); }
	.plugin-field-wide { grid-column: 1 / -1; }
	.plugin-field input { height: 34px; border-radius: 10px; padding: 0 10px; font-size: 12.5px; }
	.plugin-error { color: rgb(var(--danger)); font-weight: 600; }

	.plugin-permissions { display: flex; flex-wrap: wrap; gap: 5px; }
	.plugin-chip {
		height: 26px;
		padding: 0 9px;
		border-radius: 8px;
		border: 1px solid rgb(var(--fg) / 0.12);
		background: rgb(var(--fg) / 0.05);
		font-size: 11px;
		font-weight: 600;
		color: rgb(var(--fg) / 0.65);
		cursor: pointer;
	}
	.plugin-chip:hover { color: rgb(var(--fg)); background: rgb(var(--fg) / 0.09); }
	.plugin-chip[aria-pressed="true"] { border-color: rgb(var(--brand-500) / 0.4); background: rgb(var(--brand-500) / 0.16); color: rgb(var(--brand-400)); }

	.plugin-template-row { display: flex; align-items: center; gap: 5px; margin-left: auto; flex-wrap: wrap; }
	.plugin-template-label { display: inline-flex; align-items: center; gap: 4px; font-size: 10.5px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.03em; color: rgb(var(--fg) / 0.4); }

	.plugin-code {
		width: 100%;
		min-height: 340px;
		resize: vertical;
		border-radius: 12px;
		border: 1px solid rgb(var(--fg) / 0.12);
		background: rgb(0 0 0 / 0.35);
		padding: 12px;
		font-family: ui-monospace, "SFMono-Regular", Consolas, monospace;
		font-size: 12px;
		line-height: 1.55;
		color: rgb(var(--fg) / 0.9);
		tab-size: 2;
	}
	.plugin-code:focus-visible { outline: 2px solid rgb(var(--brand-400) / 0.6); outline-offset: 1px; }

	.plugin-preview-frame {
		width: 100%;
		min-height: 300px;
		border: 1px solid rgb(var(--fg) / 0.12);
		border-radius: 12px;
		background: rgb(0 0 0 / 0.22);
	}

	.plugin-hint { margin: 0; font-size: 11px; line-height: 1.45; color: rgb(var(--fg) / 0.45); }
	.plugin-shield-note { display: flex; align-items: center; gap: 6px; margin: 0; font-size: 11px; color: rgb(var(--fg) / 0.5); }
	.plugin-shield-note.is-danger { color: rgb(var(--danger)); }
	.plugin-unsaved { font-size: 11px; font-weight: 600; color: rgb(234 179 8); }

	@container (max-width: 900px) {
		.plugin-editor-grid { grid-template-columns: minmax(0, 1fr); }
	}
</style>
