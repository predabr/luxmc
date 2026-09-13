import { api } from "./client";

export async function runTeamworkPreview(): Promise<string> {
	return api.invoke<string>("teamwork_preview");
}
