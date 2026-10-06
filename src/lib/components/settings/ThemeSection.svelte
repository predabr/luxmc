<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { Check, Sparkles, Image, Palette, Eye, Upload, Trash2 } from "lucide-svelte";
	import WallpaperThumbnail from "./WallpaperThumbnail.svelte";
    import { importWallpaper } from "$lib/api/wallpaper";
	import { themeStore, THEMES, ACCENTS, BACKGROUNDS } from "$lib/stores/theme.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { schedulePersist, persistNow } from "$lib/stores/persistence.svelte";
	import { startSoundscape, stopSoundscape, setSoundscapeVolume } from "$lib/utils/sound";
	import { open } from "@tauri-apps/plugin-dialog";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	type Props = {
		onSave?: () => void;
	};

	let { onSave }: Props = $props();
	const { t } = useTranslation();

	function getThemeName(id: string): string {
		if (id === "dark") return t("settings.appearanceOptions.themeDark");
		if (id === "light") return t("settings.appearanceOptions.themeLight");
		return THEMES[id]?.name ?? id;
	}

	function getBgName(id: string): string {
		const map: Record<string, string> = {
			obsidian: t("settings.appearanceOptions.bgObsidian"),
			cosmos: t("settings.appearanceOptions.bgCosmos"),
			night: t("settings.appearanceOptions.bgNight"),
			day: t("settings.appearanceOptions.bgDay"),
			aurora: t("settings.appearanceOptions.bgAurora"),
			sakura: t("settings.appearanceOptions.bgSakura"),
			lush: t("settings.appearanceOptions.bgLush"),
		};
		return map[id] || BACKGROUNDS[id]?.name || id;
	}

	function getAccentName(id: string): string {
		const map: Record<string, string> = {
			gold: t("settings.appearanceOptions.accentGold"),
			rose: t("settings.appearanceOptions.accentRose"),
			violet: t("settings.appearanceOptions.accentViolet"),
			orange: t("settings.appearanceOptions.accentOrange"),
			blue: t("settings.appearanceOptions.accentBlue"),
			cyan: t("settings.appearanceOptions.accentCyan"),
			emerald: t("settings.appearanceOptions.accentEmerald"),
			purple: t("settings.appearanceOptions.accentPurple"),
		};
		return map[id] || ACCENTS[id]?.name || id;
	}

	let currentTheme = $derived(themeStore.theme);
	let currentAccent = $derived(themeStore.accent);
	let currentBackground = $derived(themeStore.background);

	let blurEffects = $state(settings.value.blur !== false);
	let smoothAnimations = $state(settings.value.animations !== false);
	let performanceMode = $state(settings.value.performanceMode ?? false);
	let liveWallpaper = $state(settings.value.liveWallpaper !== false);
	let soundscapesEnabled = $state(settings.value.soundscapesEnabled !== false);
	let soundscapeVolume = $state(settings.value.soundscapeVolume ?? 0.2);

	async function handleImportWallpaper() {
		try {
			const selected = await open({
				multiple: false,
				filters: [
					{
						name: uiText("ui.dd33c7dbf640a6af"),
						extensions: ["mp4", "webm", "gif", "png", "jpg", "jpeg", "webp"]
					}
				]
			});
			if (!selected || typeof selected !== "string") return;
			const isVideo = selected.toLowerCase().endsWith(".mp4") || selected.toLowerCase().endsWith(".webm");
			const managedPath = await importWallpaper(selected);
            themeStore.setCustomWallpaper(managedPath, isVideo ? "video" : "image", selected);
            await persistNow();
			toast(t("settings.appearanceOptions.wallpaperImported"), "success");
			onSave?.();
		} catch (e) {
			toast(String(e), "error");
		}
	}

	function selectTheme(tId: string) {
		themeStore.setTheme(tId);
		settings.patch({
			theme: tId === "light" ? "default-light" : "default-dark"
		});
		schedulePersist();
		toast(`${t("settings.appearanceTitle")}: ${getThemeName(tId)}`, "success");
		onSave?.();
	}

	function selectBackground(bgId: string) {
		themeStore.setBackground(bgId);
		toast(`${t("settings.appearanceOptions.launcherBg")}: ${getBgName(bgId)}`, "success");
		onSave?.();
	}

	function selectWallpaper(url: string) {
		if (!themeStore.selectWallpaper(url)) return;
		toast(t("settings.appearanceOptions.wallpaperActivated"), "success");
		onSave?.();
	}

	function removeWallpaper(url: string) {
		themeStore.removeWallpaper(url);
		toast(t("settings.appearanceOptions.wallpaperRemoved"), "info");
		onSave?.();
	}

	function selectAccent(aId: string) {
		themeStore.setAccent(aId);
		settings.patch({
			accentTheme: aId as import("$lib/stores/settings.svelte").AccentTheme
		});
		schedulePersist();
		toast(`${t("settings.appearanceOptions.accentColor")}: ${getAccentName(aId)}`, "success");
		onSave?.();
	}

	function toggleLiveWallpaper() {
		liveWallpaper = !liveWallpaper;
		settings.patch({ liveWallpaper });
		schedulePersist();
	}

	function toggleSoundscapes() {
		soundscapesEnabled = !soundscapesEnabled;
		settings.patch({ soundscapesEnabled });
		schedulePersist();
		if (soundscapesEnabled) {
			startSoundscape("overworld");
		} else {
			stopSoundscape();
		}
	}

	function handleSoundscapeVolumeChange(e: Event) {
		const target = e.target as HTMLInputElement;
		soundscapeVolume = parseFloat(target.value);
		settings.patch({ soundscapeVolume });
		schedulePersist();
		setSoundscapeVolume(soundscapeVolume);
	}

	function toggleBlur() {
		blurEffects = !blurEffects;
		settings.patch({ blur: blurEffects });
		schedulePersist();
	}

	function toggleAnimations() {
		smoothAnimations = !smoothAnimations;
		settings.patch({ animations: smoothAnimations });
		schedulePersist();
	}

	function togglePerformanceMode() {
		performanceMode = !performanceMode;
		settings.patch({
			performanceMode,
			blur: !performanceMode && blurEffects,
			animations: !performanceMode && smoothAnimations
		});
		appState.performanceMode = performanceMode;
		schedulePersist();
		toast(performanceMode ? t("settings.perfModeTitle") : t("settings.appearanceOptions.blur"), "info");
	}
