import { api } from "./client";

export async function appPerformUpdate(downloadUrl: string): Promise<void> {
	return api.invoke("app_perform_update", { downloadUrl });
}
