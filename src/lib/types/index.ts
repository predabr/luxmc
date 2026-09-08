export interface VersionSummary {
	id: string;
	type: "release" | "snapshot" | "old_beta" | "old_alpha";
	url: string;
	time: string;
	releaseTime: string;
}

export interface VersionManifest {
	latest: { release: string; snapshot: string };
	versions: VersionSummary[];
}

export interface ModSearchResult {
	source: "modrinth" | "curseforge";
	projectId: string;
	slug: string;
	title: string;
	description: string;
	iconUrl?: string;
	downloads: number;
	author: string;
}

export interface ModDetail {
	projectId: string;
	title: string;
	description: string;
	body: string;
	iconUrl?: string;
	versions: ModVersion[];
}

export interface ModVersion {
	id: string;
	projectId: string;
	name: string;
	versionNumber: string;
	mcVersions: string[];
	loaders: string[];
	downloadUrl: string;
	fileName: string;
	sha1: string;
	size: number;
	dependencies: Array<{
		projectId: string;
		type: "required" | "optional" | "incompatible" | "embedded";
	}>;
}

export interface DependencyNode {
	id: string;
	title: string;
	reason: "direct" | "transitive";
}

export interface ResolutionPlan {
	toInstall: DependencyNode[];
	toRemove: DependencyNode[];
	conflicts: Array<{ a: string; b: string; reason: string }>;
}
