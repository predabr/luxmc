<script lang="ts">
	import { FolderOpen, Sparkles } from "lucide-svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { open } from "@tauri-apps/plugin-dialog";

	type Props = {
		onSave: () => void;
	};

	let { onSave }: Props = $props();

	let javaPath = $state(settings.value.javaPath || "/usr/bin/java");
	let minRam = $state(settings.value.minRamMb || 2048);
	let maxRam = $state(settings.value.maxRamMb || 6144);
	let selectedGc = $state("aikar");
	let jvmArgs = $state(settings.value.jvmArgs || "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200");
	let disableExplicitGc = $state(true);
	let parallelRefProc = $state(true);
	let tieredCompilation = $state(true);

	function autoOptimizeRam() {
		minRam = 2048;
		maxRam = 6144;
		selectedGc = "aikar";
		jvmArgs = "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC";
		disableExplicitGc = true;
		parallelRefProc = true;
		tieredCompilation = true;
		toast("RAM e Flags do GC otimizadas com sucesso para Linux!", "success");
	}

	async function browseJava() {
		try {
			const selected = await open({ directory: false, multiple: false });
			if (selected && typeof selected === "string") {
				javaPath = selected;
				toast(`Java selecionado: ${selected}`, "success");
			}
		} catch (e) {
			toast(String(e), "error");
		}
	}
</script>

<div class="space-y-4">
	<div class="bg-gradient-to-r from-[#1c1d22] to-[#25262c] border border-brand-500/30 rounded-2xl p-4 flex items-center justify-between shadow-md">
		<div>
			<div class="text-xs font-black text-white flex items-center gap-2">
				<Sparkles class="w-4 h-4 text-brand-500" /> Otimizar RAM e GC Automaticamente
			</div>
			<div class="text-[10px] text-white/60 mt-0.5">Detecta sua memória instalada e aplica as Aikar's Flags ideais para eliminar engasgos de FPS</div>
		</div>
		<button
			class="bg-brand-500 hover:bg-[#ebd095] active:scale-95 text-black font-black text-xs px-4 py-2.5 rounded-xl transition-all cursor-pointer shadow-md shrink-0 flex items-center gap-1.5"
			onclick={autoOptimizeRam}
		>
			<Sparkles class="w-3.5 h-3.5" /> Otimizar Agora
		</button>
	</div>

	<div>
		<span class="text-xs font-bold text-white block mb-1.5">Executável Java do Sistema</span>
		<div class="flex gap-2">
			<input type="text" bind:value={javaPath} class="flex-1 bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white focus:border-brand-500 focus:outline-none font-mono" />
			<button class="bg-[#24252a] hover:bg-white/10 active:scale-95 text-white text-xs font-bold px-4 py-2 rounded-2xl border border-white/10 flex items-center gap-1.5 cursor-pointer" onclick={browseJava}>
				<FolderOpen class="w-3.5 h-3.5" /> Procurar
			</button>
		</div>
	</div>

	<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-4 space-y-3">
		<div class="flex items-center justify-between">
			<span class="text-xs font-bold text-white">Alocação de Memória RAM Máxima (Xmx)</span>
			<span class="text-xs font-mono font-bold text-brand-500">{maxRam} MB ({Math.round(maxRam / 1024)} GB)</span>
		</div>

		<input
			type="range"
			min="1024"
			max="16384"
			step="512"
			bind:value={maxRam}
			class="w-full accent-brand-500 cursor-pointer"
		/>

		<div class="flex justify-between text-[10px] text-white/30 font-mono font-bold">
			<span>1 GB</span>
			<span>4 GB</span>
			<span>8 GB</span>
			<span>12 GB</span>
			<span>16 GB</span>
		</div>
	</div>

	<div>
		<span class="text-xs font-bold text-white block mb-1.5">Argumentos JVM Customizados (Flags)</span>
		<input type="text" bind:value={jvmArgs} class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white font-mono focus:border-brand-500 focus:outline-none" />
	</div>

	<div class="space-y-2">
		{#each [
			{ title: 'Desativar Chamadas Explícitas de GC (-XX:+DisableExplicitGC)', desc: 'Impede que mods forcem paradas de coleta e causem travamentos no jogo', val: disableExplicitGc, toggle: () => disableExplicitGc = !disableExplicitGc },
			{ title: 'Processamento Paralelo de Referências (-XX:+ParallelRefProcEnabled)', desc: 'Distribui a limpeza de referências fracas por todos os núcleos da CPU', val: parallelRefProc, toggle: () => parallelRefProc = !parallelRefProc },
			{ title: 'Compilação JIT Tiered (-XX:+TieredCompilation)', desc: 'Compilação nativa em múltiplos níveis para carregamento rápido', val: tieredCompilation, toggle: () => tieredCompilation = !tieredCompilation }
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
