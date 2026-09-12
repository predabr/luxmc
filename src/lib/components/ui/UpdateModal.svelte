<script lang="ts">
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { DownloadCloud, Sparkles, ArrowRight, X } from "lucide-svelte";
	import { getVersion } from "@tauri-apps/api/app";
	import { openUrl } from "@tauri-apps/plugin-opener";

	let showModal = $state(false);
	let currentVersion = $state("");
	let latestVersion = $state("");
	let releaseUrl = $state("");
	let releaseNotes = $state("");
	let isChecking = $state(true);

	onMount(async () => {
		try {
			currentVersion = await getVersion();
			
			// Busca a última release no GitHub
			const res = await fetch("https://api.github.com/repos/predabr/luxmc/releases/latest");
			if (!res.ok) return;
			
			const data = await res.json();
			const tag = data.tag_name; // ex: "v0.2.0"
			latestVersion = tag.replace("v", "");
			
			if (isNewerVersion(currentVersion, latestVersion)) {
				releaseUrl = data.html_url;
				releaseNotes = data.body || "Atualização de melhorias e performance!";
				
				// Espera 2 segundos antes de mostrar pra cutscene passar
				setTimeout(() => {
					showModal = true;
				}, 2000);
			}
		} catch (error) {
			console.error("Failed to check for updates:", error);
		} finally {
			isChecking = false;
		}
	});

	function isNewerVersion(current: string, latest: string) {
		const cleanParts = (v: string) =>
			v.replace(/^v/i, "").split("-")[0].split(".").map((x) => parseInt(x, 10) || 0);
		const currParts = cleanParts(current);
		const latestParts = cleanParts(latest);
		
		for (let i = 0; i < Math.max(currParts.length, latestParts.length); i++) {
			const c = currParts[i] || 0;
			const l = latestParts[i] || 0;
			if (l > c) return true;
			if (l < c) return false;
		}
		return false;
	}

	function handleUpdate() {
		openUrl(releaseUrl);
		showModal = false;
	}
</script>

{#if showModal}
	<div class="fixed inset-0 z-[100] flex items-center justify-center p-6 bg-black/80 backdrop-blur-md" in:fade={{ duration: 300 }} out:fade={{ duration: 200 }}>
		<div class="relative w-full max-w-lg bg-[#18191c] border border-brand-500/30 rounded-3xl p-8 shadow-2xl overflow-hidden" in:scale={{ start: 0.95, duration: 300, opacity: 0 }}>
			
			<!-- Glow de fundo -->
			<div class="absolute -top-20 -right-20 w-64 h-64 bg-brand-500/20 rounded-full blur-3xl pointer-events-none"></div>
			<div class="absolute -bottom-20 -left-20 w-64 h-64 bg-brand-500/10 rounded-full blur-3xl pointer-events-none"></div>

			<!-- Close Button -->
			<button 
				class="absolute top-4 right-4 p-2 rounded-full text-white/50 hover:text-white hover:bg-white/10 transition-colors"
				onclick={() => showModal = false}
			>
				<X class="w-5 h-5" />
			</button>

			<div class="relative flex flex-col items-center text-center space-y-6">
				
				<!-- Ícone -->
				<div class="w-20 h-20 bg-brand-500/15 border-2 border-brand-500/30 rounded-full flex items-center justify-center shadow-[0_0_30px_rgba(var(--brand-500-rgb),0.3)]">
					<DownloadCloud class="w-10 h-10 text-brand-500" />
				</div>

				<!-- Títulos -->
				<div>
					<h2 class="text-2xl font-black text-white flex items-center justify-center gap-2">
						<Sparkles class="w-5 h-5 text-brand-500" /> Nova Versão Disponível!
					</h2>
					<p class="text-sm text-white/60 mt-2">O Luxmc ficou ainda melhor. É hora de atualizar!</p>
				</div>

				<!-- Versões -->
				<div class="flex items-center justify-center gap-4 w-full bg-black/40 rounded-2xl p-4 border border-white/5">
					<div class="flex flex-col items-center">
						<span class="text-[10px] text-white/40 font-bold uppercase tracking-widest">Sua Versão</span>
						<span class="text-lg font-mono text-white/80 font-bold">v{currentVersion}</span>
					</div>
					<ArrowRight class="w-5 h-5 text-white/30" />
					<div class="flex flex-col items-center">
						<span class="text-[10px] text-brand-500 font-bold uppercase tracking-widest">Nova Versão</span>
						<span class="text-lg font-mono text-brand-500 font-black">v{latestVersion}</span>
					</div>
				</div>

				<!-- Release Notes -->
				<div class="w-full text-left bg-white/5 rounded-2xl p-4 max-h-32 overflow-y-auto custom-scrollbar border border-white/5">
					<span class="text-[10px] text-white/40 font-bold uppercase tracking-widest block mb-2">Novidades:</span>
					<p class="text-xs text-white/80 whitespace-pre-line leading-relaxed">
						{releaseNotes}
					</p>
				</div>

				<!-- Botão de Ação -->
				<button 
					class="w-full py-3.5 bg-brand-500 hover:brightness-110 text-black font-black rounded-2xl transition-all hover:scale-[1.02] active:scale-[0.98] flex items-center justify-center gap-2 shadow-[0_0_20px_rgba(var(--brand-500-rgb),0.4)]"
					style="background-color: var(--accent-color);"
					onclick={handleUpdate}
				>
					Baixar Atualização <DownloadCloud class="w-4 h-4" />
				</button>
				
				<button 
					class="text-xs font-bold text-white/40 hover:text-white transition-colors"
					onclick={() => showModal = false}
				>
					Lembrar mais tarde
				</button>
			</div>
		</div>
	</div>
{/if}
