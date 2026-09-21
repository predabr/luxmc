<script lang="ts">
	import { Check, Sparkles, Image, Palette, Eye, Upload, Film } from "lucide-svelte";
	import { themeStore, THEMES, ACCENTS, BACKGROUNDS } from "$lib/stores/theme.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { schedulePersist } from "$lib/stores/persistence.svelte";
	import { startSoundscape, stopSoundscape, setSoundscapeVolume } from "$lib/utils/sound";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";

	type Props = {
		onSave?: () => void;
	};

	let { onSave }: Props = $props();

	let currentTheme = $derived(themeStore.theme);
	let currentAccent = $derived(themeStore.accent);
	let currentBackground = $derived(themeStore.background);

	let blurEffects = $state(settings.value.blur !== false);
	let smoothAnimations = $state(settings.value.animations !== false);
	let mysticAuraGlow = $state(true);
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
						name: "Wallpaper (Vídeo, GIF ou Imagem)",
						extensions: ["mp4", "webm", "gif", "png", "jpg", "jpeg", "webp"]
					}
				]
			});
			if (!selected || typeof selected !== "string") return;
			const isVideo = selected.toLowerCase().endsWith(".mp4") || selected.toLowerCase().endsWith(".webm");
			const dataUrl = isVideo
				? `http://127.0.0.1:49152/media?path=${encodeURIComponent(selected)}`
				: convertFileSrc(selected);
			themeStore.setCustomWallpaper(dataUrl, isVideo ? "video" : "image");
			toast("Wallpaper importado com sucesso! Exibindo perfeitamente recortado.", "success");
			onSave?.();
		} catch (e) {
			toast("Erro ao importar wallpaper: " + String(e), "error");
		}
	}

	function selectTheme(tId: string) {
		themeStore.setTheme(tId);
		settings.patch({
			theme: tId === "light" ? "default-light" : "default-dark"
		});
		schedulePersist();
		toast(`Tema alterado para ${THEMES[tId]?.name ?? tId}`, "success");
		onSave?.();
	}

	function selectBackground(bgId: string) {
		themeStore.setBackground(bgId);
		toast(`Plano de fundo alterado para ${BACKGROUNDS[bgId]?.name ?? bgId}`, "success");
		onSave?.();
	}

	function selectAccent(aId: string) {
		themeStore.setAccent(aId);
		settings.patch({
			accentTheme: aId as import("$lib/stores/settings.svelte").AccentTheme
		});
		schedulePersist();
		toast(`Destaque alterado para ${ACCENTS[aId]?.name ?? aId}`, "success");
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
		toast(performanceMode ? "Modo Ultra Desempenho ativado" : "Efeitos visuais restaurados", "info");
	}
</script>

