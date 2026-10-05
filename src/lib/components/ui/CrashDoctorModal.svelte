<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { fade, scale } from "svelte/transition";
	import { goto } from "$app/navigation";
	import { 
		AlertTriangle, 
		CheckCircle2, 
		Wrench, 
		FileText, 
		X, 
		ChevronDown, 
		ChevronUp,
		Zap,
		ShieldAlert
	} from "lucide-svelte";
	import { crashDoctor } from "$lib/stores/crashDoctor.svelte";

	let showSnippet = $state(false);

	const diagnosis = $derived(crashDoctor.diagnosis);

	async function handleFix() {
		await crashDoctor.applyFix();
	}

	function goToLogs() {
		crashDoctor.close();
		goto("/logs");
	}
</script>

{#if crashDoctor.isOpen && diagnosis}
	<div 
		class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/75 backdrop-blur-md"
		transition:fade={{ easing: quintOut, duration: 220 }}
	>
		<div 
			class="relative w-full max-w-2xl bg-bg-elevated border border-amber-500/30 rounded-3xl p-6 md:p-8 shadow-2xl overflow-hidden"
			transition:scale={{ easing: backOut, duration: 260, start: 0.95 }}
		>
			<div class="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-amber-500 via-red-500 to-amber-500"></div>

			<div class="flex items-start justify-between gap-4 mb-6">
				<div class="flex items-center gap-3">
					<div class="w-12 h-12 rounded-2xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
						<ShieldAlert class="w-6 h-6" />
					</div>
					<div>
						<span class="text-[10px] font-black uppercase tracking-wider text-amber-400 bg-amber-500/10 px-2.5 py-0.5 rounded-full border border-amber-500/20">
							{uiText("ui.6e6395afe8f4010d")}
						</span>
						<h3 class="text-lg font-black text-fg mt-1">
							{diagnosis.title}
						</h3>
					</div>
				</div>

				<button 
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					onclick={() => crashDoctor.close()}
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			<div class="space-y-4">
				<div class="bg-red-500/10 border border-red-500/20 rounded-2xl p-4 text-xs text-red-200 leading-relaxed">
					<div class="flex items-center gap-2 font-bold text-red-400 mb-1">
						<AlertTriangle class="w-4 h-4 shrink-0" />
						{uiText("ui.b09ca51e68b1750c")}
					</div>
					{diagnosis.message}
				</div>

				<div class="bg-emerald-500/10 border border-emerald-500/20 rounded-2xl p-4 text-xs text-emerald-200 leading-relaxed">
					<div class="flex items-center gap-2 font-bold text-emerald-400 mb-1">
						<CheckCircle2 class="w-4 h-4 shrink-0" />
						{uiText("ui.1e09f1225b2e98c6")}
					</div>
					{diagnosis.solution}
				</div>

				{#if diagnosis.logSnippet}
					<div class="border border-fg/5 rounded-2xl bg-bg-overlay/40 overflow-hidden text-xs">
						<button 
							type="button"
							class={launcherButton({ variant: "ghost", size: "sm", class: "w-full flex items-center justify-between" })}
							onclick={() => showSnippet = !showSnippet}
						>
							<span class="flex items-center gap-2 font-mono text-[11px]">
								<FileText class="w-3.5 h-3.5 text-amber-400" />
								{uiText("ui.609466ff1e25d164")}
							</span>
							{#if showSnippet}
								<ChevronUp class="w-4 h-4" />
							{:else}
								<ChevronDown class="w-4 h-4" />
							{/if}
						</button>
						{#if showSnippet}
							<pre class="p-3.5 pt-0 text-[10px] font-mono text-amber-200/80 overflow-x-auto whitespace-pre-wrap leading-relaxed max-h-48 custom-scrollbar border-t border-fg/5">
								{diagnosis.logSnippet}
							</pre>
						{/if}
					</div>
				{/if}
			</div>

			<div class="flex flex-wrap items-center justify-end gap-3 mt-6 pt-5 border-t border-fg/5">
				<button 
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
					onclick={goToLogs}
				>
					{uiText("ui.7b6364fb7c71ffd9")}
				</button>

				{#if diagnosis.recommendedAction === 'increase_ram' || diagnosis.recommendedAction === 'install_java'}
                    <button class={launcherButton({ variant: "primary", size: "sm", class: "disabled:opacity-50" })} disabled={crashDoctor.isFixing} onclick={handleFix}>
                        {crashDoctor.isFixing ? 'Aplicando…' : diagnosis.recommendedAction === 'increase_ram' ? uiText("ui.903dc5af4673a6a8") : uiText("ui.154995d8266e1321")}
                    </button>
                {:else if diagnosis.recommendedAction === 'repair_modpack'}
					<button 
						type="button"
						class={launcherButton({ variant: "secondary", size: "sm", class: "from-emerald-500 to-emerald-600 hover:from-emerald-400 hover:to-emerald-500 flex items-center gap-2 disabled:opacity-50" })}
						disabled={crashDoctor.isFixing}
						onclick={handleFix}
					>
						<Wrench class="w-4 h-4 fill-current" />
						{crashDoctor.isFixing ? 'Reparando...' : uiText("ui.833a431b68e5e822")}
					</button>
				{:else if diagnosis.recommendedAction === 'disable_optifine' || diagnosis.recommendedAction === 'disable_mod'}
					<button 
						type="button"
						class={launcherButton({ variant: "secondary", size: "sm", class: "from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500 flex items-center gap-2 disabled:opacity-50" })}
						disabled={crashDoctor.isFixing}
						onclick={handleFix}
					>
						<Zap class="w-4 h-4 fill-current" />
						{crashDoctor.isFixing ? 'Aplicando...' : uiText("ui.23b4182f3e5afbc2")}
					</button>
				{/if}

				<button 
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
					onclick={() => crashDoctor.close()}
				>
					{uiText("ui.3f3f7d88e05abb17")}
				</button>
			</div>
		</div>
	</div>
{/if}
