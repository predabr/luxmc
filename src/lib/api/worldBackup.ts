import { api } from "./client";

export interface WorldBackupEntry {
	fileName: string;
	filePath: string;
	sizeBytes: number;
	createdAt: string;
	worldName: string;
}

export async function instanceBackupWorld(
	profileId: string,
	worldFolder: string
): Promise<WorldBackupEntry> {
	return api.invoke<WorldBackupEntry>("instance_backup_world", {
		profileId,
		worldFolder
	});
}

export async function instanceListWorldBackups(
	profileId: string
): Promise<WorldBackupEntry[]> {
	return api.invoke<WorldBackupEntry[]>("instance_list_world_backups", {
		profileId
	});
}
