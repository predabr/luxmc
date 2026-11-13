import {
	saveServerSettings,
	serverManage,
	serverSettings,
	serverPublicAccess,
	updateLocalServer,
	type LocalServerConfig
} from '$lib/api/localServers';
import { modsVersions } from '$lib/api/mods';
import type { ModSearchResultItem, ModVersion } from '$lib/api/types';
import { formatGiB, requiredJavaMajor } from '$lib/mcserver/version';
import { platform } from '$lib/mcserver/state.svelte';

export interface PackPlan {
	version: ModVersion;
	mcVersion: string;
	loaderKind: string;
	loaderVersion: string;
	javaMajor: number;
	steps: string[];
}

const LOADER_ORDER = ['neoforge', 'forge', 'fabric', 'quilt'];

export class PackProvisionError extends Error {}

function pickLoader(loaders: string[] | undefined, versionNumber: string) {
	const normalized = (loaders ?? []).map((kind) => kind.toLowerCase());
	const kind = LOADER_ORDER.find((candidate) => normalized.includes(candidate));
	if (!kind) throw new PackProvisionError('Este modpack não declara um loader compatível com servidor (Fabric, Forge, NeoForge ou Quilt).');
	return { kind, loaderVersion: versionNumber };
}

export async function resolvePackPlan(pack: ModSearchResultItem, versionId?: string): Promise<PackPlan> {
	const versions = await modsVersions(pack.sourceId, '', pack.source);
	if (versions.length === 0) throw new PackProvisionError('O autor não publicou nenhuma versão deste modpack.');
	const ordered = versionId ? versions.filter((item) => item.id === versionId) : versions;
	const chosen = ordered.find((item) => (item.gameVersions ?? []).length > 0 && (item.loaders ?? []).length > 0) ?? ordered[0];
	if (!chosen) throw new PackProvisionError('Versão do modpack indisponível.');
	const mcVersion = chosen.gameVersions?.[0];
	if (!mcVersion) throw new PackProvisionError('A versão escolhida não informa o Minecraft necessário.');
	const { kind, loaderVersion } = pickLoader(chosen.loaders, chosen.versionNumber);
	const javaMajor = requiredJavaMajor(mcVersion);
	const steps = [
		`Minecraft ${mcVersion} fixado no servidor`,
		`${kind} ${loaderVersion} instalado`,
		`Java ${javaMajor}+ exigido pelo jogo`,
		'pasta local liberada na rede (LAN)',
		'porta fixa e aberta no roteador quando possível',
		'servidor livre para jogadores de outros PCs'
	];
	return { version: chosen, mcVersion, loaderKind: kind, loaderVersion, javaMajor, steps };
}

function suggestedRam(totalRamMb: number): number {
	if (!totalRamMb) return 6144;
	const half = Math.floor((totalRamMb * 0.55) / 512) * 512;
	return Math.max(4096, Math.min(12288, half || 4096));
}

export interface ProvisionOptions {
	/** Monta o servidor inteiro do zero (sem mundo nem mods anteriores). */
	fresh?: boolean;
	onProgress?: (message: string) => void;
}

/**
 * Ajusta o servidor inteiro para rodar o modpack: versão do jogo, loader, memória,
 * mundo limpo, rede liberada e UPnP. Devolve os passos aplicados.
 */
export async function provisionPack(
	server: LocalServerConfig,
	pack: ModSearchResultItem,
	plan: PackPlan,
	options: ProvisionOptions = {}
): Promise<string[]> {
	const report: string[] = [];
	const step = (message: string) => options.onProgress?.(message);
	const settings = await serverSettings(server.id);

	step('Ajustando versão e loader do servidor…');
	const ram = options.fresh ? suggestedRam(platform.totalRamMb) : server.ramMb;
	const updated = await updateLocalServer(server.id, server.name, ram, Math.max(server.maxPlayers, 10), plan.mcVersion);
	report.push(`Versão fixada em Minecraft ${plan.mcVersion}`);
	if (plan.loaderKind !== 'vanilla') {
		await serverManage({ action: 'set_loader', id: server.id, loader: { kind: plan.loaderKind, version: plan.loaderVersion } });
		report.push(`${plan.loaderKind} ${plan.loaderVersion} instalado`);
	}

	step('Aplicando ajustes de rede e jogo…');
	const tuned = await saveServerSettings(server.id, {
		...settings,
		port: settings.port || 25565,
		listenOnNetwork: true,
		onlineMode: settings.onlineMode !== false,
		pvp: true,
		allowFlight: false,
		whitelist: settings.whitelist ?? false,
		viewDistance: Math.min(10, Math.max(6, settings.viewDistance)),
		simulationDistance: Math.min(6, Math.max(4, settings.simulationDistance))
	});
	report.push('Rede local liberada na porta do servidor');

	step('Baixando o pack oficial e verificando dependências…');
	const install = await serverManage<{ applied: number; duplicates?: string[] }>({
		action: 'install_catalog_pack',
		id: server.id,
		project: pack.sourceId,
		version: plan.version.id,
		source: pack.source
	});
	report.push(`Pack aplicado (${install.applied} arquivos)`);
	if (install.duplicates?.length) report.push(`${install.duplicates.length} mod(s) duplicado(s) isolados em mods/.luxmc-duplicates`);

	step('Abrindo a porta no roteador…');
	try {
		await serverPublicAccess(server.id, 'enable');
		report.push('Porta redirecionada automaticamente (UPnP)');
	} catch {
		report.push('UPnP indisponível: amigos da mesma rede já podem entrar');
	}

	// O backend recusa mexer em settings com o servidor rodando, então relê o estado final.
	await saveServerSettings(server.id, tuned).catch(() => null);
	report.push(`Memória dedicada: ${formatGiB(updated.ramMb)}`);
	return report;
}
