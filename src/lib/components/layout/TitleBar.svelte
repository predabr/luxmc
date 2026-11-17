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
	let active = $state(true);
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
		const setActive = (value: boolean) => () => { active = value; };
		const onFocus = setActive(true);
		const onBlur = setActive(false);
		window.addEventListener("focus", onFocus);
		window.addEventListener("blur", onBlur);
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
		return () => {
			disposed = true;
			stop?.();
			window.removeEventListener("focus", onFocus);
			window.removeEventListener("blur", onBlur);
		};
	});
</script>

<header class="lux-titlebar" class:is-inactive={!active} data-tauri-drag-region>
	<div class="lux-titlebar-brand" data-tauri-drag-region>
		<img src="/logo.png" alt="" class="lux-titlebar-logo" draggable="false" data-tauri-drag-region />
		<span class="lux-titlebar-name" data-tauri-drag-region>Luxmc</span>
		{#if version}<span class="lux-titlebar-version" data-tauri-drag-region>{version}</span>{/if}
	</div>
	{#if controls}
		<div class="lux-titlebar-actions">
			<button type="button" class="lux-titlebar-button is-minimize" onclick={minimize} title={uiText("window.minimize")} aria-label={uiText("window.minimize")}>
				<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.4 6h7.2" /></svg>
			</button>
			<button type="button" class="lux-titlebar-button is-maximize" onclick={toggleMaximize} title={maximized ? uiText("window.restore") : uiText("window.maximize")} aria-label={maximized ? uiText("window.restore") : uiText("window.maximize")}>
				{#if maximized}
					<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3.4 4.6h4v4h-4z" /><path d="M5 4.6V3.2h3.8V7h-1.4" /></svg>
				{:else}
					<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3.2 5.2V3.2h3.6M8.8 6.8v2H5.2" /></svg>
				{/if}
			</button>
			<button type="button" class="lux-titlebar-button is-close" onclick={closeWindow} title={uiText("window.close")} aria-label={uiText("window.close")}>
				<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3.4 3.4l5.2 5.2M8.6 3.4l-5.2 5.2" /></svg>
			</button>
		</div>
	{/if}
</header>
