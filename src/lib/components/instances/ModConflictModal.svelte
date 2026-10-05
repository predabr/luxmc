<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { fade, scale } from "svelte/transition";
	import {
		AlertTriangle,
		ShieldAlert,
		X,
		Check,
		Wrench,
		Sparkles
	} from "lucide-svelte";
	import type { PreLaunchCheckResult, ModConflict } from "$lib/api";
	import { instanceModToggle } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let {
		isOpen = $bindable(false),
		profileId,
		conflictsResult,
		onResolved,
		onProceedAnyway,
		onClose
	}: {
		isOpen: boolean;
		profileId: string;
		conflictsResult: PreLaunchCheckResult | null;
		onResolved: () => void;
		onProceedAnyway: () => void;
		onClose: () => void;
	} = $props();

	let resolving = $state(false);

	async function handleFixConflict(c: ModConflict) {
		resolving = true;
		try {
			await instanceModToggle(profileId, c.fileToDisable, false);
			toast(uiText("ui.0a7a8e884d0d0a53", {arg0: (c.fileToDisable)}), "success");
			playSound("click");
			onResolved();
		} catch (e) {
			toast(uiText("ui.1f8172ccbfdea83c") + String(e), "error");
		} finally {
			resolving = false;
		}
	}

	async function handleFixAll() {
		if (!conflictsResult) return;
		resolving = true;
		try {
			for (const c of conflictsResult.conflicts) {
				await instanceModToggle(profileId, c.fileToDisable, false);
			}
			toast(uiText("ui.1fe2e66643f2b34e"), "success");
			playSound("click");
			onResolved();
		} catch (e) {
			toast(uiText("ui.507ea47b65b10764") + String(e), "error");
		} finally {
			resolving = false;
		}
	}
</script>

{#if isOpen && conflictsResult && conflictsResult.hasConflicts}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/85 backdrop-blur-md select-none" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="w-full max-w-lg rounded-3xl bg-bg-elevated border border-rose-500/30 p-6 shadow-2xl space-y-5" in:scale={{ easing: backOut, start: 0.95, duration: 260 }}>
			<!-- Header -->
			<div class="flex items-center justify-between border-b border-fg/10 pb-4">
				<div class="flex items-center gap-3">
					<div class="w-11 h-11 rounded-2xl bg-rose-500/20 border border-rose-500/40 flex items-center justify-center text-rose-400">
						<ShieldAlert class="w-6 h-6" />
					</div>
					<div>
						<h3 class="text-base font-black text-fg">{uiText("ui.c5a37f47c43c4086")}</h3>
						<p class="text-xs text-rose-300/80">{uiText("ui.b6614b670d15e66d")}</p>
					</div>
				</div>

				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					onclick={onClose}
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			<!-- Description -->
			<p class="text-xs text-fg/60 leading-relaxed">
				{uiText("ui.a45b17833c52cb94")}
			</p>

			<!-- Conflicts List -->
			<div class="space-y-3 max-h-64 overflow-y-auto pr-1 custom-scrollbar">
				{#each conflictsResult.conflicts as c}
					<div class="p-4 rounded-2xl bg-bg-elevated border border-rose-500/20 space-y-2.5">
						<div class="flex items-center justify-between">
							<span class="text-xs font-black text-rose-300">{c.title}</span>
							<span class="text-[10px] font-mono text-fg/40 bg-fg/5 px-2 py-0.5 rounded">{uiText("ui.bc8c15d67161c8c5")}</span>
						</div>
						<p class="text-[11px] text-fg/50 leading-relaxed">{c.description}</p>
						<div class="flex items-center justify-between pt-1">
							<span class="text-[10px] font-bold text-emerald-400">{c.recommendedAction}</span>
							<button
								type="button"
								class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
								onclick={() => handleFixConflict(c)}
								disabled={resolving}
							>
								<Wrench class="w-3.5 h-3.5" />
								<span>{uiText("ui.bcebd53d539ad6fa")}</span>
							</button>
						</div>
					</div>
				{/each}

				{#each conflictsResult.duplicates as dup}
					<div class="p-3 rounded-xl bg-amber-500/10 border border-amber-500/20 text-xs text-amber-300">
						{dup}
					</div>
				{/each}
			</div>

			<!-- Actions -->
			<div class="flex items-center justify-between pt-2 border-t border-fg/10">
				<button
					type="button"
					class={launcherButton({ variant: "ghost", size: "sm", class: "hover:underline" })}
					onclick={onProceedAnyway}
				>
					{uiText("ui.2a84a857a609900b")}
				</button>

				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "from-emerald-500 to-teal-500 hover:from-emerald-400 hover:to-teal-400 uppercase tracking-wider" })}
					onclick={handleFixAll}
					disabled={resolving}
				>
					{uiText("ui.64cd32a49ef9ef71")}
				</button>
			</div>
		</div>
	</div>
{/if}