<div class="space-y-6">
	<!-- Tema Base: Claro / Escuro -->
	<div>
		<div class="flex items-center gap-2 mb-2.5">
			<Palette class="w-4 h-4 text-brand-500" />
			<span class="text-xs font-bold text-fg uppercase tracking-wider">Modo de Exibição</span>
		</div>
		<div class="grid grid-cols-2 gap-3">
			{#each Object.values(THEMES) as th}
				<button
					type="button"
					class="p-4 rounded-2xl border flex items-center justify-between transition-all active:scale-[0.98] cursor-pointer {currentTheme === th.id ? 'border-brand-500 bg-bg-subtle shadow-lg ring-2 ring-brand-500/30' : 'border-fg/5 bg-bg-subtle hover:border-fg/20'}"
					onclick={() => selectTheme(th.id)}
				>
					<div class="flex items-center gap-3">
						<div class="w-5 h-5 rounded-full border border-fg/20 shrink-0 shadow-inner" style="background-color: {th.previewColor};"></div>
						<span class="text-xs font-bold text-fg truncate">{th.name}</span>
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
			<span class="text-xs font-bold text-fg uppercase tracking-wider">Plano de Fundo do Launcher</span>
		</div>
		<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-7 gap-2.5">
			<!-- Botão Importar Wallpaper Personalizado -->
			<button
				type="button"
				class="p-3 rounded-2xl border flex flex-col items-center gap-2 transition-all active:scale-[0.98] cursor-pointer border-dashed border-fg/20 bg-bg-subtle/50 hover:border-brand-500/50 hover:bg-bg-subtle"
				onclick={handleImportWallpaper}
				title="Importe qualquer vídeo MP4/WebM ou imagem PNG/JPG/GIF"
			>
				<div class="w-full h-12 rounded-xl border border-fg/10 bg-fg/[0.04] shadow-inner flex items-center justify-center relative overflow-hidden text-brand-500">
					<Upload class="w-5 h-5 stroke-[2]" />
				</div>
				<span class="text-[11px] font-bold text-fg/90 truncate">+ Importar</span>
			</button>

			<!-- Card de Wallpaper Personalizado Importado -->
			{#if themeStore.customWallpaperUrl}
				<button
					type="button"
					class="p-3 rounded-2xl border flex flex-col items-center gap-2 transition-all active:scale-[0.98] cursor-pointer {currentBackground === 'custom' ? 'border-brand-500 bg-bg-subtle shadow-lg ring-2 ring-brand-500/30' : 'border-fg/5 bg-bg-subtle hover:border-fg/20'}"
					onclick={() => {
						themeStore.setBackground("custom");
						toast("Wallpaper personalizado ativado!", "success");
						onSave?.();
					}}
					title="Seu wallpaper importado"
				>
					<div class="w-full h-12 rounded-xl border border-fg/10 shadow-inner flex items-center justify-center relative overflow-hidden bg-black/40">
						{#if themeStore.customWallpaperType === "video"}
							<Film class="w-5 h-5 text-brand-400 opacity-80" />
						{:else}
							<img src={themeStore.customWallpaperUrl} alt="Preview" class="w-full h-full object-cover" />
						{/if}
						{#if currentBackground === "custom"}
							<div class="absolute inset-0 bg-black/30 flex items-center justify-center">
								<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shadow-md">
									<Check class="w-3.5 h-3.5 stroke-[3]" />
								</div>
							</div>
						{/if}
					</div>
					<span class="text-[11px] font-bold text-fg/90 truncate">Personalizado</span>
				</button>
			{/if}

			{#each Object.values(BACKGROUNDS) as bg}
				<button
					type="button"
					class="p-3 rounded-2xl border flex flex-col items-center gap-2 transition-all active:scale-[0.98] cursor-pointer {currentBackground === bg.id ? 'border-brand-500 bg-bg-subtle shadow-lg ring-2 ring-brand-500/30' : 'border-fg/5 bg-bg-subtle hover:border-fg/20'}"
					onclick={() => selectBackground(bg.id)}
				>
					<div class="w-full h-12 rounded-xl border border-fg/10 shadow-inner flex items-center justify-center relative overflow-hidden" style="background-color: {bg.preview};">
						{#if currentBackground === bg.id}
							<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shadow-md">
								<Check class="w-3.5 h-3.5 stroke-[3]" />
							</div>
						{/if}
					</div>
					<span class="text-[11px] font-bold text-fg/90 truncate">{bg.name}</span>
				</button>
			{/each}
		</div>
	</div>

	<!-- Destaque Azul & Acentos -->
	<div>
		<div class="flex items-center gap-2 mb-2.5">
			<Sparkles class="w-4 h-4 text-brand-500" />
			<span class="text-xs font-bold text-fg uppercase tracking-wider">Cor de Destaque</span>
		</div>
		<div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
			{#each Object.values(ACCENTS) as ac}
				<button
					type="button"
					class="p-3 rounded-2xl border flex items-center gap-2.5 transition-all cursor-pointer {currentAccent === ac.id ? 'border-brand-500 bg-bg-subtle shadow-md ring-2 ring-brand-500/40' : 'border-fg/5 bg-bg-subtle hover:border-fg/20'}"
					onclick={() => selectAccent(ac.id)}
				>
					<div class="w-4 h-4 rounded-full shadow-md shrink-0 flex items-center justify-center" style="background-color: {ac.hex};"></div>
					<span class="text-xs font-bold text-fg/90 truncate">{ac.name}</span>
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
			{ title: 'Aura Mística Neon & Sombras', desc: 'Glow dinâmico e sombras holográficas com aceleração GPU', val: mysticAuraGlow, toggle: () => mysticAuraGlow = !mysticAuraGlow },
			{ title: 'Live Wallpaper Interativo 3D', desc: 'Campo volumétrico dinâmico de partículas no plano de fundo', val: liveWallpaper, toggle: toggleLiveWallpaper },
			{ title: 'Atmosfera Acústica Procedural (Soundscapes)', desc: 'Paisagem sonora ambiente imersiva gerada via Web Audio API', val: soundscapesEnabled, toggle: toggleSoundscapes },
			{ title: 'Desfoque de Vidro (Backdrop-blur)', desc: 'Efeito translúcido de alta fidelidade na interface e painéis', val: blurEffects, toggle: toggleBlur },
			{ title: 'Animações Fluidas de 144Hz / Alta Taxa de Quadros', desc: 'Transições magnéticas aceleradas com curvas cúbicas suaves', val: smoothAnimations, toggle: toggleAnimations },
			{ title: 'Modo Ultra Desempenho (Desativa Efeitos)', desc: 'Remove sombras e desfoques para economizar bateria e GPU integrada', val: performanceMode, toggle: togglePerformanceMode }
		] as opt}
			<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex items-center justify-between hover:border-fg/10 transition-all">
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
			<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex items-center justify-between hover:border-fg/10 transition-all">
				<div>
					<div class="text-xs font-bold text-fg">Volume da Atmosfera Acústica</div>
					<div class="text-[10px] text-fg/40">Intensidade do sintetizador de ambiente ({Math.round(soundscapeVolume * 100)}%)</div>
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
