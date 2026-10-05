<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
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
				<span>{uiText("ui.655fb2b73364e8cf")}</span>
			</div>
			<button type="button" class={launcherButton({ variant: "ghost", size: "icon", class: "" })} onclick={onClose}>
				<X class="w-4 h-4" />
			</button>
		</div>
		<div class="text-xs text-fg/70 space-y-3 leading-relaxed">
			<p>{uiText("ui.64a83f7b3227dcdf")} <strong>{uiText("ui.e80f086fb0ca1156")}</strong> {uiText("ui.499561d0651881b0")}</p>
			<p>{uiText("ui.8208e3b6eb7b3c5a")} <strong>{uiText("mods.install")}</strong> {uiText("ui.dfbf3a14b058d399")} <strong>{uiText("settings.catVersions")}</strong>.</p>
			<p>{uiText("ui.fb48432b1f8a7b07")}</p>
			<p>{uiText("ui.d9dc783e81c1d42e")}</p>
		</div>
		<button
			type="button"
			class={launcherButton({ variant: "primary", size: "sm", class: "w-full" })}
			onclick={onClose}
		>
			{uiText("ui.3f3f7d88e05abb17")}
		</button>
	</div>
</div>
