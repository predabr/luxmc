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
