<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { Play, Pause, RotateCcw } from "lucide-svelte";
	import WaveSurfer from "wavesurfer.js";

	type Props = {
		url: string;
		title?: string;
	};

	let { url, title = uiText("ui.a7cde0366e7b9efb") }: Props = $props();

	let container = $state<HTMLDivElement | null>(null);
	let isPlaying = $state(false);
	let duration = $state(0);
	let currentTime = $state(0);
	let wavesurfer: WaveSurfer | null = null;

	$effect(() => {
		if (!container || !url) return;

		const colors = getComputedStyle(document.documentElement);
        const accent = colors.getPropertyValue("--brand-400").trim();
        const foreground = colors.getPropertyValue("--fg").trim();
		const ws = WaveSurfer.create({
			container,
			waveColor: `rgb(${accent} / 0.25)`,
			progressColor: `rgb(${accent})`,
			cursorColor: `rgb(${foreground})`,
			barWidth: 2,
			barGap: 3,
			barRadius: 2,
			height: 48,
			url
		});

		ws.on("ready", () => {
			duration = ws.getDuration();
		});

		ws.on("timeupdate", (time) => {
			currentTime = time;
		});

		ws.on("play", () => {
			isPlaying = true;
		});

		ws.on("pause", () => {
			isPlaying = false;
		});

		ws.on("finish", () => {
			isPlaying = false;
		});

		wavesurfer = ws;

		return () => {
			try {
				ws.destroy();
			} catch {}
			wavesurfer = null;
		};
	});

	function togglePlay() {
		wavesurfer?.playPause();
	}

	function restart() {
		wavesurfer?.seekTo(0);
	}

	function formatTime(secs: number) {
		const m = Math.floor(secs / 60);
		const s = Math.floor(secs % 60);
		return `${m}:${s < 10 ? "0" : ""}${s}`;
	}
</script>

<div class="bg-bg-subtle border border-fg/10 rounded-2xl p-4 flex flex-col gap-3">
	<div class="flex items-center justify-between text-xs font-semibold text-fg/80">
		<span>{title}</span>
		<span class="font-mono text-fg/50">{formatTime(currentTime)} / {formatTime(duration)}</span>
	</div>

	<div bind:this={container} class="w-full h-12"></div>

	<div class="flex items-center gap-2">
		<button
			type="button"
			class={launcherButton({ variant: "ghostBrand", size: "sm", class: "flex items-center gap-1.5" })}
			onclick={togglePlay}
		>
			{#if isPlaying}
				<Pause class="w-3.5 h-3.5" />
				<span>{uiText("ui.cd25ee5b1db8f5b0")}</span>
			{:else}
				<Play class="w-3.5 h-3.5" />
				<span>{uiText("ui.6e11a30e8703bef8")}</span>
			{/if}
		</button>

		<button
			type="button"
			class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
			onclick={restart}
			aria-label={uiText("ui.0b81a13d6a01cb50")}
		>
			<RotateCcw class="w-3.5 h-3.5" />
		</button>
	</div>
</div>
