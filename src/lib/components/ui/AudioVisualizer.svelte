<script lang="ts">
	import { Play, Pause, RotateCcw } from "lucide-svelte";
	import WaveSurfer from "wavesurfer.js";

	type Props = {
		url: string;
		title?: string;
	};

	let { url, title = "Pré-visualização de Áudio" }: Props = $props();

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
			class="px-3 py-1.5 rounded-xl bg-brand-500/20 hover:bg-brand-500/30 text-brand-300 text-xs font-bold transition flex items-center gap-1.5 cursor-pointer"
			onclick={togglePlay}
		>
			{#if isPlaying}
				<Pause class="w-3.5 h-3.5" />
				<span>Pausar</span>
			{:else}
				<Play class="w-3.5 h-3.5" />
				<span>Tocar</span>
			{/if}
		</button>

		<button
			type="button"
			class="p-1.5 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg/70 text-xs transition cursor-pointer"
			onclick={restart}
			aria-label="Reiniciar áudio"
		>
			<RotateCcw class="w-3.5 h-3.5" />
		</button>
	</div>
</div>