</script>

<div class="space-y-6">
	<!-- Tema Base: Claro / Escuro -->
	<div>
		<div class="flex items-center gap-2 mb-2.5">
			<Palette class="w-4 h-4 text-brand-500" />
			<span class="text-xs font-bold text-fg uppercase tracking-wider">{t("settings.appearanceOptions.displayMode")}</span>
		</div>
		<div class="grid grid-cols-2 gap-3">
			{#each Object.values(THEMES) as th}
				<button
					type="button"
					class="p-4 rounded-2xl border flex items-center justify-between transition-[color,background-color,border-color,box-shadow,transform,opacity] active:scale-[0.98] cursor-pointer {currentTheme === th.id ? 'border-brand-500 bg-bg-subtle shadow-lg ring-2 ring-brand-500/30' : 'border-fg/5 bg-bg-subtle hover:border-fg/20'}"
					onclick={() => selectTheme(th.id)}
				>
					<div class="flex items-center gap-3">
						<div class="w-5 h-5 rounded-full border border-fg/20 shrink-0 shadow-inner" style="background-color: {th.previewColor};"></div>
						<span class="text-xs font-bold text-fg truncate">{getThemeName(th.id)}</span>
					</div>
					{#if currentTheme === th.id}
						<Check class="w-4 h-4 text-brand-500 stroke-[3]" />
					{/if}
				</button>
			{/each}
		</div>
	</div>

	<!-- Personalização de Fundo / Wallpaper -->
	<div>
		<div class="flex items-center gap-2 mb-2.5">
			<Image class="w-4 h-4 text-brand-500" />
			<span class="text-xs font-bold text-fg uppercase tracking-wider">{t("settings.appearanceOptions.launcherBg")}</span>
		</div>
		<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-2.5">
			<!-- Botão Importar Wallpaper Personalizado -->
			<button
				type="button"
				class="p-3 rounded-2xl border flex flex-col items-center gap-2 transition-[color,background-color,border-color,box-shadow,transform,opacity] active:scale-[0.98] cursor-pointer border-dashed border-fg/20 bg-bg-subtle/50 hover:border-brand-500/50 hover:bg-bg-subtle"
				onclick={handleImportWallpaper}
				title={t("settings.appearanceOptions.importWallpaperTooltip")}
			>
				<div class="w-full aspect-video rounded-xl border border-fg/10 bg-fg/[0.04] shadow-inner flex items-center justify-center relative overflow-hidden text-brand-500">
					<Upload class="w-5 h-5 stroke-[2]" />
				</div>
				<span class="text-[11px] font-bold text-fg/90 truncate">{t("settings.appearanceOptions.importWallpaper")}</span>
			</button>

            {#each themeStore.wallpaperLibrary as wallpaper (wallpaper.url)}
				<div class="relative min-w-0" title={wallpaper.url}>
					<button
						type="button"
						class="w-full p-3 rounded-2xl border flex flex-col items-center gap-2 transition-[color,background-color,border-color,box-shadow,transform,opacity] active:scale-[0.98] cursor-pointer {currentBackground === 'custom' && themeStore.customWallpaperUrl === wallpaper.url ? 'border-brand-500 bg-bg-subtle shadow-lg ring-2 ring-brand-500/30' : 'border-fg/5 bg-bg-subtle hover:border-fg/20'}"
						onclick={() => selectWallpaper(wallpaper.url)}
						aria-label={uiText("ui.fa331a451c4e16e1", {arg0: (wallpaper.name)})}
					>
						<div class="w-full aspect-video rounded-xl border border-fg/10 flex items-center justify-center relative overflow-hidden bg-bg">
							<WallpaperThumbnail url={wallpaper.url} type={wallpaper.type} />
							{#if currentBackground === "custom" && themeStore.customWallpaperUrl === wallpaper.url}
								<div class="absolute inset-0 bg-black/30 flex items-center justify-center">
									<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shadow-md">
										<Check class="w-3.5 h-3.5 stroke-[3]" />
									</div>
								</div>
							{/if}
						</div>
						<span class="w-full text-[11px] font-bold text-fg/90 truncate">{wallpaper.name}</span>
					</button>
					<button type="button" class={launcherButton({ variant: "danger", size: "icon", class: "absolute top-2 right-2 h-8 w-8 rounded-lg" })} onclick={() => removeWallpaper(wallpaper.url)} aria-label={uiText("ui.b4732d72f959f8b2", {arg0: (wallpaper.name)})} title={uiText("ui.965aaaed9892ba29")}>
						<Trash2 class="w-3.5 h-3.5" />
					</button>
				</div>
			{/each}

			{#each Object.values(BACKGROUNDS) as bg}
				<button
					type="button"
					class="p-3 rounded-2xl border flex flex-col items-center gap-2 transition-[color,background-color,border-color,box-shadow,transform,opacity] active:scale-[0.98] cursor-pointer {currentBackground === bg.id ? 'border-brand-500 bg-bg-subtle shadow-lg ring-2 ring-brand-500/30' : 'border-fg/5 bg-bg-subtle hover:border-fg/20'}"
					onclick={() => selectBackground(bg.id)}
				>
					<div class="w-full aspect-video rounded-xl border border-fg/10 shadow-inner flex items-center justify-center relative overflow-hidden" style="background-color: {bg.preview};">
						{#if currentBackground === bg.id}
							<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shadow-md">
								<Check class="w-3.5 h-3.5 stroke-[3]" />
							</div>
						{/if}
					</div>
					<span class="text-[11px] font-bold text-fg/90 truncate">{getBgName(bg.id)}</span>
				</button>
			{/each}
		</div>
	</div>

	<!-- Destaque Azul & Acentos -->
	<div>
		<div class="flex items-center gap-2 mb-2.5">
			<Sparkles class="w-4 h-4 text-brand-500" />
			<span class="text-xs font-bold text-fg uppercase tracking-wider">{t("settings.appearanceOptions.accentColor")}</span>
		</div>
		<div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
			{#each Object.values(ACCENTS) as ac}
				<button
					type="button"
					class="p-3 rounded-2xl border flex items-center gap-2.5 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {currentAccent === ac.id ? 'border-brand-500 bg-bg-subtle shadow-md ring-2 ring-brand-500/40' : 'border-fg/5 bg-bg-subtle hover:border-fg/20'}"
					onclick={() => selectAccent(ac.id)}
				>
					<div class="w-4 h-4 rounded-full shadow-md shrink-0 flex items-center justify-center" style="background-color: {ac.hex};"></div>
					<span class="text-xs font-bold text-fg/90 truncate">{getAccentName(ac.id)}</span>
					{#if currentAccent === ac.id}
						<Check class="w-3.5 h-3.5 text-brand-500 stroke-[3] ml-auto" />
					{/if}
				</button>
			{/each}
		</div>
	</div>

	<!-- Efeitos & Otimizações Visuais -->
	<div class="space-y-2 pt-2 border-t border-fg/5">
		{#each [
			{ title: t("settings.appearanceOptions.liveWallpaper"), desc: t("settings.appearanceOptions.liveWallpaperDesc"), val: liveWallpaper, toggle: toggleLiveWallpaper },
			{ title: t("settings.appearanceOptions.soundscapes"), desc: t("settings.appearanceOptions.soundscapesDesc"), val: soundscapesEnabled, toggle: toggleSoundscapes },
			{ title: t("settings.appearanceOptions.blur"), desc: t("settings.appearanceOptions.blurDesc"), val: blurEffects, toggle: toggleBlur },
			{ title: t("settings.appearanceOptions.animations"), desc: t("settings.appearanceOptions.animationsDesc"), val: smoothAnimations, toggle: toggleAnimations },
			{ title: t("settings.appearanceOptions.perfMode"), desc: t("settings.appearanceOptions.perfModeDesc"), val: performanceMode, toggle: togglePerformanceMode }
		] as opt}
			<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex items-center justify-between hover:border-fg/10 transition-[color,background-color,border-color,box-shadow,transform,opacity]">
				<div>
					<div class="text-xs font-bold text-fg">{opt.title}</div>
					<div class="text-[10px] text-fg/40">{opt.desc}</div>
				</div>
				<button
					type="button"
					role="switch"
					aria-label={opt.title}
					aria-checked={opt.val}
					class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-brand-500 shadow-md shadow-brand-500/40' : 'bg-bg-subtle'}"
					onclick={opt.toggle}
				>
					<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-bg-overlay' : 'translate-x-0 bg-fg'}"></span>
				</button>
			</div>
		{/each}

		{#if soundscapesEnabled}
			<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex items-center justify-between hover:border-fg/10 transition-[color,background-color,border-color,box-shadow,transform,opacity]">
				<div>
					<div class="text-xs font-bold text-fg">{t("settings.appearanceOptions.soundscapeVolume")}</div>
					<div class="text-[10px] text-fg/40">{t("settings.appearanceOptions.soundscapeVolumeDesc")} ({Math.round(soundscapeVolume * 100)}%)</div>
				</div>
				<input
					type="range"
					min="0.05"
					max="1"
					step="0.05"
					value={soundscapeVolume}
					oninput={handleSoundscapeVolumeChange}
					class="w-32 accent-brand-500 cursor-pointer"
				/>
			</div>
		{/if}
	</div>
</div>
