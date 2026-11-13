import { compareMcVersion, parseMcVersion } from './version';

export type GameruleType = 'bool' | 'int';

export interface GameruleDef {
	id: string;
	label: string;
	description: string;
	type: GameruleType;
	category: string;
	defaultValue: boolean | number;
	since: string;
	until?: string;
	min?: number;
	max?: number;
}

export const GAMERULE_CATEGORIES = [
	{ id: 'gameplay', label: 'Jogabilidade geral' },
	{ id: 'world', label: 'Comportamento do mundo' },
	{ id: 'player', label: 'Jogadores' },
	{ id: 'mob', label: 'Criaturas' },
	{ id: 'items', label: 'Itens e inventário' },
	{ id: 'time', label: 'Tempo e clima' },
	{ id: 'damage', label: 'Dano e regeneração' },
	{ id: 'advanced', label: 'Regras avançadas' }
] as const;

export const GAMERULES: GameruleDef[] = [
	// Jogabilidade geral
	{ id: 'announceAdvancements', label: 'Anunciar conquistas', description: 'Anuncia no chat quando um jogador completa uma conquista.', type: 'bool', category: 'gameplay', defaultValue: true, since: '1.12' },
	{ id: 'commandBlockOutput', label: 'Saída de blocos de comando', description: 'Mostra no chat o resultado dos comandos executados por blocos de comando.', type: 'bool', category: 'gameplay', defaultValue: true, since: '1.8' },
	{ id: 'disableElytraMovementCheck', label: 'Ignorar verificação de élitros', description: 'Desativa a verificação antitrapaça de movimento com élitros. Útil com mods de movimento.', type: 'bool', category: 'gameplay', defaultValue: false, since: '1.9' },
	{ id: 'disablePlayerMovementCheck', label: 'Ignorar verificação de movimento', description: 'Desativa a verificação antitrapaça de movimento dos jogadores.', type: 'bool', category: 'gameplay', defaultValue: false, since: '1.19.4' },
	{ id: 'doLimitedCrafting', label: 'Fabricação limitada', description: 'Exige desbloquear receitas antes de fabricar itens.', type: 'bool', category: 'gameplay', defaultValue: false, since: '1.12' },
	{ id: 'logAdminCommands', label: 'Registrar comandos de administração', description: 'Grava no registro do servidor os comandos executados por operadores.', type: 'bool', category: 'gameplay', defaultValue: true, since: '1.8' },
	{ id: 'sendCommandFeedback', label: 'Retorno de comandos', description: 'Mostra aos jogadores o resultado dos comandos executados.', type: 'bool', category: 'gameplay', defaultValue: true, since: '1.8' },
	{ id: 'showDeathMessages', label: 'Mensagens de morte', description: 'Mostra no chat quando um jogador morre.', type: 'bool', category: 'gameplay', defaultValue: true, since: '1.8' },
	{ id: 'reducedDebugInfo', label: 'Informações de depuração reduzidas', description: 'Oculta coordenadas e detalhes da tela de depuração (F3).', type: 'bool', category: 'gameplay', defaultValue: false, since: '1.8' },
	{ id: 'spectatorsGenerateChunks', label: 'Espectadores geram terreno', description: 'Permite que jogadores no modo espectador carreguem novos trechos do mundo.', type: 'bool', category: 'gameplay', defaultValue: true, since: '1.9' },
	{ id: 'doImmediateRespawn', label: 'Renascimento imediato', description: 'Os jogadores renascem imediatamente, sem a tela de morte.', type: 'bool', category: 'gameplay', defaultValue: false, since: '1.15' },
	{ id: 'forgiveDeadPlayers', label: 'Perdoar jogadores mortos', description: 'Criaturas neutras irritadas perdoam o jogador após ele morrer.', type: 'bool', category: 'gameplay', defaultValue: true, since: '1.16' },
	{ id: 'universalAnger', label: 'Raiva universal de criaturas neutras', description: 'Criaturas neutras irritadas atacam qualquer jogador próximo, não apenas o agressor.', type: 'bool', category: 'gameplay', defaultValue: false, since: '1.16' },
	{ id: 'spawnRadius', label: 'Raio de renascimento', description: 'Distância máxima, em blocos, em que jogadores renascem ao redor do ponto de origem.', type: 'int', category: 'gameplay', defaultValue: 10, since: '1.8', min: 0, max: 1000 },

	// Comportamento do mundo
	{ id: 'doFireTick', label: 'Propagação de fogo', description: 'O fogo se espalha, queima blocos e apaga naturalmente.', type: 'bool', category: 'world', defaultValue: true, since: '1.8' },
	{ id: 'mobGriefing', label: 'Destruição por criaturas', description: 'Creepers, endermen, ghasts e aldeões zumbis podem alterar ou destruir blocos.', type: 'bool', category: 'world', defaultValue: true, since: '1.8' },
	{ id: 'doTileDrops', label: 'Blocos derrubam itens', description: 'Blocos quebrados derrubam seus itens.', type: 'bool', category: 'world', defaultValue: true, since: '1.8' },
	{ id: 'tntExplodes', label: 'Explosão de TNT', description: 'A TNT explode quando ativada.', type: 'bool', category: 'world', defaultValue: true, since: '1.19' },
	{ id: 'doVinesSpread', label: 'Propagação de trepadeiras', description: 'Trepadeiras crescem e se espalham pelas superfícies.', type: 'bool', category: 'world', defaultValue: true, since: '1.20.2' },
	{ id: 'lavaSourceConversion', label: 'Conversão de fontes de lava', description: 'Permite criar novas fontes de lava de forma renovável.', type: 'bool', category: 'world', defaultValue: false, since: '1.20.2' },
	{ id: 'waterSourceConversion', label: 'Conversão de fontes de água', description: 'Permite criar novas fontes de água ao unir fluxos.', type: 'bool', category: 'world', defaultValue: true, since: '1.20.2' },
	{ id: 'snowAccumulationHeight', label: 'Acúmulo de neve', description: 'Número máximo de camadas de neve que se acumulam ao cair. Use 0 para impedir acúmulo.', type: 'int', category: 'world', defaultValue: 1, since: '1.19.3', min: 0, max: 8 },
	{ id: 'globalSoundEvents', label: 'Eventos sonoros globais', description: 'Alguns sons, como portais do End, são ouvidos por todos os jogadores em qualquer distância.', type: 'bool', category: 'world', defaultValue: true, since: '1.19' },
	{ id: 'projectilesCanBreakBlocks', label: 'Projéteis quebram blocos', description: 'Projéteis, como bolas de fogo de ghast, podem quebrar blocos frágeis.', type: 'bool', category: 'world', defaultValue: true, since: '1.19' },
	{ id: 'blockExplosionDropDecay', label: 'Decaimento de itens de explosão em blocos', description: 'Parte dos itens derrubados por explosões em blocos é destruída.', type: 'bool', category: 'world', defaultValue: true, since: '1.19' },
	{ id: 'randomTickSpeed', label: 'Velocidade de crescimento do mundo', description: 'Frequência dos ciclos aleatórios que controlam crescimento de plantas, gelo e fogo. O padrão é 3.', type: 'int', category: 'world', defaultValue: 3, since: '1.8', min: 0, max: 4096 },

	// Jogadores
	{ id: 'keepInventory', label: 'Manter inventário', description: 'Os jogadores conservam itens e experiência ao morrer.', type: 'bool', category: 'player', defaultValue: false, since: '1.8' },
	{ id: 'naturalRegeneration', label: 'Regeneração natural', description: 'A vida se regenera quando a fome está cheia.', type: 'bool', category: 'player', defaultValue: true, since: '1.8' },
	{ id: 'doInsomnia', label: 'Geração de phantoms', description: 'Phantoms atacam jogadores que ficam muitos dias sem dormir.', type: 'bool', category: 'player', defaultValue: true, since: '1.13' },
	{ id: 'drowningDamage', label: 'Dano de afogamento', description: 'Os jogadores sofrem dano ao ficar sem ar embaixo d\u2019água.', type: 'bool', category: 'player', defaultValue: true, since: '1.15' },
	{ id: 'fallDamage', label: 'Dano de queda', description: 'Os jogadores sofrem dano ao cair de grandes alturas.', type: 'bool', category: 'player', defaultValue: true, since: '1.15' },
	{ id: 'fireDamage', label: 'Dano de fogo', description: 'Os jogadores sofrem dano ao pegar fogo.', type: 'bool', category: 'player', defaultValue: true, since: '1.15' },
	{ id: 'freezeDamage', label: 'Dano de congelamento', description: 'Os jogadores sofrem dano ao ficar presos em neve fofa.', type: 'bool', category: 'player', defaultValue: true, since: '1.17' },
	{ id: 'playersSleepingPercentage', label: 'Percentual de jogadores dormindo', description: 'Percentual de jogadores que precisa dormir para passar a noite. Use 100 para exigir todos, 0 para nunca exigir sono.', type: 'int', category: 'player', defaultValue: 100, since: '1.17', min: 0, max: 100 },

	// Criaturas
	{ id: 'doMobSpawning', label: 'Geração de criaturas', description: 'Criaturas surgem naturalmente no mundo. Não afeta geradores nem invocações.', type: 'bool', category: 'mob', defaultValue: true, since: '1.8' },
	{ id: 'spawnMonsters', label: 'Geração de monstros', description: 'Monstros hostis surgem naturalmente no mundo.', type: 'bool', category: 'mob', defaultValue: true, since: '1.21.9' },
	{ id: 'doMobLoot', label: 'Criaturas derrubam itens', description: 'Criaturas mortas derrubam seus itens.', type: 'bool', category: 'mob', defaultValue: true, since: '1.8' },
	{ id: 'doPatrolSpawning', label: 'Patrulhas de saqueadores', description: 'Patrulhas de saqueadores surgem próximas aos jogadores.', type: 'bool', category: 'mob', defaultValue: true, since: '1.14' },
	{ id: 'doTraderSpawning', label: 'Vendedor ambulante', description: 'O vendedor ambulante visita os jogadores periodicamente.', type: 'bool', category: 'mob', defaultValue: true, since: '1.14' },
	{ id: 'doWardenSpawning', label: 'Geração do Warden', description: 'O Warden pode ser invocado por sensores sculk nas cavernas profundas.', type: 'bool', category: 'mob', defaultValue: true, since: '1.19' },
	{ id: 'mobExplosionDropDecay', label: 'Decaimento de itens de explosão de criaturas', description: 'Parte dos itens derrubados por explosões de criaturas é destruída.', type: 'bool', category: 'mob', defaultValue: true, since: '1.19' },
	{ id: 'maxEntityCramming', label: 'Limite de aglomeração de criaturas', description: 'Número máximo de criaturas que pode ocupar o mesmo espaço antes de sofrerem dano por sufocamento.', type: 'int', category: 'mob', defaultValue: 24, since: '1.11', min: 0, max: 1000 },

	// Itens e inventário
	{ id: 'doEntityDrops', label: 'Entidades derrubam itens', description: 'Entidades que não são criaturas, como molduras e carrinhos, derrubam itens ao serem destruídas.', type: 'bool', category: 'items', defaultValue: true, since: '1.8' },
	{ id: 'commandModificationBlockLimit', label: 'Limite de blocos por comando', description: 'Número máximo de blocos alterados por comandos como /fill e /clone.', type: 'int', category: 'items', defaultValue: 32768, since: '1.19', min: 1, max: 2147483647 },
	{ id: 'maxCommandChainLength', label: 'Comprimento máximo de cadeias de comandos', description: 'Número máximo de blocos de comando em cadeia executados em sequência.', type: 'int', category: 'items', defaultValue: 65536, since: '1.8', min: 0, max: 2147483647 },
	{ id: 'maxCommandForkCount', label: 'Limite de ramificações de comandos', description: 'Número máximo de contextos criados por comandos como /execute as.', type: 'int', category: 'items', defaultValue: 65536, since: '1.19.4', min: 0, max: 2147483647 },

	// Tempo e clima
	{ id: 'doDaylightCycle', label: 'Ciclo de dia e noite', description: 'O tempo avança naturalmente, alternando entre dia e noite.', type: 'bool', category: 'time', defaultValue: true, since: '1.8' },
	{ id: 'doWeatherCycle', label: 'Ciclo de clima', description: 'O clima alterna naturalmente entre limpo, chuva e tempestade.', type: 'bool', category: 'time', defaultValue: true, since: '1.8' },

	// Dano e regeneração
	// (regras de dano ficam em "Jogadores"; esta categoria reúne efeitos sobre o mundo)
	{ id: 'tntExplosionDropDecay', label: 'Decaimento de itens de explosão de TNT', description: 'Parte dos itens derrubados por explosões de TNT é destruída.', type: 'bool', category: 'damage', defaultValue: false, since: '1.19' },

	// Regras avançadas
	{ id: 'minecartMaxSpeed', label: 'Velocidade máxima de carrinhos', description: 'Velocidade máxima experimental de carrinhos, em blocos por segundo.', type: 'int', category: 'advanced', defaultValue: 8, since: '1.21.2', min: 1, max: 1000 },
	{ id: 'playersNetherPortalDefaultDelay', label: 'Espera do portal do Nether (sobrevivência)', description: 'Tempo, em ciclos, que um jogador fora do modo criativo espera dentro do portal antes de ser teleportado.', type: 'int', category: 'advanced', defaultValue: 80, since: '1.20.2', min: 0, max: 2147483647 },
	{ id: 'playersNetherPortalCreativeDelay', label: 'Espera do portal do Nether (criativo)', description: 'Tempo, em ciclos, que um jogador no modo criativo espera dentro do portal antes de ser teleportado.', type: 'int', category: 'advanced', defaultValue: 1, since: '1.20.2', min: 0, max: 2147483647 }
];

