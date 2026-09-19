<script lang="ts">
	import { Globe } from "lucide-svelte";
	import { setLocale } from "$lib/stores/persistence.svelte";

	type Props = {
		selectedLanguage: string;
		onLanguageChange: (lang: string) => void;
	};

	let { selectedLanguage, onLanguageChange }: Props = $props();

	function handleChange() {
		setLocale(selectedLanguage === 'pt-BR' ? 'pt-BR' : 'en');
		onLanguageChange(selectedLanguage);
	}
</script>

<div>
	<div class="text-xs font-bold text-fg mb-2">Idioma</div>
	<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-3.5 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<div class="w-9 h-9 rounded-xl bg-fg/5 flex items-center justify-center text-fg/40">
				<Globe class="w-4 h-4" />
			</div>
			<div>
				<div class="text-xs font-bold text-fg">Idioma</div>
				<div class="text-[10px] text-fg/40">Escolha o seu idioma preferido</div>
			</div>
		</div>
		<select
			bind:value={selectedLanguage}
			onchange={handleChange}
			class="bg-bg-subtle border border-fg/10 rounded-xl px-4 py-2 text-xs text-fg font-bold focus:outline-none focus:border-brand-400 cursor-pointer"
		>
			<option value="pt-BR">Português (Brasil)</option>
			<option value="en">English (US)</option>
		</select>
	</div>
</div>
