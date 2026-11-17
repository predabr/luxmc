<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { ArrowLeft, Loader2, ShieldCheck, ShieldX } from "lucide-svelte";
	import { button as launcherButton } from "$lib/components/ui/button";
	import { pluginsApi, type PluginManifest } from "$lib/api/plugins";
	import { plugins } from "$lib/stores/plugins.svelte";
	import { buildSandboxDocument } from "$lib/utils/pluginSandbox";
	import { toast } from "$lib/stores/toasts.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();
	const pluginId = $derived(page.params.id ?? "");

	let frame = $state<HTMLIFrameElement | null>(null);
	let documentHtml = $state<string | null>(null);
	let manifest = $state<PluginManifest | null>(null);
	let error = $state<string | null>(null);
	let loading = $state(true);

	const STORAGE_PREFIX = "luxmc.plugin.";
	const STORAGE_LIMIT = 64 * 1024;

	function storageKey(plugin: string, key: string) {
		return `${STORAGE_PREFIX}${plugin}.${key}`;
	}

	function readStorage(plugin: string, key: string) {
		try {
			return localStorage.getItem(storageKey(plugin, key));
		} catch {
			return null;
		}
	}

	function writeStorage(plugin: string, key: string, value: string) {
		if (key.length > 120 || value.length > STORAGE_LIMIT) return false;
		try {
			localStorage.setItem(storageKey(plugin, key), value);
			return true;
		} catch {
			return false;
		}
	}

	async function boot() {
		if (!pluginId) return;
		loading = true;
		error = null;
		try {
			const bundle = await pluginsApi.script(pluginId);
			manifest = bundle.manifest;
			documentHtml = buildSandboxDocument(bundle.manifest, bundle.script);
			if (frame) frame.srcdoc = documentHtml;
		} catch (cause) {
			error = String(cause);
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (frame && documentHtml) frame.srcdoc = documentHtml;
	});

	let listener: ((event: MessageEvent) => void) | null = null;

	onMount(() => {
		listener = (event: MessageEvent) => {
			const data = event.data;
			if (!data || data.source !== "luxmc-plugin") return;
			if (event.source !== frame?.contentWindow) return;
			switch (data.type) {
				case "toast":
					toast(`${manifest?.name ?? "Plugin"}: ${data.payload}`, "info");
					break;
				case "error":
					toast(`${manifest?.name ?? "Plugin"}: ${data.payload}`, "error");
					break;
				case "storage:get":
					frame?.contentWindow?.postMessage({ source: "luxmc-host", type: "storage:value", payload: { key: data.payload, value: readStorage(pluginId, data.payload) } }, "*");
					break;
				case "storage:set":
					writeStorage(pluginId, data.payload.key, data.payload.value);
					break;
				case "resize":
					break;
			}
		};
		window.addEventListener("message", listener);
		void boot();
	});

	onDestroy(() => {
		if (listener) window.removeEventListener("message", listener);
	});
</script>

<section class="plugin-host flex flex-col gap-4">
	<header class="flex flex-wrap items-center gap-3">
		<button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "gap-2" })} onclick={() => goto("/plugins")}>
			<ArrowLeft class="h-3.5 w-3.5" /> {t("plugins.title")}
		</button>
		<div class="min-w-0">
			<h1 class="truncate text-lg font-semibold tracking-tight text-fg">{manifest?.name ?? pluginId}</h1>
			{#if manifest}
				<p class="text-[11.5px] text-fg-muted">{t("plugins.author")} {manifest.author || "—"} · v{manifest.version || "1.0.0"}</p>
			{/if}
		</div>
		{#if manifest}
			<span class="plugin-sandbox-pill"><ShieldCheck class="h-3.5 w-3.5" /> {t("plugins.sandbox")}</span>
		{/if}
		<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "ml-auto gap-2" })} onclick={() => void plugins.rescan(pluginId)}>
			{t("plugins.rescan")}
		</button>
	</header>

	{#if loading}
		<div class="plugin-host-body flex items-center justify-center gap-2 text-[12.5px] text-fg-muted"><Loader2 class="h-4 w-4 animate-spin" /> {t("plugins.notLoaded")}</div>
	{:else if error}
		<div class="plugin-host-body plugin-host-error">
			<ShieldX class="h-5 w-5" />
			<p>{error}</p>
			<button type="button" class={launcherButton({ variant: "secondary", size: "sm" })} onclick={() => boot()}>{t("plugins.rescan")}</button>
		</div>
	{:else}
		<iframe
			bind:this={frame}
			title={manifest?.name ?? pluginId}
			class="plugin-host-frame"
			sandbox="allow-scripts"
			referrerpolicy="no-referrer"
		></iframe>
	{/if}
</section>

<style>
	.plugin-host { container-type: inline-size; }

	.plugin-sandbox-pill {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 9px;
		border-radius: 999px;
		border: 1px solid rgb(var(--brand-500) / 0.3);
		background: rgb(var(--brand-500) / 0.12);
		font-size: 10.5px;
		font-weight: 700;
		letter-spacing: 0.02em;
		text-transform: uppercase;
		color: rgb(var(--brand-400));
	}

	.plugin-host-body {
		min-height: 220px;
		padding: 16px;
		border-radius: 16px;
		border: 1px solid rgb(var(--fg) / 0.1);
		background: rgb(var(--bg-elevated) / 0.6);
	}

	.plugin-host-error { display: flex; flex-direction: column; align-items: center; gap: 10px; text-align: center; color: rgb(var(--danger)); }
	.plugin-host-error p { margin: 0; font-size: 12.5px; }

	.plugin-host-frame {
		width: 100%;
		min-height: 60vh;
		border: 1px solid rgb(var(--fg) / 0.1);
		border-radius: 16px;
		background: rgb(var(--bg-elevated) / 0.55);
	}
</style>
