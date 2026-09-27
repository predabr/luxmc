import { javaInstall } from "$lib/api/java";
import { getSystemSpecs } from "$lib/api/system";
import { crashDoctorDiagnose, type CrashDiagnosis } from "$lib/api/doctor";
import { profilesGet, profilesUpdate, instanceModToggle, instanceFileTree } from "$lib/api/instances";
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
			if (diagnosis.recommendedAction === "increase_ram") {
                const [profile, system] = await Promise.all([profilesGet(currentProfileId), getSystemSpecs()]);
                const current = profile.ramMb || 2048;
                const maximum = Math.floor((system.totalRamMb - 2048) / 1024) * 1024;
                const next = Math.min(current + 2048, maximum);
                if (next <= current) throw new Error("Não há RAM livre suficiente para aumentar a alocação com segurança. Reduza mods ou texturas.");
                await profilesUpdate({ id: currentProfileId, ramMb: next });
                toast(`Memória da instância ajustada para ${next} MB.`, "success");
                isOpen = false;
                return true;
            } else if (diagnosis.recommendedAction === "install_java") {
                const match = diagnosis.logSnippet?.match(/class file version (\d+)/i);
                const major = match ? Number(match[1]) - 44 : null;
                if (!major || major < 8 || major > 25) throw new Error("Versão necessária não identificada. Consulte o log completo.");
                const runtime = await javaInstall(major);
                if (!runtime.installed || !runtime.path) throw new Error("Java não foi instalado.");
                await profilesUpdate({ id: currentProfileId, javaPath: runtime.path });
                toast(`Java ${major} configurado para esta instância.`, "success");
                isOpen = false;
                return true;
            } else if (diagnosis.recommendedAction === "repair_modpack") {
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
			} else if (diagnosis.recommendedAction === "fix_jvm_args") {
				const profile = await profilesGet(currentProfileId);
				const target = diagnosis.offendingMod || "";
				if (profile.jvmArgs && target) {
					const cleaned = profile.jvmArgs
						.split(/\s+/)
						.filter(arg => !arg.includes(target))
						.join(" ");
					await profilesUpdate({ id: currentProfileId, jvmArgs: cleaned });
				} else {
					await profilesUpdate({ id: currentProfileId, jvmArgs: "" });
				}
				toast("Argumentos JVM incompatíveis corrigidos com sucesso!", "success");
				isOpen = false;
				playSound("chime");
				return true;
			}
		} catch (e) {
			toast("Falha ao aplicar correção automática: " + String(e), "error");
		} finally {
			isFixing = false;
		}
		return false;
	}

	async function autoHealCrash(profileId: string, logContent?: string | null): Promise<boolean> {
		try {
			const d = await crashDoctorDiagnose(profileId, logContent);
			if (!d.hasError) return false;
			diagnosis = d;
			currentProfileId = profileId;
			if (d.recommendedAction && d.recommendedAction !== "increase_ram") {
				const fixed = await applyFix();
				if (fixed) {
					toast(`Luxmc Auto-Reparo: ${d.title} corrigido automaticamente com sucesso!`, "success");
					playSound("chime");
					return true;
				}
			}
			open(d, profileId);
			return false;
		} catch {
			return false;
		}
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
		applyFix,
		autoHealCrash
	};
}

export const crashDoctor = createCrashDoctorStore();
