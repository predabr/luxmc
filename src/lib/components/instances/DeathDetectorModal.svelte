<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { 
		Skull, 
		MapPin, 
		Copy, 
		Check, 
		RotateCcw, 
		Compass, 
		History,
		AlertTriangle
	} from "lucide-svelte";
	import { api } from "$lib/api/client";
	import { toast } from "$lib/stores/toasts.svelte";
	import Modal from "$lib/components/ui/Modal.svelte";
	import Button from "$lib/components/ui/Button.svelte";

	type DeathInfo = {
		hasDeath: boolean;
		deathMessage: string;
		timestamp: string;
		x?: number | null;
		y?: number | null;
		z?: number | null;
		dimension: string;
		worldName?: string | null;
	};

	type Props = {
		open: boolean;
		profileId: string;
		onClose: () => void;
		onOpenSnapshots?: () => void;
	};

	let { open, profileId, onClose, onOpenSnapshots }: Props = $props();

	let deathInfo = $state<DeathInfo | null>(null);
	let loading = $state(false);
	let copied = $state(false);

	async function checkDeath() {
		if (!profileId) return;
		loading = true;
		try {
			deathInfo = await api.invoke<DeathInfo>("death_tracker_get_last_death", { profileId });
		} catch (e) {
			deathInfo = null;
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (open && profileId) {
			checkDeath();
		}
	});

	function copyCoordinates() {
		if (!deathInfo || deathInfo.x == null || deathInfo.y == null || deathInfo.z == null) return;
		const text = `/execute in minecraft:${deathInfo.dimension.toLowerCase()} run tp @s ${Math.round(deathInfo.x)} ${Math.round(deathInfo.y)} ${Math.round(deathInfo.z)}`;
		navigator.clipboard.writeText(text);
		copied = true;
		toast("Comando de teletransporte copiado para o clipboard!", "success");
		setTimeout(() => copied = false, 2000);
	}
</script>

<Modal isOpen={open} {onClose} title={uiText("ui.f198285c022b1236")}>
	<div class="flex flex-col gap-4 text-xs">
		{#if loading}
			<div class="py-10 flex flex-col items-center justify-center gap-2 text-fg/50">
				<Compass class="w-6 h-6 animate-spin text-rose-400" />
				<span>{uiText("ui.6a2b0848f48720be")}</span>
			</div>
		{:else if !deathInfo?.hasDeath}
			<div class="py-8 flex flex-col items-center justify-center gap-2 text-center bg-bg-subtle rounded-2xl border border-fg/5 p-6">
				<div class="w-10 h-10 rounded-full bg-emerald-500/15 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
					<Check class="w-5 h-5" />
				</div>
				<h4 class="text-fg font-bold text-sm">{uiText("ui.ea68cdba1858dea4")}</h4>
				<p class="text-fg/50 text-[11px] max-w-xs">
					{uiText("ui.223bf224f91b0bc3")}
				</p>
			</div>
		{:else}
			<div class="bg-rose-500/10 border border-rose-500/25 rounded-2xl p-4 flex flex-col gap-3">
				<div class="flex items-center gap-2 text-rose-400 font-bold text-sm">
					<Skull class="w-5 h-5" />
					<span>{uiText("ui.e35b1c84397cbe70")}</span>
				</div>

				<div class="bg-bg-overlay/40 rounded-xl p-3 border border-fg/5 font-mono text-fg/90">
					"{deathInfo.deathMessage}"
				</div>

				<div class="grid grid-cols-2 gap-2 text-fg/80">
					<div class="bg-bg-subtle rounded-xl p-2.5 border border-fg/5 flex flex-col gap-0.5">
						<span class="text-[10px] text-fg/40 font-semibold uppercase">{uiText("ui.b1fd9aa859e07d5f")}</span>
						<span class="font-mono text-xs font-bold text-amber-300">
							{uiText("ui.939fc7d2410705f9")} {Math.round(deathInfo.x ?? 0)} {uiText("ui.1d1cef6048633552")} {Math.round(deathInfo.y ?? 64)} {uiText("ui.f3dd3ee8e1967bf7")} {Math.round(deathInfo.z ?? 0)}
						</span>
					</div>

					<div class="bg-bg-subtle rounded-xl p-2.5 border border-fg/5 flex flex-col gap-0.5">
						<span class="text-[10px] text-fg/40 font-semibold uppercase">{uiText("ui.b73f5d7ab8e56f22")}</span>
						<span class="font-mono text-xs font-bold text-purple-300 capitalize">
							{deathInfo.dimension}
						</span>
					</div>
				</div>

				<div class="flex items-center gap-2 pt-1">
					<button
						type="button"
						class={launcherButton({ variant: "ghostBrand", size: "sm", class: "flex-1 flex items-center justify-center gap-1.5" })}
						onclick={copyCoordinates}
					>
						{#if copied}
							<Check class="w-3.5 h-3.5" />
							<span>{uiText("ui.a8fe0fc805d5fd50")}</span>
						{:else}
							<Copy class="w-3.5 h-3.5" />
							<span>{uiText("ui.d5edeba0ce8627db")}</span>
						{/if}
					</button>

					{#if onOpenSnapshots}
						<button
							type="button"
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center justify-center gap-1.5" })}
							onclick={() => {
								onClose();
								onOpenSnapshots();
							}}
						>
							<History class="w-3.5 h-3.5" />
							<span>{uiText("ui.f1ec5d78ff100a1c")}</span>
						</button>
					{/if}
				</div>
			</div>
		{/if}

		<div class="flex items-center justify-end pt-2 border-t border-fg/5">
			<Button variant="secondary" size="sm" onclick={onClose}>
				{uiText("statusBanner.dismiss")}
			</Button>
		</div>
	</div>
</Modal>
