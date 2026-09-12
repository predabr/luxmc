<script lang="ts">
	import { Check, Sparkles } from "lucide-svelte";
	import { themeStore, THEMES, ACCENTS } from "$lib/stores/theme.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { schedulePersist } from "$lib/stores/persistence.svelte";

	type Props = {
		onSave?: () => void;
	};

	let { onSave }: Props = $props();

	let currentTheme = $derived(themeStore.theme);
	let currentAccent = $derived(themeStore.accent);
	let blurEffects = $state(settings.value.blur !== false);
	let smoothAnimations = $state(settings.value.animations !== false);
	let mysticAuraGlow = $state(true);
	let performanceMode = $state(settings.value.performanceMode ?? false);
	let quantumParticles = $state(!settings.value.performanceMode);

	function selectTheme(tId: string) {
		themeStore.setTheme(tId);
		settings.patch({
			theme: tId === "light" ? "default-light" : "default-dark"
		});
		schedulePersist();
		toast(`Tema alterado para ${THEMES[tId]?.name ?? tId}`, "success");
		onSave?.();
	}

	function selectAccent(aId: string) {
		themeStore.setAccent(aId);
		settings.patch({
			accentTheme: aId as any
		});
		schedulePersist();
		toast(`Destaque alterado para ${ACCENTS[aId]?.name ?? aId}`, "success");
		onSave?.();
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
		quantumParticles = !performanceMode;
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
	<div>
		<span class="text-xs font-bold text-white block mb-2">Tema Visual do Launcher</span>
		<div class="grid grid-cols-2 gap-3">
			{#each Object.values(THEMES) as th}
				<button
					type="button"
					class="p-4 rounded-2xl border flex items-center justify-center gap-3 transition-all active:scale-95 cursor-pointer {currentTheme === th.id ? 'border-brand-500 bg-[#222328] shadow-md ring-2 ring-brand-500/30' : 'border-white/5 bg-[#1c1d22] hover:border-white/20'}"
					onclick={() => selectTheme(th.id)}
				>
					<div class="w-5 h-5 rounded-full border border-white/30 shrink-0 shadow-inner" style="background-color: {th.previewColor};"></div>
					<span class="text-xs font-bold text-white truncate">{th.name}</span>
					{#if currentTheme === th.id}
						<Check class="w-4 h-4 text-brand-500 stroke-[3]" />
					{/if}
				</button>
			{/each}
		</div>
	</div>

	<div>
		<span class="text-xs font-bold text-white block mb-2">Cor de Destaque Neon</span>
		<div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-7 gap-2">
			{#each Object.values(ACCENTS) as ac}
				<button
					type="button"
					class="p-3 rounded-2xl border flex flex-col items-center gap-1.5 transition-all cursor-pointer {currentAccent === ac.id ? 'border-brand-500 bg-[#222328] shadow-md scale-105 ring-2 ring-brand-500/40' : 'border-white/5 bg-[#1c1d22] hover:border-white/20'}"
					onclick={() => selectAccent(ac.id)}
				>
					<div class="w-5 h-5 rounded-full shadow-md relative flex items-center justify-center" style="background-color: {ac.hex};">
						{#if currentAccent === ac.id}
							<Check class="w-3 h-3 text-black stroke-[3]" />
						{/if}
					</div>
					<span class="text-[10px] font-bold text-white/90 truncate">{ac.name}</span>
				</button>
			{/each}
		</div>
	</div>

	<div class="space-y-2">
		{#each [
			{ title: 'Aura Mística Neon & Brilho Dourado', desc: 'Glow dinâmico e sombras holográficas nas bordas dos cartões', val: mysticAuraGlow, toggle: () => mysticAuraGlow = !mysticAuraGlow },
			{ title: 'Desfoque de Vidro Holográfico (Backdrop-blur)', desc: 'Efeito translúcido com aceleração gráfica na interface', val: blurEffects, toggle: toggleBlur },
			{ title: 'Animações Fluidas de 144Hz / Alta Taxa de Quadros', desc: 'Transições magnéticas aceleradas com curvas cúbicas suaves', val: smoothAnimations, toggle: toggleAnimations },
			{ title: 'Partículas Quânticas de Fundo', desc: 'Partículas discretas flutuando no plano de fundo do launcher', val: quantumParticles, toggle: () => quantumParticles = !quantumParticles },
			{ title: 'Modo Ultra Desempenho (Desativa Efeitos)', desc: 'Remove sombras e desfoques para economizar bateria e GPU integrada', val: performanceMode, toggle: togglePerformanceMode }
		] as opt}
			<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3 flex items-center justify-between hover:border-white/10 transition-all">
				<div>
					<div class="text-xs font-bold text-white">{opt.title}</div>
					<div class="text-[10px] text-white/40">{opt.desc}</div>
				</div>
				<button
					type="button"
					role="switch"
					aria-label={opt.title}
					aria-checked={opt.val}
					class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-brand-500 shadow-[0_0_12px_rgba(226,184,107,0.4)]' : 'bg-[#383a42]'}"
					onclick={opt.toggle}
				>
					<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-black' : 'translate-x-0 bg-white'}"></span>
				</button>
			</div>
		{/each}
	</div>
</div>
