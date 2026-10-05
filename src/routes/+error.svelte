<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
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
		<h2 class="text-lg font-black text-fg">{uiText("ui.7a26b45cc7cefcac")}</h2>
		<p class="text-xs text-fg-muted font-medium">
			{page.error?.message || uiText("ui.0c58118f5ac5a221")}
		</p>
	</div>
	<div class="flex items-center gap-3 mt-2">
		<button
			type="button"
			onclick={() => window.location.reload()}
			class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-2" })}
		>
			<RotateCcw class="w-3.5 h-3.5" />
			<span>{uiText("ui.916192859de49e3b")}</span>
		</button>
		<button
			type="button"
			onclick={() => goto("/")}
			class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
		>
			<Home class="w-3.5 h-3.5" />
			<span>{uiText("ui.cafc9182d178fd2f")}</span>
		</button>
	</div>
</div>
