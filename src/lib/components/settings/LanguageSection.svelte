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
	<div class="text-xs font-bold text-white mb-2">Idioma</div>
	<div class="bg-[#18191c] border border-white/5 rounded-2xl p-3.5 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<div class="w-9 h-9 rounded-xl bg-white/5 flex items-center justify-center text-white/40">
				<Globe class="w-4 h-4" />
			</div>
			<div>
				<div class="text-xs font-bold text-white">Idioma</div>
				<div class="text-[10px] text-white/40">Escolha o seu idioma preferido</div>
			</div>
		</div>
		<select
			bind:value={selectedLanguage}
			onchange={handleChange}
			class="bg-[#1c1d22] border border-white/10 rounded-xl px-4 py-2 text-xs text-white font-bold focus:outline-none focus:border-[#c5a880] cursor-pointer"
		>
			<option value="pt-BR">Português (Brasil)</option>
			<option value="en">English (US)</option>
		</select>
	</div>
</div>