function versionGte(version: string, min: string): boolean {
	const parsed = parseMcVersion(version);
	if (!parsed) return true;
	return compareMcVersion(version, min) >= 0;
}

function versionLt(version: string, max: string): boolean {
	const parsed = parseMcVersion(version);
	if (!parsed) return true;
	return compareMcVersion(version, max) < 0;
}

export interface ResolvedGamerule extends GameruleDef {
	serverName: string;
	mapValue?: (value: string) => string;
}

const SNAKE_CASE_RENAMES: Record<string, string> = {
	announceAdvancements: 'show_advancement_messages',
	commandModificationBlockLimit: 'max_block_modifications',
	disableElytraMovementCheck: 'elytra_movement_check',
	disablePlayerMovementCheck: 'player_movement_check',
	doDaylightCycle: 'advance_time',
	doEntityDrops: 'entity_drops',
	doFireTick: 'fire_spread_radius_around_player',
	doImmediateRespawn: 'immediate_respawn',
	doInsomnia: 'spawn_phantoms',
	doLimitedCrafting: 'limited_crafting',
	doMobLoot: 'mob_drops',
	doMobSpawning: 'spawn_mobs',
	doPatrolSpawning: 'spawn_patrols',
	doTileDrops: 'block_drops',
	doTraderSpawning: 'spawn_wandering_traders',
	doVinesSpread: 'spread_vines',
	doWardenSpawning: 'spawn_wardens',
	doWeatherCycle: 'advance_weather',
	maxCommandChainLength: 'max_command_sequence_length',
	maxCommandForkCount: 'max_command_forks',
	minecartMaxSpeed: 'max_minecart_speed',
	naturalRegeneration: 'natural_health_regeneration',
	snowAccumulationHeight: 'max_snow_accumulation_height',
	spawnMonsters: 'spawn_monsters',
	spawnRadius: 'respawn_radius'
};

