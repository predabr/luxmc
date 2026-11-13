export type SoftwareKind = 'vanilla' | 'fabric' | 'forge' | 'neoforge' | 'quilt' | 'paper' | 'purpur' | 'spigot';

export interface SoftwareOption {
	kind: SoftwareKind;
	name: string;
	family: 'vanilla' | 'mods' | 'plugins';
	tagline: string;
	description: string;
	supported: boolean;
	unavailableReason?: string;
	requiresVersion?: boolean;
}

export const SERVER_SOFTWARE: SoftwareOption[] = [
	{
		kind: 'vanilla',
		name: 'Vanilla',
		family: 'vanilla',
		tagline: 'Servidor oficial da Mojang',
		description: 'A experiência original do Minecraft, sem modificações. Ideal para mundos puros com amigos.',
		supported: true,
		requiresVersion: false
	},
	{
		kind: 'fabric',
		name: 'Fabric',
		family: 'mods',
		tagline: 'Mod loader leve e moderno',
		description: 'Loader de mods rápido e modular, com atualizações frequentes para as versões mais recentes.',
		supported: true,
		requiresVersion: true
	},
	{
		kind: 'forge',
		name: 'Forge',
		family: 'mods',
		tagline: 'O mod loader clássico',
		description: 'A plataforma de mods mais tradicional, com o maior catálogo histórico de mods para servidores.',
		supported: true,
		requiresVersion: true
	},
	{
		kind: 'neoforge',
		name: 'NeoForge',
		family: 'mods',
		tagline: 'A evolução do Forge',
		description: 'Sucessor moderno do Forge, mantido pela comunidade, para as versões mais recentes do Minecraft.',
		supported: true,
		requiresVersion: true
	},
	{
		kind: 'quilt',
		name: 'Quilt',
		family: 'mods',
		tagline: 'Mod loader comunitário',
		description: 'Fork do Fabric focado em ferramentas abertas e governança comunitária. Compatível com muitos mods do Fabric.',
		supported: true,
		requiresVersion: true
	},
	{
		kind: 'paper',
		name: 'Paper',
		family: 'plugins',
		tagline: 'Servidor de plugins otimizado',
		description: 'Servidor otimizado compatível com plugins Bukkit/Spigot.',
		supported: false,
		unavailableReason: 'Servidores de plugins ainda não são instalados por esta versão do LUXMC. Use Vanilla ou um mod loader.'
	},
	{
		kind: 'purpur',
		name: 'Purpur',
		family: 'plugins',
		tagline: 'Paper com opções extras',
		description: 'Fork do Paper com centenas de opções de personalização.',
		supported: false,
		unavailableReason: 'Servidores de plugins ainda não são instalados por esta versão do LUXMC. Use Vanilla ou um mod loader.'
	},
	{
		kind: 'spigot',
		name: 'Spigot',
		family: 'plugins',
		tagline: 'Servidor de plugins clássico',
		description: 'A base clássica de servidores de plugins.',
		supported: false,
		unavailableReason: 'Servidores de plugins ainda não são instalados por esta versão do LUXMC. Use Vanilla ou um mod loader.'
	}
];

export function softwareByKind(kind: string): SoftwareOption {
	return SERVER_SOFTWARE.find((option) => option.kind === kind) ?? SERVER_SOFTWARE[0];
}

export function supportsMods(kind: string): boolean {
	return ['fabric', 'forge', 'neoforge', 'quilt'].includes(kind);
}

export function supportsPlugins(kind: string): boolean {
	return ['paper', 'purpur', 'spigot'].includes(kind);
}

export const LOADER_KINDS = ['fabric', 'forge', 'neoforge', 'quilt'] as const;
