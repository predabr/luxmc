<script lang="ts">
	import { onMount } from "svelte";
	import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";

	type WindowLike = {
		minimize: () => Promise<void>;
		toggleMaximize: () => Promise<void>;
		close: () => Promise<void>;
		isMaximized: () => Promise<boolean>;
		onResized: (handler: () => void) => Promise<() => void>;
	};

	let maximized = $state(false);
	let version = $state("");
	let controls = $state(false);
	let win: WindowLike | null = null;

	async function windowApi(): Promise<WindowLike> {
		if (win) return win;
		const { getCurrentWindow } = await import("@tauri-apps/api/window");
		win = getCurrentWindow() as unknown as WindowLike;
		return win;
	}

	async function minimize() {
		try { await (await windowApi()).minimize(); } catch { /* window api unavailable */ }
	}

	async function toggleMaximize() {
		try {
			const current = await windowApi();
			await current.toggleMaximize();
			maximized = await current.isMaximized();
		} catch { /* window api unavailable */ }
	}

	async function closeWindow() {
		try { await (await windowApi()).close(); } catch { /* window api unavailable */ }
	}

	onMount(() => {
		let disposed = false;
		let stop: (() => void) | undefined;
		void (async () => {
			try {
				const current = await windowApi();
				maximized = await current.isMaximized();
				controls = true;
				const unlisten = await current.onResized(async () => {
					try { maximized = await current.isMaximized(); } catch { /* ignore */ }
				});
				if (disposed) unlisten(); else stop = unlisten;
			} catch {
				controls = false;
			}
			try {
				const { getVersion } = await import("@tauri-apps/api/app");
				version = await getVersion();
			} catch { /* version unavailable */ }
		})();
		return () => { disposed = true; stop?.(); };
	});
</script>

<header class="lux-titlebar" data-tauri-drag-region>
	<div class="lux-titlebar-brand" data-tauri-drag-region>
		<img src="/logo.png" alt="" class="lux-titlebar-logo" draggable="false" data-tauri-drag-region />
		<span class="lux-titlebar-name" data-tauri-drag-region>Luxmc</span>
		{#if version}<span class="lux-titlebar-version" data-tauri-drag-region>{version}</span>{/if}
	</div>
	{#if controls}
		<div class="lux-titlebar-actions">
			<button type="button" class="lux-titlebar-button" onclick={minimize} title={uiText("window.minimize")} aria-label={uiText("window.minimize")}>
				<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2 6h8" /></svg>
			</button>
			<button type="button" class="lux-titlebar-button" onclick={toggleMaximize} title={maximized ? uiText("window.restore") : uiText("window.maximize")} aria-label={maximized ? uiText("window.restore") : uiText("window.maximize")}>
				{#if maximized}
					<svg viewBox="0 0 12 12" aria-hidden="true"><rect x="1.6" y="3.4" width="6.4" height="6.4" rx="1.2" /><path d="M4.2 3.4V2.6A1.2 1.2 0 0 1 5.4 1.4h4A1.2 1.2 0 0 1 10.6 2.6v4A1.2 1.2 0 0 1 9.4 7.8h-.8" /></svg>
				{:else}
					<svg viewBox="0 0 12 12" aria-hidden="true"><rect x="1.9" y="1.9" width="8.2" height="8.2" rx="1.5" /></svg>
				{/if}
			</button>
			<button type="button" class="lux-titlebar-button is-close" onclick={closeWindow} title={uiText("window.close")} aria-label={uiText("window.close")}>
				<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.6 2.6l6.8 6.8M9.4 2.6l-6.8 6.8" /></svg>
			</button>
		</div>
	{/if}
</header>
