import type { CrashDiagnosis } from "$lib/api/doctor";
import { instanceModToggle, instanceFileTree } from "$lib/api/instances";
import { toast } from "$lib/stores/toasts.svelte";
import { playSound } from "$lib/utils/sound";

function createCrashDoctorStore() {
	let isOpen = $state(false);
	let diagnosis = $state<CrashDiagnosis | null>(null);
	let currentProfileId = $state<string | null>(null);
	let isFixing = $state(false);

	function open(d: CrashDiagnosis, profileId?: string | null) {
		diagnosis = d;
		currentProfileId = profileId ?? null;
		isOpen = true;
		playSound("warning");
	}

	function close() {
		isOpen = false;
	}

	async function applyFix(): Promise<boolean> {
		if (!diagnosis || !currentProfileId) return false;
		isFixing = true;
		try {
			if (diagnosis.recommendedAction === "repair_modpack") {
				toast("Reparando modpack e baixando mods faltantes...", "info");
				const { instanceRepairModpack } = await import("$lib/api/instances");
				const repaired = await instanceRepairModpack(currentProfileId);
				if (repaired > 0) {
					toast(`${repaired} mod(s) faltante(s) baixado(s) com sucesso!`, "success");
					isOpen = false;
					playSound("chime");
					return true;
				} else {
					toast("Todos os mods do modpack já estão instalados.", "info");
				}
			} else if (diagnosis.recommendedAction === "disable_optifine" || diagnosis.recommendedAction === "disable_mod") {
				const files = await instanceFileTree(currentProfileId, "mods").catch(() => []);
				const target = diagnosis.offendingMod?.toLowerCase() || "optifine";
				const modToDisable = files.find(f => f.name.toLowerCase().includes(target) && !f.name.endsWith(".disabled"));
				if (modToDisable) {
					await instanceModToggle(currentProfileId, modToDisable.name, false);
					toast(`Mod "${modToDisable.name}" desativado com sucesso!`, "success");
					isOpen = false;
					playSound("chime");
					return true;
				} else {
					toast("Arquivo do mod conflitante não encontrado na pasta de mods.", "warning");
				}
			}
		} catch (e) {
			toast("Falha ao aplicar correção automática: " + String(e), "error");
		} finally {
			isFixing = false;
		}
		return false;
	}

	return {
		get isOpen() {
			return isOpen;
		},
		set isOpen(val: boolean) {
			isOpen = val;
		},
		get diagnosis() {
			return diagnosis;
		},
		get profileId() {
			return currentProfileId;
		},
		get isFixing() {
			return isFixing;
		},
		open,
		close,
		applyFix
	};
}

export const crashDoctor = createCrashDoctorStore();
