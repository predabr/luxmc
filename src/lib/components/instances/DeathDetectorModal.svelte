<script lang="ts">
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

<Modal isOpen={open} {onClose} title="Detector de Morte & Resgate de Inventário">
	<div class="flex flex-col gap-4 text-xs">
		{#if loading}
			<div class="py-10 flex flex-col items-center justify-center gap-2 text-white/50">
				<Compass class="w-6 h-6 animate-spin text-rose-400" />
				<span>Lendo registros de sobrevivência e coordenadas...</span>
			</div>
		{:else if !deathInfo?.hasDeath}
			<div class="py-8 flex flex-col items-center justify-center gap-2 text-center bg-bg-subtle rounded-2xl border border-white/5 p-6">
				<div class="w-10 h-10 rounded-full bg-emerald-500/15 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
					<Check class="w-5 h-5" />
				</div>
				<h4 class="text-white font-bold text-sm">Você está vivo e seguro!</h4>
				<p class="text-white/50 text-[11px] max-w-xs">
					Nenhuma mensagem recente de morte foi encontrada nos registros desta sessão.
				</p>
			</div>
		{:else}
			<div class="bg-rose-500/10 border border-rose-500/25 rounded-2xl p-4 flex flex-col gap-3">
				<div class="flex items-center gap-2 text-rose-400 font-bold text-sm">
					<Skull class="w-5 h-5" />
					<span>Última Morte Registrada</span>
				</div>

				<div class="bg-black/40 rounded-xl p-3 border border-white/5 font-mono text-white/90">
					"{deathInfo.deathMessage}"
				</div>

				<div class="grid grid-cols-2 gap-2 text-white/80">
					<div class="bg-bg-subtle rounded-xl p-2.5 border border-white/5 flex flex-col gap-0.5">
						<span class="text-[10px] text-white/40 font-semibold uppercase">Coordenadas</span>
						<span class="font-mono text-xs font-bold text-amber-300">
							X: {Math.round(deathInfo.x ?? 0)} · Y: {Math.round(deathInfo.y ?? 64)} · Z: {Math.round(deathInfo.z ?? 0)}
						</span>
					</div>

					<div class="bg-bg-subtle rounded-xl p-2.5 border border-white/5 flex flex-col gap-0.5">
						<span class="text-[10px] text-white/40 font-semibold uppercase">Dimensão</span>
						<span class="font-mono text-xs font-bold text-purple-300 capitalize">
							{deathInfo.dimension}
						</span>
					</div>
				</div>

				<div class="flex items-center gap-2 pt-1">
					<button
						type="button"
						class="flex-1 py-2 px-3 rounded-xl bg-brand-500/20 hover:bg-brand-500/30 text-brand-300 border border-brand-500/30 font-bold flex items-center justify-center gap-1.5 transition cursor-pointer"
						onclick={copyCoordinates}
					>
						{#if copied}
							<Check class="w-3.5 h-3.5" />
							<span>Copiado!</span>
						{:else}
							<Copy class="w-3.5 h-3.5" />
							<span>Copiar Comando /tp</span>
						{/if}
					</button>

					{#if onOpenSnapshots}
						<button
							type="button"
							class="py-2 px-3 rounded-xl bg-purple-500/20 hover:bg-purple-500/30 text-purple-300 border border-purple-500/30 font-bold flex items-center justify-center gap-1.5 transition cursor-pointer"
							onclick={() => {
								onClose();
								onOpenSnapshots();
							}}
						>
							<History class="w-3.5 h-3.5" />
							<span>Restaurar Snapshot</span>
						</button>
					{/if}
				</div>
			</div>
		{/if}

		<div class="flex items-center justify-end pt-2 border-t border-white/5">
			<Button variant="secondary" size="sm" onclick={onClose}>
				Fechar
			</Button>
		</div>
	</div>
</Modal>
