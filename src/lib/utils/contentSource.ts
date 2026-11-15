export function contentSourceLabel(source: string | null | undefined): string {
	const normalized = (source ?? "").trim().toLowerCase();
	if (normalized === "modrinth") return "Modrinth";
	if (normalized === "curseforge") return "CurseForge";
	if (normalized === "ftb") return "Feed The Beast";
	return source ?? "";
}
