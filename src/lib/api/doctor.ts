import { api } from "./client";

export interface CrashDiagnosis {
	hasError: boolean;
	title: string;
	message: string;
	solution: string;
	category: string;
	offendingMod?: string | null;
	recommendedAction?: string | null;
	logSnippet?: string | null;
}

export async function crashDoctorDiagnose(
	profileId: string,
	logContent?: string | null
): Promise<CrashDiagnosis> {
	return api.invoke<CrashDiagnosis>("crash_doctor_diagnose", {
		profileId,
		logContent: logContent ?? null
	});
}

export interface ModConflict {
	title: string;
	description: string;
	modA: string;
	modB: string;
	recommendedAction: string;
	fileToDisable: string;
}

export interface PreLaunchCheckResult {
	hasConflicts: boolean;
	conflicts: ModConflict[];
	duplicates: string[];
}

export interface InstanceReadiness {
	ready: boolean;
	requiredJava: number;
	allocatedRamMb: number;
	recommendedRamMb: number;
	gpuVendor: string;
	gpuDriver: string;
	blockers: string[];
	warnings: string[];
	conflicts: PreLaunchCheckResult;
}

export interface RepairAllOutcome {
	repairedMods: number;
	warnings: string[];
}

export async function doctorInstanceReadiness(profileId: string): Promise<InstanceReadiness> {
	return api.invoke<InstanceReadiness>("doctor_instance_readiness", { profileId });
}

export async function doctorRepairAll(profileId: string): Promise<RepairAllOutcome> {
	return api.invoke<RepairAllOutcome>("doctor_repair_all", { profileId });
}

export async function doctorCheckInstanceConflicts(
	profileId: string
): Promise<PreLaunchCheckResult> {
	return api.invoke<PreLaunchCheckResult>("doctor_check_instance_conflicts", {
		profileId
	});
}
