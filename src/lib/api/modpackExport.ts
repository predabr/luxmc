import { api } from "./client";

export interface ExportResult {
	success: boolean;
	filePath: string;
	fileSize: number;
	totalMods: number;
}

export async function instanceExportModpack(
	profileId: string,
	exportFormat: "mrpack" | "zip",
	customName?: string
): Promise<ExportResult> {
	return api.invoke<ExportResult>("instance_export_modpack", {
		profileId,
		exportFormat,
		customName: customName || null
	});
}