const INVERTED_ON_RENAME = new Set(['disableElytraMovementCheck', 'disablePlayerMovementCheck']);

function toSnakeCase(id: string): string {
	return id.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
}

export function usesSnakeCaseGamerules(version: string): boolean {
	const snapshot = /^(\d+)w(\d+)/.exec(version);
	if (snapshot) {
		const year = Number(snapshot[1]);
		const week = Number(snapshot[2]);
		return year > 25 || (year === 25 && week >= 44);
	}
	if (!parseMcVersion(version)) return true;
	return compareMcVersion(version, '1.21.11') >= 0;
}

export function gamerulesForVersion(mcVersion: string): ResolvedGamerule[] {
	const snake = usesSnakeCaseGamerules(mcVersion);
	return GAMERULES.filter((rule) => {
		if (!versionGte(mcVersion, rule.since)) return false;
		if (rule.until && !versionLt(mcVersion, rule.until)) return false;
		return true;
	}).map((rule) => {
		if (!snake) return { ...rule, serverName: rule.id };
		const serverName = SNAKE_CASE_RENAMES[rule.id] ?? toSnakeCase(rule.id);
		const mapValue =
			rule.id === 'doFireTick'
				? (value: string) => (value === 'true' ? '128' : '0')
				: INVERTED_ON_RENAME.has(rule.id)
					? (value: string) => (value === 'true' ? 'false' : 'true')
					: undefined;
		return { ...rule, serverName, mapValue };
	});
}

export function gameruleById(id: string): GameruleDef | undefined {
	return GAMERULES.find((rule) => rule.id === id);
}
