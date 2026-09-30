<script lang="ts">
	import { onMount } from "svelte";
	import { appState } from "$lib/stores/app.svelte";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let leaving = $state(false);
	let finished = false;
	let closeTimer: ReturnType<typeof setTimeout>;
	const destination = $derived(appState.activeGameDetails?.name);

	function finish(immediate = false) {
		if (finished) return;
		finished = true;
		if (immediate) {
			onComplete();
			return;
		}
		leaving = true;
		closeTimer = setTimeout(onComplete, 1000);
	}

	onMount(() => {
		const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
		const timer = setTimeout(() => finish(false), reduced ? 150 : 2500);

		const key = (event: KeyboardEvent) => {
			if (["Escape", "Enter", " "].includes(event.key)) {
				event.preventDefault();
				finish(true);
			}
		};
		window.addEventListener("keydown", key);
		return () => {
			clearTimeout(timer);
			clearTimeout(closeTimer);
			window.removeEventListener("keydown", key);
		};
	});
</script>

<div
	class="intro fixed inset-0 z-[99999] overflow-hidden bg-bg text-fg transition-opacity duration-700 ease-out select-none"
	class:opacity-0={leaving}
	class:pointer-events-none={leaving}
	role="status"
	aria-label="Abrindo Luxmc"
>
	<div class="cutscene-background absolute inset-0 pointer-events-none">
		<div class="absolute inset-0 bg-gradient-to-br from-brand-500/10 via-bg to-success/5"></div>
		<div class="cutscene-grid absolute inset-0 opacity-40"></div>
		<div class="absolute inset-0 bg-gradient-to-t from-bg via-bg/35 to-transparent"></div>
	</div>

	<div class="absolute left-8 right-8 top-8 flex items-center justify-between text-[10px] uppercase tracking-[.25em] text-fg-muted z-20">
		<span class="font-bold">Luxmc Launcher</span>
		<span class="font-mono text-fg/40">Java Edition</span>
	</div>

	<div
		class="absolute inset-0 flex flex-col items-center justify-center px-8 z-10 transition-opacity duration-700 ease-out"
		class:opacity-0={leaving}
	>
		<div class="portal mb-8 relative h-28 w-28 sm:h-36 sm:w-36 flex items-center justify-center">
			<div class="portal-frame absolute inset-0 rounded-[2rem] border border-brand-300/30 bg-bg-elevated/80 shadow-glow"></div>
			<div class="absolute inset-3 rounded-2xl border border-fg/10 bg-bg"></div>
			<img loading="lazy" decoding="async"
				src="/logo.png"
				alt="Luxmc"
				class="emblem relative h-16 w-16 sm:h-20 sm:w-20 object-contain"
			/>
		</div>

		<div class="wordmark text-center">
			<p class="text-[10px] font-bold uppercase tracking-[.45em] text-brand-300">Explore · Construa · Compartilhe</p>
			<h1 class="mt-4 text-5xl sm:text-7xl font-black tracking-[-.05em] text-fg">Seu próximo mundo<span class="text-brand-400">.</span></h1>
			<p class="mt-4 max-w-md truncate px-4 text-xs text-fg/60 font-medium">
				{destination || "Inicializando ambiente e carregando recursos..."}
			</p>
		</div>

		<div class="mt-8 h-1 w-44 rounded-full overflow-hidden bg-fg/5 border border-fg/10">
			<div class="light h-full w-full origin-left bg-gradient-to-r from-brand-500 via-brand-300 to-emerald-400"></div>
		</div>
	</div>

	<div class="absolute bottom-6 inset-x-8 flex items-center justify-between text-[10px] uppercase tracking-[.2em] text-fg-muted z-20">
		<span>Feito para jogar do seu jeito</span>
		<button
			type="button"
			class="rounded-xl border border-fg/15 px-4 py-2 hover:bg-fg/10 hover:text-fg focus-visible:outline-brand-400 transition-colors cursor-pointer"
			onclick={() => finish(true)}
		>
			Pular <span class="ml-2 text-fg/40 font-mono">Esc</span>
		</button>
	</div>
</div>

<style>
	.cutscene-background {
		background-image:
			radial-gradient(circle at 16% 18%, rgb(var(--brand-500) / 0.16), transparent 34%),
			radial-gradient(circle at 82% 74%, rgb(var(--success) / 0.1), transparent 32%);
	}
	.cutscene-grid {
		background-image:
			linear-gradient(rgb(var(--fg) / 0.035) 1px, transparent 1px),
			linear-gradient(90deg, rgb(var(--fg) / 0.035) 1px, transparent 1px);
		background-size: 42px 42px;
	}

	.portal {
		animation: arrival 1.2s cubic-bezier(.2, .7, .2, 1) both;
	}
	.portal-frame {
		animation: portal 1.8s cubic-bezier(.2, .7, .2, 1) both;
	}
	.wordmark {
		animation: arrival 1.2s .2s cubic-bezier(.2, .7, .2, 1) both;
	}
	.light {
		animation: reveal 2.5s cubic-bezier(.2, .7, .2, 1) both;
	}

	@keyframes portal {
		from { transform: rotate(-25deg) scale(.8); opacity: 0; }
		to { transform: rotate(0deg) scale(1); opacity: 1; }
	}
	@keyframes arrival {
		from { opacity: 0; transform: translateY(20px); }
		to { opacity: 1; transform: translateY(0); }
	}
	@keyframes reveal {
		from { transform: scaleX(0); }
		to { transform: scaleX(1); }
	}

	@media (prefers-reduced-motion: reduce) {
		.portal, .portal-frame, .wordmark, .light {
			animation: none;
		}
	}
</style>
