<script lang="ts">
	import { X, HelpCircle } from "lucide-svelte";

	let { onClose }: { onClose: () => void } = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-[999999] bg-black/80 backdrop-blur-sm flex items-center justify-center p-6 cursor-pointer"
	onclick={onClose}
>
	<div class="bg-[#18191c] border border-white/10 rounded-3xl p-6 max-w-md w-full shadow-2xl space-y-4" onclick={(e) => e.stopPropagation()}>
		<div class="flex items-center justify-between border-b border-white/5 pb-3">
			<div class="flex items-center gap-2 text-white font-extrabold text-sm">
				<HelpCircle class="w-4 h-4 text-[#caa97c]" />
				<span>Como Instalar Mods no Luxmc</span>
			</div>
			<button type="button" class="text-white/40 hover:text-white cursor-pointer" onclick={onClose}>
				<X class="w-4 h-4" />
			</button>
		</div>
		<div class="text-xs text-white/70 space-y-3 leading-relaxed">
			<p>1. Certifique-se de que sua instância possui um <strong>Mod Loader</strong> instalado (Fabric, NeoForge ou Forge).</p>
			<p>2. Clique no botão <strong>Instalar</strong> no topo da página ou escolha uma versão específica na aba <strong>Versões</strong>.</p>
			<p>3. O Luxmc fará o download e adicionará o arquivo diretamente à pasta de mods da sua instância.</p>
			<p>4. Inicie o Minecraft pela tela de instâncias e aproveite!</p>
		</div>
		<button
			type="button"
			class="w-full py-2.5 rounded-xl bg-[#caa97c] text-black font-extrabold text-xs transition-all cursor-pointer hover:bg-[#e4c99c]"
			onclick={onClose}
		>
			Entendido
		</button>
	</div>
</div>
