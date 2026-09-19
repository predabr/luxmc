<script lang="ts">
	import { X, HelpCircle } from "lucide-svelte";

	let { onClose }: { onClose: () => void } = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-[999999] bg-bg-overlay/80 backdrop-blur-sm flex items-center justify-center p-6 cursor-pointer"
	onclick={onClose}
>
	<div class="bg-bg-elevated border border-fg/10 rounded-3xl p-6 max-w-md w-full shadow-2xl space-y-4" onclick={(e) => e.stopPropagation()}>
		<div class="flex items-center justify-between border-b border-fg/5 pb-3">
			<div class="flex items-center gap-2 text-fg font-extrabold text-sm">
				<HelpCircle class="w-4 h-4 text-brand-400" />
				<span>Como Instalar Mods no Luxmc</span>
			</div>
			<button type="button" class="text-fg/40 hover:text-fg cursor-pointer" onclick={onClose}>
				<X class="w-4 h-4" />
			</button>
		</div>
		<div class="text-xs text-fg/70 space-y-3 leading-relaxed">
			<p>1. Certifique-se de que sua instância possui um <strong>Mod Loader</strong> instalado (Fabric, NeoForge ou Forge).</p>
			<p>2. Clique no botão <strong>Instalar</strong> no topo da página ou escolha uma versão específica na aba <strong>Versões</strong>.</p>
			<p>3. O Luxmc fará o download e adicionará o arquivo diretamente à pasta de mods da sua instância.</p>
			<p>4. Inicie o Minecraft pela tela de instâncias e aproveite!</p>
		</div>
		<button
			type="button"
			class="w-full py-2.5 rounded-xl bg-brand-400 text-brand-foreground font-extrabold text-xs transition-all cursor-pointer hover:bg-brand-400"
			onclick={onClose}
		>
			Entendido
		</button>
	</div>
</div>
