<script lang="ts">
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { RotateCcw, Home } from "lucide-svelte";
</script>

<div class="flex h-full w-full flex-col items-center justify-center gap-5 p-8 text-center bg-bg/95 select-none">
	<div class="rounded-3xl bg-brand-500/10 border border-brand-500/20 p-5 shadow-elevated">
		<div class="h-10 w-10 text-brand-400 mx-auto flex items-center justify-center font-black text-xl">
			!
		</div>
	</div>
	<div class="space-y-1.5 max-w-md">
		<h2 class="text-lg font-black text-fg">Ops! Algo não carregou como esperado</h2>
		<p class="text-xs text-fg-muted font-medium">
			{page.error?.message || "Ocorreu uma falha temporária ao alternar de aba."}
		</p>
	</div>
	<div class="flex items-center gap-3 mt-2">
		<button
			type="button"
			onclick={() => window.location.reload()}
			class="flex items-center gap-2 px-5 py-2.5 rounded-2xl bg-brand-500 hover:bg-brand-600 text-brand-foreground text-xs font-black transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer shadow-lg shadow-brand-500/20 active:scale-[0.98]"
		>
			<RotateCcw class="w-3.5 h-3.5" />
			<span>Recarregar aplicativo</span>
		</button>
		<button
			type="button"
			onclick={() => goto("/")}
			class="flex items-center gap-2 px-5 py-2.5 rounded-2xl bg-fg/5 hover:bg-fg/10 text-fg text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer border border-fg/10 active:scale-[0.98]"
		>
			<Home class="w-3.5 h-3.5" />
			<span>Ir para o Início</span>
		</button>
	</div>
</div>
