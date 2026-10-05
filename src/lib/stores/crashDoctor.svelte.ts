import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
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
                if (next <= current) throw new Error(uiText("ui.126a59a278b613a9"));
                await profilesUpdate({ id: currentProfileId, ramMb: next });
                toast(uiText("ui.3d5656930afe1979", {arg0: (next)}), "success");
                isOpen = false;
                return true;
            } else if (diagnosis.recommendedAction === "install_java") {
                const match = diagnosis.logSnippet?.match(/class file version (\d+)/i);
                const major = match ? Number(match[1]) - 44 : null;
                if (!major || major < 8 || major > 25) throw new Error(uiText("ui.ffadc841ac9e219f"));
                const runtime = await javaInstall(major);
                if (!runtime.installed || !runtime.path) throw new Error(uiText("ui.c97eacf109e4ba54"));
                await profilesUpdate({ id: currentProfileId, javaPath: runtime.path });
                toast(uiText("ui.e0f323901b02af6e", {arg0: (major)}), "success");
                isOpen = false;
                return true;
            } else if (diagnosis.recommendedAction === "repair_modpack") {
				toast(uiText("ui.7d50972e0c11f170"), "info");
				const { instanceRepairModpack } = await import("$lib/api/instances");
				const repaired = await instanceRepairModpack(currentProfileId);
				if (repaired > 0) {
					toast(uiText("ui.f1a81e409926c472", {arg0: (repaired)}), "success");
					isOpen = false;
					playSound("chime");
					return true;
				} else {
					toast(uiText("ui.a8e28760d82cbdc0"), "info");
				}
			} else if (diagnosis.recommendedAction === "disable_optifine" || diagnosis.recommendedAction === "disable_mod") {
				const files = await instanceFileTree(currentProfileId, "mods").catch(() => []);
				const target = diagnosis.offendingMod?.toLowerCase() || "optifine";
				const modToDisable = files.find(f => f.name.toLowerCase().includes(target) && !f.name.endsWith(".disabled"));
				if (modToDisable) {
					await instanceModToggle(currentProfileId, modToDisable.name, false);
					toast(uiText("ui.96696ff6138514f7", {arg0: (modToDisable.name)}), "success");
					isOpen = false;
					playSound("chime");
					return true;
				} else {
					toast(uiText("ui.930aa3d35d10eeee"), "warning");
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
				toast(uiText("ui.15bc37994b17ae25"), "success");
				isOpen = false;
				playSound("chime");
				return true;
			}
		} catch (e) {
			toast(uiText("ui.3c2d793fe97ffdce") + String(e), "error");
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
					toast(uiText("ui.a974c6296b413f19", {arg0: (d.title)}), "success");
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
