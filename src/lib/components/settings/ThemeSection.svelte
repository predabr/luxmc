<script lang="ts">
	import { Check, Sparkles } from "lucide-svelte";
	import { themeStore, THEMES, ACCENTS } from "$lib/stores/theme.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	type Props = {
		onSave: () => void;
	};

	let { onSave }: Props = $props();

	let draftTheme = $state(themeStore.theme);
	let draftAccent = $state(themeStore.accent);
	const hasAppearanceChanges = $derived(draftTheme !== themeStore.theme || draftAccent !== themeStore.accent);
	let blurEffects = $state(settings.value.blur !== false);
	let smoothAnimations = $state(settings.value.animations !== false);
	let mysticAuraGlow = $state(true);
	let performanceMode = $state(settings.value.performanceMode ?? false);
	let quantumParticles = $state(!settings.value.performanceMode);

	function discardAppearance() {
		draftTheme = themeStore.theme;
		draftAccent = themeStore.accent;
		toast("Alterações de aparência descartadas.", "info");
	}

	function handleSave() {
		themeStore.setTheme(draftTheme);
		themeStore.setAccent(draftAccent);
		settings.patch({
			performanceMode,
			blur: blurEffects && !performanceMode,
			animations: smoothAnimations && !performanceMode
		});
		appState.performanceMode = performanceMode;
		onSave();
	}
</script>

<div class="space-y-4">
	{#if hasAppearanceChanges}
		<div class="bg-amber-500/10 border border-amber-500/30 rounded-2xl p-3 flex items-center justify-between shadow-sm animate-fade-in">
			<div class="text-xs text-amber-300 font-bold flex items-center gap-2">
				<Sparkles class="w-4 h-4 text-amber-400" />
				<span>Você tem alterações de aparência não salvas.</span>
			</div>
			<div class="flex items-center gap-2">
				<button
					type="button"
					class="text-xs px-3 py-1.5 rounded-full bg-white/10 hover:bg-white/20 text-white font-semibold transition-all cursor-pointer"
					onclick={discardAppearance}
				>
					Descartar
				</button>
				<button
					type="button"
					class="text-xs px-4 py-1.5 rounded-full font-black text-black transition-all cursor-pointer shadow-md hover:scale-105 active:scale-95 flex items-center gap-1.5"
					style="background-color: var(--accent-color, #e2b86b);"
					onclick={handleSave}
				>
					<Check class="w-3.5 h-3.5 stroke-[3]" /> Guardar Alterações
				</button>
			</div>
		</div>
	{/if}

	<div>
		<span class="text-xs font-bold text-white block mb-2">Tema Visual do Launcher (Preto ou Branco)</span>
		<div class="grid grid-cols-2 gap-3">
			{#each Object.values(THEMES) as th}
				<button
					type="button"
					class="p-4 rounded-full border flex items-center justify-center gap-3 transition-all active:scale-95 cursor-pointer {draftTheme === th.id ? 'border-brand-500 bg-[#222328] shadow-md ring-2 ring-brand-500/30' : 'border-white/5 bg-[#1c1d22] hover:border-white/20'}"
					onclick={() => {
						draftTheme = th.id;
						toast(`Tema selecionado: ${th.name}. Clique em "Guardar alterações" para aplicar.`, "info");
					}}
				>
					<div class="w-5 h-5 rounded-full border border-white/30 shrink-0 shadow-inner" style="background-color: {th.previewColor};"></div>
					<span class="text-xs font-bold text-white truncate">{th.name}</span>
				</button>
			{/each}
		</div>
	</div>

	<div>
		<span class="text-xs font-bold text-white block mb-2">Cor de Destaque (Accent Neon)</span>
		<div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-7 gap-2">
			{#each Object.values(ACCENTS) as ac}
				<button
					type="button"
					class="p-3 rounded-2xl border flex flex-col items-center gap-1.5 transition-all cursor-pointer {draftAccent === ac.id ? 'border-brand-500 bg-[#222328] shadow-md scale-105 ring-2 ring-brand-500/40' : 'border-white/5 bg-[#1c1d22] hover:border-white/20'}"
					onclick={() => {
						draftAccent = ac.id;
						toast(`Cor de destaque: ${ac.name}. Clique em "Guardar alterações" para aplicar.`, "info");
					}}
				>
					<div class="w-5 h-5 rounded-full shadow-md" style="background-color: {ac.hex};"></div>
					<span class="text-[10px] font-bold text-white/90 truncate">{ac.name}</span>
				</button>
			{/each}
		</div>
	</div>

	<div class="space-y-2">
		{#each [
			{ title: 'Aura Mística Neon & Brilho Dourado', desc: 'Glow dinâmico e sombras holográficas nas bordas dos cartões', val: mysticAuraGlow, toggle: () => mysticAuraGlow = !mysticAuraGlow },
			{ title: 'Desfoque de Vidro Holográfico (Backdrop-blur)', desc: 'Efeito translúcido com aceleração gráfica na interface', val: blurEffects, toggle: () => blurEffects = !blurEffects },
			{ title: 'Animações Fluidas de 144Hz / Alta Taxa de Quadros', desc: 'Transições magnéticas aceleradas com curvas cúbicas suaves', val: smoothAnimations, toggle: () => smoothAnimations = !smoothAnimations },
			{ title: 'Partículas Quânticas de Fundo', desc: 'Partículas discretas flutuando no plano de fundo do launcher', val: quantumParticles, toggle: () => quantumParticles = !quantumParticles },
			{ title: 'Modo Ultra Desempenho (Desativa Efeitos)', desc: 'Remove sombras e desfoques para economizar bateria e GPU integrada', val: performanceMode, toggle: () => performanceMode = !performanceMode }
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
					class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
					onclick={opt.toggle}
				>
					<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
				</button>
			</div>
		{/each}
	</div>
</div>
