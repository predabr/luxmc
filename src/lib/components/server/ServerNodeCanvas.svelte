<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import {
		Plus,
		Play,
		Square,
		FolderOpen,
		LoaderCircle,
		SlidersHorizontal,
		X,
		Package,
		Globe2,
		Power,
		Puzzle,
		Users,
		MemoryStick,
		ChevronLeft,
		ChevronRight,
		RotateCcw,
		ZoomIn,
		ZoomOut,
		Maximize,
		GripVertical,
		Download,
		TerminalSquare,
		Archive,
		Settings,
		ShieldCheck,
		Shapes,
		Palette,
		Network,
		Send,
		Trash2,
		Copy,
		Search,
		Check,
		RefreshCw,
		Lock,
		Unlock,
		Wifi
	} from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import {
		saveServerSettings,
		updateLocalServer,
		startLocalServer,
		stopLocalServer,
		openLocalServerFolder,
		serverManage,
		serverPublicAccess,
		sendServerCommand,
		serverConsoleText,
		type ServerSettings
	} from '$lib/api/localServers';
	import { modsSearch, modsVersions } from '$lib/api/mods';
	import { loadersVersions } from '$lib/api/launch';
	import { platform } from '$lib/mcserver/state.svelte';
	import { SERVER_SOFTWARE, softwareByKind, supportsMods } from '$lib/mcserver/software';
	import { provisionPack, resolvePackPlan } from '$lib/mcserver/packProvision';
	import { formatGiB, formatBytes, formatDateTime } from '$lib/mcserver/version';
	import type { ModSearchResultItem } from '$lib/api/types';
	import GlassSelect from './GlassSelect.svelte';

	let { actionBusy, onCreate, onFromPack }: { actionBusy: boolean; onCreate: () => void; onFromPack: () => void } = $props();

	interface NodeStyle {
		shape: 'rounded' | 'pill' | 'circle' | 'custom';
		radius: number;
		stroke: number;
		glass: number;
		theme: string;
	}

	interface NodePos {
		x: number;
		y: number;
	}

	const STORE_KEY = 'luxmc.nodeCanvas.v3';

	const THEMES: Record<string, { css: string; label: string }> = {
		blue: { css: 'rgb(var(--brand-400))', label: 'Azul elétrico' },
		amethyst: { css: 'rgb(var(--amethyst-400))', label: 'Ametista' },
		green: { css: 'rgb(var(--success))', label: 'Esmeralda' },
		yellow: { css: 'rgb(var(--warning))', label: 'Âmbar' },
		red: { css: 'rgb(var(--danger))', label: 'Carmesim' },
		brass: { css: 'rgb(var(--brass-400))', label: 'Latão' },
		sky: { css: 'rgb(var(--info))', label: 'Céu' },
		cyan: { css: 'rgb(var(--teal-400))', label: 'Ciano' },
		copper: { css: 'rgb(var(--copper-400))', label: 'Cobre' }
	};

	const NODE_DEFS: Record<string, { label: string; icon: typeof Package }> = {
		main: { label: 'Servidor', icon: Package },
		software: { label: 'Software', icon: Puzzle },
		modpacks: { label: 'Modpacks', icon: Download },
		world: { label: 'Mundo', icon: Globe2 },
		network: { label: 'Rede & Conexão', icon: Network },
		resources: { label: 'Recursos', icon: MemoryStick },
		eula: { label: 'EULA', icon: ShieldCheck },
		control: { label: 'Controle', icon: Power },
		console: { label: 'Console', icon: TerminalSquare },
		players: { label: 'Jogadores', icon: Users },
		backups: { label: 'Backups', icon: Archive },
		config: { label: 'Configuração', icon: Settings }
	};

	const DEFAULT_POSITIONS: Record<string, NodePos> = {
		main: { x: 620, y: 470 },
		world: { x: 170, y: 110 },
		modpacks: { x: 540, y: 110 },
		software: { x: 910, y: 110 },
		network: { x: 40, y: 380 },
		eula: { x: 1180, y: 330 },
		resources: { x: 1180, y: 700 },
		players: { x: 1120, y: 1120 },
		console: { x: 760, y: 1010 },
		control: { x: 420, y: 1010 },
		backups: { x: 40, y: 1700 },
		config: { x: 40, y: 1100 }
	};

	const DEFAULT_STYLES: Record<string, NodeStyle> = {
		main: { shape: 'rounded', radius: 24, stroke: 1.5, glass: 0.85, theme: 'blue' },
		software: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'blue' },
		modpacks: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'blue' },
		world: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'green' },
		network: { shape: 'rounded', radius: 20, stroke: 1.4, glass: 0.82, theme: 'cyan' },
		resources: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'yellow' },
		eula: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'amethyst' },
		control: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'red' },
		console: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'blue' },
		players: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'blue' },
		backups: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'yellow' },
		config: { shape: 'rounded', radius: 20, stroke: 1, glass: 0.8, theme: 'blue' }
	};

	const NODE_W: Record<string, number> = {
		main: 320,
		software: 300,
		modpacks: 320,
		world: 300,
		network: 300,
		resources: 300,
		eula: 300,
		control: 300,
		console: 320,
		players: 300,
		backups: 300,
		config: 300
	};

	const EDGES: Array<{ from: string; to: string; theme: string }> = [
		{ from: 'main', to: 'software', theme: 'blue' },
		{ from: 'software', to: 'modpacks', theme: 'blue' },
		{ from: 'main', to: 'world', theme: 'green' },
		{ from: 'main', to: 'network', theme: 'cyan' },
		{ from: 'main', to: 'eula', theme: 'amethyst' },
		{ from: 'main', to: 'resources', theme: 'yellow' },
		{ from: 'main', to: 'control', theme: 'red' },
		{ from: 'main', to: 'console', theme: 'blue' },
		{ from: 'main', to: 'players', theme: 'blue' },
		{ from: 'main', to: 'backups', theme: 'yellow' },
		{ from: 'main', to: 'config', theme: 'blue' }
	];

	const FORMULAS = [
		{ text: 'f(x) = ax² + bx + c', x: 8, y: 12, size: 15, rot: -6 },
		{ text: 'ψ(x, t) = A·e^(−iEt/ħ)', x: 42, y: 6, size: 18, rot: 4 },
		{ text: 'e^iπ + 1 = 0', x: 70, y: 22, size: 14, rot: -3 },
		{ text: '∑ 1/n² = π²/6', x: 24, y: 58, size: 16, rot: 5 },
		{ text: '∂²ψ/∂x² = −k²ψ', x: 58, y: 72, size: 17, rot: -4 },
		{ text: '∇·E = ρ/ε₀', x: 84, y: 48, size: 13, rot: 7 },
		{ text: 'f: ℝ → ℝ', x: 5, y: 82, size: 14, rot: -5 },
		{ text: 'λ = h/p', x: 36, y: 34, size: 12, rot: 3 }
	];

	const STARS = Array.from({ length: 110 }, (_, i) => ({
		id: i,
		x: Math.random() * 3200 - 200,
		y: Math.random() * 2600 - 200,
		s: Math.random() < 0.85 ? 1.5 : 2.5,
		o: 0.12 + Math.random() * 0.4
	}));

	const compact = new Intl.NumberFormat('pt-BR', { notation: 'compact', maximumFractionDigits: 1 });

	let zoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	let positions = $state<Record<string, NodePos>>({ ...DEFAULT_POSITIONS });
	let styles = $state<Record<string, NodeStyle>>({ ...DEFAULT_STYLES });
	let expanded = $state<Record<string, boolean>>({ control: true });
	let heights = $state<Record<string, number>>({});
	let widths = $state<Record<string, number>>({});
	let inspectorOpen = $state(false);
	let inspectorNodeId = $state<string>('main');
	let dragNode = $state<string | null>(null);
	let guides = $state<{ x: number[]; y: number[] }>({ x: [], y: [] });
	let momentumRaf: number | null = null;
	let viewportEl: HTMLElement | null = null;

	let worldDraft = $state({ seed: '', difficulty: 'normal' as ServerSettings['difficulty'], gamemode: 'survival' as ServerSettings['gamemode'], viewDistance: 10, simulationDistance: 10 });

	const DIFFICULTY_OPTIONS: Array<{ value: ServerSettings['difficulty']; label: string }> = [
		{ value: 'peaceful', label: 'Pacífica' },
		{ value: 'easy', label: 'Fácil' },
		{ value: 'normal', label: 'Normal' },
		{ value: 'hard', label: 'Difícil' }
	];
	const GAMEMODE_OPTIONS: Array<{ value: ServerSettings['gamemode']; label: string }> = [
		{ value: 'survival', label: 'Sobrevivência' },
		{ value: 'creative', label: 'Criativa' },
		{ value: 'adventure', label: 'Aventura' },
		{ value: 'spectator', label: 'Espectador' }
	];
	let worldSaving = $state(false);
	let ruleSaving = $state('');
	let ramDraft = $state(2048);
	let ramSaving = $state(false);
	let busyAction = $state('');
	let loaderBusy = $state('');
	let modpacks = $state<ModSearchResultItem[] | null>(null);
	let modpacksBusy = $state(false);
	let modpacksError = $state<string | null>(null);
	let packInstalling = $state('');
	let packError = $state('');
	let packProgress = $state('');
	let packQuery = $state('');
	let packOffset = $state(0);
	let packExhausted = $state(false);
	let packSource = $state<'modrinth' | 'curseforge'>('modrinth');
	let nameDraft = $state('');
	let persistedName = $state('');
	let nameSyncedId = '';
	let nameSaving = $state(false);
	let configDraft = $state({ port: 25565, motd: '', maxPlayers: 10 });
	let configSaving = $state(false);
	let consoleLines = $state<Array<{ key: string; text: string; tone: 'error' | 'warn' | 'info' }>>([]);
	let command = $state('');
	let sending = $state(false);
	let backups = $state<Array<{ id: string; createdAt: string; bytes: number; files: number }> | null>(null);
	let backupsBusy = $state(false);
	let backupCreating = $state(false);

	const config = $derived(platform.config);
	const modLoader = $derived(supportsMods(platform.loader.kind));
	const software = $derived(softwareByKind(platform.loader.kind));
	const visibleNodes = $derived.by(() => {
		if (!config) return [] as string[];
		const list = ['main', 'software', 'world', 'network', 'resources', 'eula', 'control', 'console', 'players', 'backups', 'config'];
		if (modLoader) list.splice(2, 0, 'modpacks');
		return list;
	});
	const activeEdges = $derived(EDGES.filter((edge) => visibleNodes.includes(edge.from) && visibleNodes.includes(edge.to)));
	const routed = $derived(activeEdges.map((edge) => ({ ...edge, fromSide: sideOf(edge.from, edge.to), toSide: sideInto(edge.from, edge.to) })));

	const balancedSides = $derived(balanceSides(routed));

	type Side = 'top' | 'right' | 'bottom' | 'left';

	const NORMALS: Record<Side, { x: number; y: number }> = {
		top: { x: 0, y: -1 },
		right: { x: 1, y: 0 },
		bottom: { x: 0, y: 1 },
		left: { x: -1, y: 0 }
	};

	function whOf(id: string): { w: number; h: number } {
		return { w: widths[id] ?? NODE_W[id] ?? 300, h: heights[id] ?? 150 };
	}

	const SIDE_ORDER: Side[] = ['top', 'right', 'bottom', 'left'];

	function centerOf(id: string): { x: number; y: number } {
		const p = posOf(id);
		const { w, h } = whOf(id);
		return { x: p.x + w / 2, y: p.y + h / 2 };
	}

	function angleTo(fromId: string, toId: string): number {
		const a = centerOf(fromId);
		const b = centerOf(toId);
		return Math.atan2(b.y - a.y, b.x - a.x);
	}

	function sideForAngle(angle: number): Side {
		const eighth = Math.PI / 4;
		if (angle >= -eighth && angle < eighth) return 'right';
		if (angle >= eighth && angle < 3 * eighth) return 'bottom';
		if (angle >= 3 * eighth || angle < -3 * eighth) return 'left';
		return 'top';
	}

	function sideOf(fromId: string, toId: string): Side {
		return sideForAngle(angleTo(fromId, toId));
	}

	function sideInto(fromId: string, toId: string): Side {
		return sideForAngle(angleTo(toId, fromId));
	}

	function opposite(side: Side): Side {
		return { right: 'left', left: 'right', top: 'bottom', bottom: 'top' }[side] as Side;
	}

	function pinPoint(id: string, side: Side, offset: number): { x: number; y: number } {
		const p = posOf(id);
		const { w, h } = whOf(id);
		if (side === 'right') return { x: p.x + w, y: p.y + h / 2 + offset };
		if (side === 'left') return { x: p.x, y: p.y + h / 2 + offset };
		if (side === 'top') return { x: p.x + w / 2 + offset, y: p.y };
		return { x: p.x + w / 2 + offset, y: p.y + h };
	}

	function pinsOf(id: string): Array<{ side: Side; offset: number }> {
		const perSide: Record<Side, number> = { top: 0, right: 0, bottom: 0, left: 0 };
		const order: Record<Side, number[]> = { top: [], right: [], bottom: [], left: [] };
		for (const edge of balancedSides) {
			if (edge.from === id) order[edge.fromSide].push(0);
			if (edge.to === id) order[edge.toSide].push(0);
		}
		const result: Array<{ side: Side; offset: number }> = [];
		for (const side of Object.keys(perSide) as Side[]) {
			const count = order[side].length;
			const spacing = Math.min(34, count > 1 ? 160 / (count - 1) : 0);
			for (let i = 0; i < count; i++) {
				const offset = count === 1 ? 0 : (i - (count - 1) / 2) * spacing;
				result.push({ side, offset: Math.round(offset) });
			}
		}
		return result;
	}

	function rotateSide(side: Side, steps: number): Side {
		return SIDE_ORDER[(SIDE_ORDER.indexOf(side) + steps + 4) % 4];
	}

	function balanceSides(list: Array<{ from: string; to: string; fromSide: Side; toSide: Side; theme: string }>) {
		const capacity = Math.max(1, Math.ceil(list.length / 4));
		const used: Record<Side, number> = { top: 0, right: 0, bottom: 0, left: 0 };
		const hub = 'main';
		const ordered = [...list].sort((a, b) => {
			const aPeer = a.from === hub ? a.to : a.from;
			const bPeer = b.from === hub ? b.to : b.from;
			return angleTo(hub, aPeer) - angleTo(hub, bPeer);
		});
		const result = new Map<(typeof list)[number], { fromSide: Side; toSide: Side }>();
		for (const edge of ordered) {
			const touchesHub = edge.from === hub || edge.to === hub;
			if (!touchesHub) {
				result.set(edge, { fromSide: edge.fromSide, toSide: edge.toSide });
				continue;
			}
			const base = edge.from === hub ? edge.fromSide : edge.toSide;
			let chosen = base;
			for (const step of [0, 1, -1, 2]) {
				const candidate = rotateSide(base, step);
				if (used[candidate] < capacity) {
					chosen = candidate;
					break;
				}
			}
			used[chosen] += 1;
			result.set(
				edge,
				edge.from === hub
					? { fromSide: chosen, toSide: opposite(chosen) }
					: { fromSide: opposite(chosen), toSide: chosen }
			);
		}
		return list.map((edge) => ({ ...edge, ...(result.get(edge) ?? { fromSide: edge.fromSide, toSide: edge.toSide }) }));
	}

	function edgePath(edge: { from: string; to: string; fromSide: Side; toSide: Side }): string {
		const { from: fromId, to: toId, fromSide, toSide } = edge;
		const fromOffset = offsetFor(fromId, fromSide, toId);
		const toOffset = offsetFor(toId, toSide, fromId);
		const p1 = pinPoint(fromId, fromSide, fromOffset);
		const p2 = pinPoint(toId, toSide, toOffset);
		const n1 = NORMALS[fromSide];
		const n2 = NORMALS[toSide];
		const dist = Math.hypot(p2.x - p1.x, p2.y - p1.y);
		const k = Math.max(42, Math.min(240, dist * 0.42));
		return `M ${p1.x} ${p1.y} C ${p1.x + n1.x * k} ${p1.y + n1.y * k}, ${p2.x + n2.x * k} ${p2.y + n2.y * k}, ${p2.x} ${p2.y}`;
	}

	function offsetFor(id: string, side: Side, peerId: string): number {
		const pins = pinsOf(id).filter((pin) => pin.side === side);
		if (pins.length <= 1) return 0;
		const wh = whOf(id);
		const peers = balancedSides
			.filter((edge) => (edge.from === id && edge.fromSide === side) || (edge.to === id && edge.toSide === side))
			.map((edge) => {
				const peer = edge.from === id ? edge.to : edge.from;
				const p = posOf(peer);
				const s = side === 'right' || side === 'left' ? p.y + (heights[peer] ?? 150) / 2 : p.x + (widths[peer] ?? NODE_W[peer] ?? 300) / 2;
				return { peer, s };
			})
			.sort((a, b) => a.s - b.s);
		const index = peers.findIndex((entry) => entry.peer === peerId);
		if (index < 0) return 0;
		const span = Math.min(160, (side === 'right' || side === 'left' ? wh.h : wh.w) * 0.7);
		const spacing = span / (peers.length - 1);
		return Math.round((index - (peers.length - 1) / 2) * spacing);
	}
	const servers = $derived(platform.servers);
	const lod = $derived<'full' | 'schematic' | 'map'>(zoom < 0.26 ? 'map' : zoom < 0.62 ? 'schematic' : 'full');
	const gaugePct = $derived(Math.min(100, Math.round((ramDraft / (platform.totalRamMb || 32768)) * 100)));
	const cpuPoints = $derived.by(() => {
		const history = platform.procCpuHistory;
		if (history.length < 2) return '0,35 120,35';
		return history.map((value, index) => `${((index / (history.length - 1)) * 120).toFixed(1)},${(35 - (Math.min(100, value) / 100) * 33).toFixed(1)}`).join(' ');
	});

	const LOADER_CHOICES = [SERVER_SOFTWARE[0], ...SERVER_SOFTWARE.filter((option) => ['fabric', 'forge', 'neoforge', 'quilt'].includes(option.kind))];

	const RULES: Array<{ key: keyof ServerSettings; label: string }> = [
		{ key: 'pvp', label: 'PvP' },
		{ key: 'hardcore', label: 'Modo Hardcore' },
		{ key: 'whitelist', label: 'Lista branca' },
		{ key: 'keepInventory', label: 'Manter inventário ao morrer' },
		{ key: 'daylightCycle', label: 'Ciclo dia/noite' },
		{ key: 'naturalRegeneration', label: 'Regeneração natural' },
		{ key: 'mobSpawning', label: 'Geração de criaturas' },
		{ key: 'monsterSpawning', label: 'Geração de monstros' },
		{ key: 'allowFlight', label: 'Voo permitido' },
		{ key: 'commandBlocks', label: 'Blocos de comando' }
	];

	$effect(() => {
		const settings = platform.settings;
		if (!settings) return;
		worldDraft = {
			seed: settings.seed ?? '',
			difficulty: settings.difficulty,
			gamemode: settings.gamemode,
			viewDistance: settings.viewDistance,
			simulationDistance: settings.simulationDistance
		};
		configDraft = { port: settings.port, motd: settings.motd, maxPlayers: config?.maxPlayers ?? 10 };
	});

	$effect(() => {
		ramDraft = platform.allocatedRamMb;
	});

	$effect(() => {
		const id = config?.id ?? '';
		const serverName = config?.name ?? '';
		if (id !== nameSyncedId) {
			nameSyncedId = id;
			persistedName = serverName;
			nameDraft = serverName;
		} else if (!nameSaving && nameDraft === persistedName && serverName !== persistedName) {
			persistedName = serverName;
			nameDraft = serverName;
		}
	});

	let packsKey = '';
	$effect(() => {
		if (!modLoader || !expanded.modpacks || !config) return;
		const key = [config.id, config.mcVersion, platform.loader.kind].join(":");
		if (packsKey === key) return;
		packsKey = key;
		untrack(() => void loadModpacks(platform.loader.kind, config.mcVersion));
	});

	$effect(() => {
		if (!expanded.console || !config) return;
		const id = config.id;
		const pull = async () => {
			try {
				const result = await serverConsoleText(id);
				const incoming = result.text.length === 0 ? [] : result.text.split('\n').slice(-14);
				consoleLines = incoming.map((raw, index) => ({ key: `${index}-${raw.slice(0, 24)}`, text: raw, tone: classifyLine(raw) }));
			} catch {
				consoleLines = [];
			}
		};
		untrack(() => void pull());
		const timer = setInterval(() => void pull(), 2000);
		return () => clearInterval(timer);
	});

	$effect(() => {
		if (!expanded.backups || !config) return;
		const id = config.id;
		const pull = async () => {
			try {
				const result = await serverManage<Array<{ id: string; createdAt: string; bytes: number; files: number }>>({ action: 'backups', id });
				backups = result;
			} catch {
				backups = null;
			}
		};
		untrack(() => void pull());
	});

	function classifyLine(raw: string): 'error' | 'warn' | 'info' {
		const lower = raw.toLowerCase();
		if (lower.includes('error') || lower.includes('exception') || lower.includes('fatal')) return 'error';
		if (lower.includes('warn') || lower.includes('aviso')) return 'warn';
		return 'info';
	}

	onMount(() => {
		try {
			const raw = localStorage.getItem(STORE_KEY);
			if (!raw) return;
			const stored: unknown = JSON.parse(raw);
			if (!stored || typeof stored !== 'object') return;
			const data = stored as Record<string, unknown>;
			if (data.positions && typeof data.positions === 'object') {
				const next: Record<string, NodePos> = { ...DEFAULT_POSITIONS };
				for (const [id, pos] of Object.entries(data.positions as Record<string, NodePos>)) {
					if (pos && typeof pos.x === 'number' && typeof pos.y === 'number') next[id] = pos;
				}
				positions = next;
			}
			if (data.styles && typeof data.styles === 'object') {
				const next: Record<string, NodeStyle> = { ...DEFAULT_STYLES };
				for (const [id, style] of Object.entries(data.styles as Record<string, NodeStyle>)) {
					if (style && DEFAULT_STYLES[id]) next[id] = { shape: style.shape, radius: style.radius, stroke: style.stroke, glass: style.glass, theme: style.theme };
				}
				styles = next;
			}
			if (data.expanded && typeof data.expanded === 'object') expanded = { ...expanded, ...(data.expanded as Record<string, boolean>) };
			if (typeof data.zoom === 'number') zoom = Math.min(2, Math.max(0.35, data.zoom));
			if (typeof data.panX === 'number') panX = data.panX;
			if (typeof data.panY === 'number') panY = data.panY;
		} catch {}
		return () => { cancelMomentum(); destroyWheel(); clearTimeout(zoomIdleTimer); };
	});

	function persist() {
		try {
			localStorage.setItem(STORE_KEY, JSON.stringify({ positions, styles, expanded, zoom, panX, panY }));
		} catch {}
	}

	function styleOf(id: string): NodeStyle {
		return styles[id] ?? DEFAULT_STYLES[id];
	}

	function radiusOf(id: string): string {
		const style = styleOf(id);
		if (style.shape === 'circle') return '50%';
		if (style.shape === 'pill') return '999px';
		if (style.shape === 'custom') return `${style.radius}px`;
		return '22px';
	}

	function themeCss(id: string): string {
		return THEMES[styleOf(id).theme]?.css ?? THEMES.blue.css;
	}

	function nodeStyleAttr(id: string): string {
		const style = styleOf(id);
		return `--node-theme: ${themeCss(id)}; --node-radius: ${radiusOf(id)}; --node-stroke: ${style.stroke}px; --node-glass: ${style.glass}`;
	}

	function posOf(id: string): NodePos {
		return positions[id] ?? DEFAULT_POSITIONS[id] ?? { x: 40, y: 40 };
	}

	function selectServer(id: string) {
		platform.select(id);
		void platform.loadServerData(id);
	}

	function stepServer(delta: number) {
		const list = platform.servers;
		if (list.length === 0) return;
		const index = list.findIndex((server) => server.id === platform.selectedId);
		const next = list[(index + delta + list.length) % list.length] ?? list[0];
		selectServer(next.id);
	}

	function toggleNode(id: string) {
		if (id === 'main') return;
		expanded = { ...expanded, [id]: !expanded[id] };
		persist();
	}

	function focusNode(id: string) {
		inspectorNodeId = id;
	}

	function cycleShape(id: string) {
		const order: NodeStyle['shape'][] = ['rounded', 'pill', 'circle', 'custom'];
		const current = styleOf(id).shape;
		const next = order[(order.indexOf(current) + 1) % order.length];
		styles = { ...styles, [id]: { ...styleOf(id), shape: next } };
		persist();
	}

	function cycleTheme(id: string) {
		const order = Object.keys(THEMES);
		const next = order[(order.indexOf(styleOf(id).theme) + 1) % order.length];
		styles = { ...styles, [id]: { ...styleOf(id), theme: next } };
		persist();
	}

	function fanOut() {
		positions = { ...DEFAULT_POSITIONS };
		persist();
	}

	function cancelMomentum() {
		if (momentumRaf !== null) {
			cancelAnimationFrame(momentumRaf);
			momentumRaf = null;
		}
	}

	const SNAP = 9;

	function snapAxis(value: number, id: string, axis: 'x' | 'y'): { value: number; guide: number | null } {
		const { w, h } = whOf(id);
		const size = axis === 'x' ? w : h;
		let best = value;
		let guide: number | null = null;
		for (const otherId of visibleNodes) {
			if (otherId === id) continue;
			const other = posOf(otherId);
			const otherSize = axis === 'x' ? whOf(otherId).w : whOf(otherId).h;
			const targets = [other[axis], other[axis] + otherSize / 2, other[axis] + otherSize];
			for (const target of targets) {
				const mine = [value, value + size / 2, value + size];
				for (const edge of mine) {
					const delta = target - edge;
					if (Math.abs(delta) <= SNAP && Math.abs(delta) < Math.abs(best - value || SNAP + 1)) {
						best = value + delta;
						guide = target;
					}
				}
			}
		}
		return { value: Math.round(best), guide };
	}

	function onNodeGrab(event: PointerEvent, id: string) {
		if (event.button !== 0) return;
		const target = event.target as HTMLElement;
		if (target.closest('button, input, select, label, a')) return;
		cancelMomentum();
		inspectorNodeId = id;
		const origin = { ...posOf(id) };
		const startX = event.clientX;
		const startY = event.clientY;
		const onHeader = Boolean(target.closest('.node-head'));
		let moved = false;
		let samples: Array<{ t: number; x: number; y: number }> = [];
		let latest = { ...origin };
		dragNode = id;
		let frame = 0;
		let pending: PointerEvent | null = null;
		const apply = () => {
			frame = 0;
			const moveEvent = pending;
			if (!moveEvent) return;
			const dx = moveEvent.clientX - startX;
			const dy = moveEvent.clientY - startY;
			const rawX = origin.x + dx / zoom;
			const rawY = origin.y + dy / zoom;
			const snappedX = snapAxis(rawX, id, 'x');
			const snappedY = snapAxis(rawY, id, 'y');
			if (snappedX.value === latest.x && snappedY.value === latest.y && frame === 0) return;
			latest = { x: snappedX.value, y: snappedY.value };
			guides = {
				x: snappedX.guide === null ? [] : [snappedX.guide],
				y: snappedY.guide === null ? [] : [snappedY.guide]
			};
			positions = { ...positions, [id]: latest };
		};
		const move = (moveEvent: PointerEvent) => {
			const dx = moveEvent.clientX - startX;
			const dy = moveEvent.clientY - startY;
			if (Math.abs(dx) + Math.abs(dy) > 3) moved = true;
			if (!moved) return;
			pending = moveEvent;
			if (!frame) frame = requestAnimationFrame(apply);
			const now = performance.now();
			samples.push({ t: now, x: moveEvent.clientX, y: moveEvent.clientY });
			while (samples.length > 6 || (samples.length > 1 && now - samples[0].t > 90)) samples.shift();
		};
		const up = () => {
			window.removeEventListener('pointermove', move);
			window.removeEventListener('pointerup', up);
			cancelAnimationFrame(frame);
			if (!moved) {
				dragNode = null;
				guides = { x: [], y: [] };
				if (onHeader) toggleNode(id);
				return;
			}
			if (samples.length >= 2) {
				const first = samples[0];
				const last = samples[samples.length - 1];
				const dt = Math.max(1, last.t - first.t);
				let vx = ((last.x - first.x) / dt) * 16;
				let vy = ((last.y - first.y) / dt) * 16;
				if (Math.hypot(vx, vy) > 12) {
					const glide = () => {
						vx *= 0.86;
						vy *= 0.86;
						if (Math.hypot(vx, vy) < 3) {
							momentumRaf = null;
							guides = { x: [], y: [] };
							positions = {
								...positions,
								[id]: { x: Math.round(latest.x / 10) * 10, y: Math.round(latest.y / 10) * 10 }
							};
							dragNode = null;
							persist();
							return;
						}
						latest = { x: latest.x + (vx / zoom), y: latest.y + (vy / zoom) };
						positions = { ...positions, [id]: latest };
						momentumRaf = requestAnimationFrame(glide);
					};
					momentumRaf = requestAnimationFrame(glide);
					return;
				}
			}
			dragNode = null;
			guides = { x: [], y: [] };
			persist();
		};
		window.addEventListener('pointermove', move);
		window.addEventListener('pointerup', up);
	}

	function onBackgroundDown(event: PointerEvent) {
		if (event.button !== 0) return;
		const target = event.target as HTMLElement;
		if (target.closest('.node, .tabs, .zoombar, button')) return;
		const startX = event.clientX;
		const startY = event.clientY;
		const originPanX = panX;
		const originPanY = panY;
		const move = (moveEvent: PointerEvent) => {
			panX = originPanX + (moveEvent.clientX - startX);
			panY = originPanY + (moveEvent.clientY - startY);
		};
		const up = () => {
			window.removeEventListener('pointermove', move);
			window.removeEventListener('pointerup', up);
			persist();
		};
		window.addEventListener('pointermove', move);
		window.addEventListener('pointerup', up);
	}

	function zoomAt(clientX: number, clientY: number, factor: number) {
		if (!viewportEl) return;
		const rect = viewportEl.getBoundingClientRect();
		const px = clientX - rect.left;
		const py = clientY - rect.top;
		const next = Math.min(2.2, Math.max(0.16, zoom * factor));
		if (next === zoom) return;
		const k = next / zoom;
		panX = px - (px - panX) * k;
		panY = py - (py - panY) * k;
		zoom = next;
		persist();
	}

	let wheelRaf = 0;
	let zoomIdleTimer: ReturnType<typeof setTimeout> | undefined;
	let wheelTargetX = 0, wheelTargetY = 0, wheelDelta = 0;
	function applyWheel() {
		wheelRaf = 0;
		viewportEl?.querySelector('.layer')?.classList.add('zooming');
		clearTimeout(zoomIdleTimer);
		zoomIdleTimer = setTimeout(() => viewportEl?.querySelector('.layer')?.classList.remove('zooming'), 160);
		const delta = wheelDelta;
		wheelDelta = 0;
		if (!delta) return;
		zoomAt(wheelTargetX, wheelTargetY, Math.exp(-delta * 0.0016));
	}
	function onWheel(event: WheelEvent) {
		event.preventDefault();
		if (event.ctrlKey) return;
		wheelTargetX = event.clientX;
		wheelTargetY = event.clientY;
		wheelDelta += event.deltaY;
		if (!wheelRaf) wheelRaf = requestAnimationFrame(applyWheel);
	}
	function destroyWheel() {
		if (wheelRaf) cancelAnimationFrame(wheelRaf);
		wheelRaf = 0;
	}

	function zoomStep(factor: number) {
		if (!viewportEl) return;
		const rect = viewportEl.getBoundingClientRect();
		zoomAt(rect.left + rect.width / 2, rect.top + rect.height / 2, factor);
	}

	function resetView() {
		zoom = 1;
		panX = 0;
		panY = 0;
		persist();
	}

	function fitView() {
		if (!viewportEl) return;
		const rect = viewportEl.getBoundingClientRect();
		let left = Infinity, top = Infinity, right = -Infinity, bottom = -Infinity;
		for (const id of visibleNodes) {
			const position = posOf(id);
			const size = whOf(id);
			left = Math.min(left, position.x);
			top = Math.min(top, position.y);
			right = Math.max(right, position.x + size.w);
			bottom = Math.max(bottom, position.y + size.h);
		}
		if (!Number.isFinite(left)) return;
		const padding = 120;
		const width = right - left + padding * 2;
		const height = bottom - top + padding * 2;
		const next = Math.min(1.15, Math.max(0.16, Math.min(rect.width / width, rect.height / height)));
		zoom = next;
		panX = rect.width / 2 - ((left + right) / 2) * next;
		panY = rect.height / 2 - ((top + bottom) / 2) * next;
		persist();
	}

	function briefOf(id: string): string {
		switch (id) {
			case 'main': return `${platform.playersOnline}/${config?.maxPlayers ?? 0} jogadores · ${formatGiB(platform.allocatedRamMb)}`;
			case 'software': return `${software.name}${platform.loader.version ? ' ' + platform.loader.version : ''}`;
			case 'world': return `${platform.settings?.difficulty ?? 'normal'} · ${platform.settings?.gamemode ?? 'survival'}`;
			case 'network': return platform.running ? publicAddress || lanAddress || 'sem endereço' : 'offline';
			case 'resources': return `${formatGiB(platform.allocatedRamMb)} · ${Math.round(platform.telemetry?.processCpuPercent ?? 0)}% CPU`;
			case 'eula': return config?.eulaAccepted ? 'aceito' : 'pendente';
			case 'control': return PHASE_LABEL[phaseOf()] ?? phaseOf();
			case 'console': return `${consoleLines.length} linhas`;
			case 'players': return `${platform.playersOnline} online`;
			case 'backups': return backups ? `${backups.length} backups` : '—';
			case 'config': return `porta ${platform.port ?? platform.settings?.port ?? '—'}`;
			default: return '';
		}
	}

	function phaseOf(): string {
		return platform.phase;
	}

	const PHASE_LABEL: Record<string, string> = {
		running: 'Online',
		starting: 'Iniciando',
		preparing: 'Preparando',
		downloading: 'Baixando',
		java: 'Preparando o Java',
		stopping: 'Encerrando',
		unreachable: 'Sem resposta',
		failed: 'Falhou',
		stopped: 'Offline'
	};

	async function setLoader(kind: string) {
		const target = config;
		if (!target || loaderBusy) return;
		if (kind === platform.loader.kind) {
			if (modLoader) expanded = { ...expanded, modpacks: true };
			return;
		}
		if (platform.activeForSelected && !['stopped', 'failed'].includes(platform.phase)) {
			toast('Encerre o servidor antes de trocar o software.', 'warning');
			return;
		}
		const previous = platform.loader;
		loaderBusy = kind;
		platform.loader = { kind, version: '' };
		expanded = { ...expanded, modpacks: supportsMods(kind) };
		try {
			if (kind === 'vanilla') {
				await serverManage({ action: 'set_loader', id: target.id, loader: { kind: 'vanilla', version: '' } });
				toast('Vanilla selecionado: o arquivo oficial é baixado durante a inicialização.', 'info');
			} else {
				const result = await loadersVersions(kind, target.mcVersion);
				const version = result.versions.find((item) => item.stable)?.id ?? result.versions[0]?.id ?? '';
				if (!version) {
					platform.loader = previous;
					expanded = { ...expanded, modpacks: supportsMods(previous.kind) };
					toast(`Nenhuma versão de ${softwareByKind(kind).name} é compatível com o Minecraft ${target.mcVersion}.`, 'error');
					return;
				}
				await serverManage({ action: 'set_loader', id: target.id, loader: { kind, version } });
				platform.loader = { kind, version };
				toast(`${softwareByKind(kind).name} ${version} será instalado pelo instalador oficial.`, 'success');
			}
			await platform.loadServerData(target.id);
		} catch (cause) {
			platform.loader = previous;
			expanded = { ...expanded, modpacks: supportsMods(previous.kind) };
			toast(serverMessage(cause), 'error');
		} finally {
			loaderBusy = '';
		}
	}

	const PACK_PAGE = 20;

	async function loadModpacks(_kind?: string, _mcVersion?: string, reset = true) {
		if (modpacksBusy) return;
		modpacksBusy = true;
		modpacksError = null;
		if (reset) {
			packOffset = 0;
			packExhausted = false;
			modpacks = [];
		}
		try {
			const loader = platform.loader.kind === 'vanilla' ? undefined : platform.loader.kind;
			const found = await modsSearch(packQuery.trim(), config?.mcVersion ?? '', PACK_PAGE, packOffset, 'modpack', packQuery.trim() ? 'relevance' : 'downloads', loader, undefined, packSource);
			const usable = found.filter((pack) => {
				const categories = (pack.categories ?? []).map((category) => category.toLowerCase());
				if (categories.length === 0) return true;
				return categories.some((category) => ['fabric', 'forge', 'neoforge', 'quilt', 'vanilla'].includes(category));
			});
			const fresh = usable.filter((pack) => !(modpacks ?? []).some((item) => item.sourceId === pack.sourceId && item.source === pack.source));
			modpacks = reset ? usable : [...(modpacks ?? []), ...fresh];
			packExhausted = found.length < PACK_PAGE || (!reset && fresh.length === 0);
			packOffset += found.length;
		} catch (cause) {
			if (reset) modpacks = null;
			if (packSource === 'curseforge') packSource = 'modrinth';
			modpacksError = friendlyPackError(serverMessage(cause));
		} finally {
			modpacksBusy = false;
		}
	}

	function friendlyPackError(message: string): string {
		const lower = message.toLowerCase();
		if (lower.includes('curseforge') && (lower.includes('chave') || lower.includes('api') || lower.includes('indispon'))) {
			return 'CurseForge precisa de uma chave de API neste ambiente. Voltando ao catálogo Modrinth.';
		}
		return message;
	}

	function onPackScroll(event: Event) {
		const node = event.currentTarget as HTMLElement;
		if (packExhausted || modpacksBusy) return;
		if (node.scrollHeight - node.scrollTop - node.clientHeight < 120) void loadModpacks(undefined, undefined, false);
	}

	async function installPack(pack: ModSearchResultItem) {
		const target = config;
		if (!target || packInstalling) return;
		packInstalling = pack.source + pack.sourceId;
		packError = '';
		try {
			if (platform.running) {
				packProgress = 'Encerrando o servidor…';
				await stopLocalServer(target.id);
			}
			const plan = await resolvePackPlan(pack);
			packProgress = 'Ajustando o servidor inteiro para o pack…';
			const applied = await provisionPack(target, pack, plan, { onProgress: (message) => (packProgress = message) });
			packProgress = '';
			toast(`Pack “${pack.title}” configurado: ${applied.length} ajustes aplicados. Inicie o servidor.`, 'success');
			await platform.loadServerData(target.id);
			await platform.refreshServers();
		} catch (cause) {
			packError = serverMessage(cause);
			packProgress = '';
		} finally {
			packInstalling = '';
		}
	}

	async function saveWorld() {
		const target = config;
		const settings = platform.settings;
		if (!target || !settings || worldSaving) return;
		worldSaving = true;
		try {
			const next = await saveServerSettings(target.id, {
				...settings,
				seed: worldDraft.seed.trim() === '' ? null : worldDraft.seed.trim(),
				difficulty: worldDraft.difficulty,
				gamemode: worldDraft.gamemode,
				viewDistance: worldDraft.viewDistance,
				simulationDistance: worldDraft.simulationDistance
			});
			platform.settings = { ...platform.settings!, ...next };
			toast('Mundo atualizado.', 'success');
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			worldSaving = false;
		}
	}

	async function toggleRule(key: keyof ServerSettings, checked: boolean) {
		const target = config;
		const settings = platform.settings;
		if (!target || !settings || ruleSaving) return;
		ruleSaving = key;
		platform.settings = { ...settings, [key]: checked };
		try {
			const next = await saveServerSettings(target.id, { ...settings, [key]: checked });
			platform.settings = { ...platform.settings!, ...next };
		} catch (cause) {
			platform.settings = settings;
			toast(serverMessage(cause), 'error');
		} finally {
			ruleSaving = '';
		}
	}

	async function saveRam() {
		const target = config;
		if (!target || ramSaving || ramDraft === target.ramMb) return;
		ramSaving = true;
		try {
			await updateLocalServer(target.id, target.name, ramDraft, target.maxPlayers);
			await platform.refreshServers();
			toast(`RAM do servidor definida para ${formatGiB(ramDraft)}.`, 'success');
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			ramSaving = false;
		}
	}

	function mirrorName() {
		const overview = platform.overview;
		if (!overview || !config) return;
		platform.overview = {
			...overview,
			servers: overview.servers.map((server) => (server.id === config.id ? { ...server, name: nameDraft } : server))
		};
	}

	async function saveName() {
		const target = config;
		if (!target || nameSaving || nameDraft.trim() === '' || nameDraft.trim() === persistedName) return;
		nameSaving = true;
		try {
			await updateLocalServer(target.id, nameDraft.trim(), target.ramMb, target.maxPlayers);
			persistedName = nameDraft.trim();
			await platform.refreshServers();
			toast('Nome do servidor atualizado.', 'success');
		} catch (cause) {
			nameDraft = target.name;
			mirrorName();
			toast(serverMessage(cause), 'error');
		} finally {
			nameSaving = false;
		}
	}

	async function saveConfig() {
		const target = config;
		const settings = platform.settings;
		if (!target || !settings || configSaving) return;
		configSaving = true;
		try {
			await updateLocalServer(target.id, target.name, target.ramMb, configDraft.maxPlayers);
			const next = await saveServerSettings(target.id, { ...settings, port: configDraft.port, motd: configDraft.motd });
			platform.settings = { ...platform.settings!, ...next };
			await platform.refreshServers();
			toast('Configuração salva.', 'success');
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			configSaving = false;
		}
	}

	let addressCopied = $state('');
	let onlineModeBusy = $state(false);
	let publicBusy = $state(false);
	let diagnosticBusy = $state(false);
	let showChecks = $state(false);
	let deleteOpen = $state(false);
	let deleteName = $state('');
	let deleting = $state(false);

	async function deleteServer() {
		const target = config;
		if (!target || deleting || deleteName !== target.name) return;
		deleting = true;
		try {
			await serverManage({ action: 'delete_server', id: target.id, name: target.name, deleteBackups: true });
			toast(`Servidor “${target.name}” excluído.`, 'success');
			deleteOpen = false;
			deleteName = '';
			await platform.refreshServers();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			deleting = false;
		}
	}

	const localhostAddress = $derived(platform.port ? `127.0.0.1:${platform.port}` : '');
	const lanAddress = $derived(platform.diagnostic?.lanAddresses?.[0] ?? '');
	const publicAddress = $derived(platform.publicAccess?.address || platform.diagnostic?.publicAddress || platform.publicAccess?.externalIp || '');

	async function copyAddress(label: string, value: string) {
		if (!value) {
			toast('Ligue o servidor para obter o endereço.', 'warning');
			return;
		}
		try {
			await navigator.clipboard.writeText(value);
			addressCopied = label;
			toast(`Endereço ${label} copiado.`, 'success');
			setTimeout(() => { if (addressCopied === label) addressCopied = ''; }, 1800);
		} catch {
			toast('Não foi possível copiar o endereço.', 'error');
		}
	}

	async function toggleOnlineMode(original: boolean) {
		const target = config;
		const settings = platform.settings;
		if (!target || !settings || onlineModeBusy) return;
		onlineModeBusy = true;
		try {
			const next = await saveServerSettings(target.id, { ...settings, onlineMode: original });
			platform.settings = { ...settings, ...next, onlineMode: original };
			toast(
				original
					? 'Contas originais exigidas. O servidor validará cada jogador na Mojang.'
					: 'Modo offline ativo: amigos com conta pirata ou sem conta podem entrar.',
				'success'
			);
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			onlineModeBusy = false;
		}
	}

	async function togglePublicAccess() {
		const target = config;
		if (!target || publicBusy) return;
		publicBusy = true;
		try {
			if (platform.publicAccess?.enabled) {
				await serverPublicAccess(target.id, 'disable');
				toast('Acesso externo encerrado.', 'info');
			} else {
				await serverPublicAccess(target.id, 'enable');
				toast('Servidor exposto na internet com redirecionamento automático de porta.', 'success');
			}
			await Promise.all([platform.refreshPublicAccess(), platform.refreshServers(), platform.refreshDiagnostic()]);
		} catch (cause) {
			toast(friendlyNetworkError(serverMessage(cause)), 'warning');
			await platform.refreshPublicAccess();
		} finally {
			publicBusy = false;
		}
	}

	function planJavaMajor(): number {
		const version = config?.mcVersion ?? '1.21';
		const parts = version.split('.').map((part) => Number.parseInt(part, 10) || 0);
		if ((parts[0] ?? 1) > 1 || (parts[1] ?? 0) >= 21) return 21;
		if ((parts[1] ?? 0) >= 18) return 17;
		return 8;
	}

	function friendlyNetworkError(message: string): string {
		const lower = message.toLowerCase();
		if (lower.includes('upnp') || lower.includes('gateway') || lower.includes('roteador') || lower.includes('10060') || lower.includes('não respondeu')) {
			return 'Seu roteador não aceitou a abertura automática de porta (UPnP desligado ou sem suporte). Amigos da mesma rede já podem entrar pelo endereço local. Para amigos de fora, use a sala P2P em Amigos ou libere a porta no roteador.';
		}
		if (lower.includes('cgnat')) return 'Sua operadora usa CGNAT: abertura de porta não funciona. Use a sala P2P em Amigos para jogar com quem está fora da sua rede.';
		if (lower.includes('permiss') || lower.includes('pro')) return 'A abertura automática de porta faz parte do Luxmc Plus/Pro. Alternativa gratuita: sala P2P em Amigos.';
		if (lower.includes('parado') || lower.includes('offline') || lower.includes('inicie')) return 'Inicie o servidor antes de abrir a porta para a internet.';
		return message;
	}

	async function runNetworkDiagnostic() {
		if (diagnosticBusy) return;
		diagnosticBusy = true;
		try {
			await Promise.all([platform.refreshDiagnostic(), platform.refreshPublicAccess()]);
			toast('Diagnóstico de rede atualizado.', 'info');
		} finally {
			diagnosticBusy = false;
		}
	}

	$effect(() => {
		if (!expanded.network || !config) return;
		const id = config.id;
		untrack(() => { void platform.refreshDiagnostic(); void platform.refreshPublicAccess(); });
		const timer = setInterval(() => {
			if (platform.selectedId === id) void platform.refreshDiagnostic();
		}, 15000);
		return () => clearInterval(timer);
	});

	async function sendCommand() {
		const target = config;
		const text = command.trim();
		if (!target || sending || text === '') return;
		sending = true;
		try {
			await sendServerCommand(target.id, text.startsWith('/') ? text : `/${text}`);
			command = '';
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			sending = false;
		}
	}

	async function createBackup() {
		const target = config;
		if (!target || backupCreating) return;
		backupCreating = true;
		try {
			await serverManage({ action: 'backup', id: target.id });
			backups = await serverManage<Array<{ id: string; createdAt: string; bytes: number; files: number }>>({ action: 'backups', id: target.id });
			toast('Backup criado.', 'success');
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			backupCreating = false;
		}
	}

	async function restoreBackup(id: string) {
		const target = config;
		if (!target || backupsBusy) return;
		backupsBusy = true;
		try {
			await serverManage({ action: 'restore_backup', id: target.id, backup: id });
			toast('Backup restaurado.', 'success');
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			backupsBusy = false;
		}
	}

	async function start() {
		const target = config;
		if (!target || busyAction) return;
		busyAction = 'start';
		try {
			await startLocalServer(target.id);
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyAction = '';
		}
	}

	async function stop() {
		const target = config;
		if (!target || busyAction) return;
		busyAction = 'stop';
		try {
			await stopLocalServer(target.id);
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyAction = '';
		}
	}

	async function restart() {
		const target = config;
		if (!target || busyAction) return;
		busyAction = 'restart';
		try {
			await stopLocalServer(target.id);
			await startLocalServer(target.id);
			toast('Servidor reiniciado.', 'success');
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyAction = '';
		}
	}

	async function openFolder() {
		const target = config;
		if (!target) return;
		try {
			await openLocalServerFolder(target.id);
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		}
	}

	function setShape(shape: NodeStyle['shape']) {
		const id = inspectorNodeId;
		styles = { ...styles, [id]: { ...styleOf(id), shape } };
		persist();
	}

	function setTheme(theme: string) {
		const id = inspectorNodeId;
		styles = { ...styles, [id]: { ...styleOf(id), theme } };
		persist();
	}

	function patchStyle(patch: Partial<NodeStyle>) {
		const id = inspectorNodeId;
		styles = { ...styles, [id]: { ...styleOf(id), ...patch } };
		persist();
	}
</script>

<div class="canvas" bind:this={viewportEl} onpointerdown={onBackgroundDown} onwheel={onWheel} role="application" aria-label="Área de trabalho dos servidores">
	<div class="space" aria-hidden="true"></div>
	<div class="layer lod-{lod}" class:dragging={dragNode !== null} style="transform: translate({panX}px, {panY}px) scale({zoom}); --lodpx: {(1 / zoom).toFixed(4)}">
		<div class="dust" aria-hidden="true">
			{#each STARS as star (star.id)}
				<span class="star" style="left: {star.x}px; top: {star.y}px; width: {star.s}px; height: {star.s}px; opacity: {star.o}"></span>
			{/each}
			{#each FORMULAS as formula, index (index)}
				<span class="formula" style="left: {formula.x}%; top: {formula.y}%; font-size: {formula.size}px; rotate: {formula.rot}deg">{formula.text}</span>
			{/each}
		</div>

		<svg class="wires" aria-hidden="true">
			{#each balancedSides as edge (edge.from + edge.to)}
				{@const path = edgePath(edge)}
				{#if path}
					<path class="wire" d={path} style="stroke: {THEMES[edge.theme]?.css ?? THEMES.blue.css}"></path>
					{#if platform.running}
						<path class="wire-flow" d={path} style="stroke: {THEMES[edge.theme]?.css ?? THEMES.blue.css}"></path>
					{/if}
				{/if}
			{/each}
		</svg>

		{#if guides.x.length || guides.y.length}
			<svg class="guides" aria-hidden="true">
				{#each guides.x as x (x)}
					<line x1={x} y1={-4000} x2={x} y2={8000}></line>
				{/each}
				{#each guides.y as y (y)}
					<line x1={-4000} y1={y} x2={8000} y2={y}></line>
				{/each}
			</svg>
		{/if}

		{#if config}
			{#each visibleNodes as nodeId (nodeId)}
				{@const style = styleOf(nodeId)}
				{@const isOpen = nodeId === 'main' || expanded[nodeId]}
				{@const NodeIcon = NODE_DEFS[nodeId].icon}
				<div
					class="node"
					class:dragging={dragNode === nodeId}
					class:open={isOpen}
					style="{nodeStyleAttr(nodeId)}; left: {posOf(nodeId).x}px; top: {posOf(nodeId).y}px; width: {NODE_W[nodeId] ?? 300}px"
					role="group"
					aria-label="Módulo {NODE_DEFS[nodeId].label}"
					bind:clientHeight={heights[nodeId]}
					bind:clientWidth={widths[nodeId]}
					oncontextmenu={(event) => { event.preventDefault(); inspectorNodeId = nodeId; inspectorOpen = true; }}
				>
					{#each pinsOf(nodeId) as pin, index (`${pin.side}-${index}`)}
						<span
							class="port pin-{pin.side}"
							style={pin.side === 'right' || pin.side === 'left' ? `top: calc(50% + ${pin.offset}px - var(--pin-r))` : `left: calc(50% + ${pin.offset}px - var(--pin-r))`}
							aria-hidden="true"
						></span>
					{/each}
					<div class="handles" aria-hidden="false">
						<button type="button" class="handle" title="Alternar forma do nó" aria-label="Alternar forma do nó" onclick={() => cycleShape(nodeId)}><Shapes class="h-3 w-3" /></button>
						<button type="button" class="handle" title="Alternar cor do nó" aria-label="Alternar cor do nó" onclick={() => cycleTheme(nodeId)}><Palette class="h-3 w-3" /></button>
						<button type="button" class="handle" title="Abrir inspetor de estilo" aria-label="Abrir inspetor de estilo" onclick={() => { inspectorNodeId = nodeId; inspectorOpen = true; }}><SlidersHorizontal class="h-3 w-3" /></button>
					</div>
					<header
						class="node-head"
						class:collapsed={!isOpen}
						onpointerdown={(event) => onNodeGrab(event, nodeId)}
						role="button"
						tabindex="0"
						aria-label="Mover ou alternar o módulo {NODE_DEFS[nodeId].label}"
						onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ') toggleNode(nodeId); }}
					>
						<GripVertical class="grip h-3.5 w-3.5" />
						<span class="node-icon"><NodeIcon class="h-3.5 w-3.5" /></span>
						<span class="node-label">{NODE_DEFS[nodeId].label}</span>
						{#if nodeId !== 'main'}
							<button type="button" class="node-x" title="Recolher módulo" onclick={(event) => { event.stopPropagation(); toggleNode(nodeId); }}>
								<X class="h-3 w-3" />
							</button>
						{/if}
					</header>

					<span class="brief" aria-hidden="true">{briefOf(nodeId)}</span>

					{#if isOpen && nodeId === 'main' && config}
						<div class="body-main">
							<h2 class="server-name">{config.name}</h2>
							<p class="server-sub">
								Minecraft {config.mcVersion} · {PHASE_LABEL[phaseOf()] ?? phaseOf()}{platform.port ? ` · porta ${platform.port}` : ''}
							</p>
							<div class="main-stats">
								<span class="stat"><Users class="h-3.5 w-3.5" /> {platform.playersOnline}/{config.maxPlayers}</span>
								<span class="stat"><MemoryStick class="h-3.5 w-3.5" /> {formatGiB(platform.allocatedRamMb)}</span>
								<span class="stat"><Puzzle class="h-3.5 w-3.5" /> {software.name}{platform.loader.kind !== 'vanilla' ? ` ${platform.loader.version}` : ''}</span>
							</div>
							<button type="button" class="mini" onclick={fanOut}>
								<Network class="h-3.5 w-3.5" /> Expandir módulos
							</button>
						</div>
					{/if}

					{#if isOpen && nodeId === 'software'}
						<div class="body-node">
							<div class="loader-grid">
								{#each LOADER_CHOICES as option (option.kind)}
									<button
										type="button"
										class="chip"
										class:active={platform.loader.kind === option.kind}
										disabled={loaderBusy !== ''}
										onclick={() => void setLoader(option.kind)}
									>
										{#if loaderBusy === option.kind}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{:else}<Puzzle class="h-3.5 w-3.5" />{/if}
										{option.name}
									</button>
								{/each}
							</div>
							<p class="node-note">
								{#if platform.loader.kind === 'vanilla'}
									Vanilla ativo: o arquivo oficial da Mojang é baixado durante a inicialização. O módulo de modpacks fica oculto.
								{:else}
									{software.name} {platform.loader.version} ativo. Os modpacks compatíveis aparecem no sub-nó conectado.
								{/if}
							</p>
						</div>
					{/if}

					{#if isOpen && nodeId === 'modpacks' && modLoader}
						<div class="body-node">
							<div class="pack-search">
								<input
									type="search"
									placeholder="Buscar modpack…"
									bind:value={packQuery}
									onkeydown={(event) => { if (event.key === 'Enter') void loadModpacks(undefined, undefined, true); }}
									aria-label="Buscar modpack para este servidor"
								/>
								<button type="button" class="mini" aria-label="Pesquisar modpacks" onclick={() => void loadModpacks(undefined, undefined, true)}>
									<Search class="h-3.5 w-3.5" />
								</button>
							</div>
							<div class="pack-sources">
								<button type="button" class="chip" class:active={packSource === 'modrinth'} onclick={() => { packSource = 'modrinth'; void loadModpacks(undefined, undefined, true); }}>Modrinth</button>
								<button type="button" class="chip" class:active={packSource === 'curseforge'} onclick={() => { packSource = 'curseforge'; void loadModpacks(undefined, undefined, true); }}>CurseForge</button>
							</div>
							{#if modpacksBusy && (modpacks?.length ?? 0) === 0}
								<p class="node-note busy"><LoaderCircle class="h-3.5 w-3.5 animate-spin" /> Consultando o catálogo oficial…</p>
							{:else if modpacksError}
								<p class="node-note bad">{modpacksError}</p>
								<button type="button" class="mini" onclick={() => void loadModpacks(undefined, undefined, true)}>Tentar de novo</button>
							{:else if modpacks && modpacks.length > 0}
								<ul class="packs scrollable" onscroll={onPackScroll}>
									{#each modpacks as pack (pack.source + pack.sourceId)}
										<li class="pack">
											{#if pack.iconUrl}<img src={pack.iconUrl} alt="" class="pack-icon" loading="lazy" />{:else}<span class="pack-icon fallback"><Package class="h-3.5 w-3.5" /></span>{/if}
											<span class="pack-info">
												<span class="pack-title">{pack.title}</span>
												<span class="pack-meta">{compact.format(pack.downloads)} downloads · {pack.author ?? 'Autor desconhecido'}</span>
											</span>
											<button
												type="button"
												class="mini"
												disabled={packInstalling !== ''}
												aria-label={`Instalar o pack ${pack.title} neste servidor`}
												onclick={() => void installPack(pack)}
											>
												{#if packInstalling === pack.source + pack.sourceId}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{:else}<Download class="h-3.5 w-3.5" />{/if}
												Usar pack
											</button>
										</li>
									{/each}
								</ul>
								{#if modpacksBusy && (modpacks?.length ?? 0) > 0}<p class="node-note busy"><LoaderCircle class="h-3.5 w-3.5 animate-spin" /> Carregando mais packs…</p>{/if}
								{#if packProgress}<p class="node-note busy"><LoaderCircle class="h-3.5 w-3.5 animate-spin" /> {packProgress}</p>{/if}
								{#if packError}<p class="node-note bad">{packError}</p>{/if}
								<p class="node-note">Usar pack fixa a versão, troca o loader e libera arede/porta automaticamente. O servidor precisa estar parado e sem mods aplicados.</p>
							{:else}
								<p class="node-note">Nenhum modpack do catálogo para {software.name} no Minecraft {config?.mcVersion}.</p>
							{/if}
						</div>
					{/if}

					{#if isOpen && nodeId === 'world' && config}
						<div class="body-node">
							{#if platform.settings}
								<label class="field">
									<span>Seed do mundo</span>
									<input type="text" bind:value={worldDraft.seed} placeholder="Aleatória" />
								</label>
								<label class="field">
									<span>Dificuldade</span>
									<GlassSelect options={DIFFICULTY_OPTIONS} bind:value={worldDraft.difficulty} ariaLabel="Dificuldade" />
								</label>
								<label class="field">
									<span>Modo de jogo</span>
									<GlassSelect options={GAMEMODE_OPTIONS} bind:value={worldDraft.gamemode} ariaLabel="Modo de jogo" />
								</label>
								<label class="slider">
									<span>Distância de renderização · {worldDraft.viewDistance} chunks</span>
									<input type="range" min="2" max="32" step="1" bind:value={worldDraft.viewDistance} />
								</label>
								<label class="slider">
									<span>Distância de simulação · {worldDraft.simulationDistance} chunks</span>
									<input type="range" min="3" max="16" step="1" bind:value={worldDraft.simulationDistance} />
								</label>
								<button type="button" class="mini accent" disabled={worldSaving} onclick={() => void saveWorld()}>
									{#if worldSaving}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{/if}
									Salvar mundo
								</button>
								<div class="divider"><span>Regras do jogo</span></div>
								{#each RULES as rule (rule.key)}
									<label class="switch-row">
										<span>{rule.label}</span>
										<span class="switch">
											<input
												type="checkbox"
												checked={Boolean(platform.settings[rule.key])}
												disabled={ruleSaving !== ''}
												onchange={(event) => void toggleRule(rule.key, event.currentTarget.checked)}
											/>
											<span class="track" class:waiting={ruleSaving === rule.key}><span class="knob"></span></span>
										</span>
									</label>
								{/each}
							{:else}
								<p class="node-note">Carregando configurações do mundo…</p>
							{/if}
						</div>
					{/if}

					{#if isOpen && nodeId === 'network' && config}
						<div class="body-node">
							<div class="net-mode" role="group" aria-label="Modo de autenticação">
								<button
									type="button"
									class="mode"
									class:active={platform.settings?.onlineMode !== false}
									disabled={onlineModeBusy || !platform.settings}
									onclick={() => void toggleOnlineMode(true)}
								>
									<Lock class="h-3.5 w-3.5" /> Originais
								</button>
								<button
									type="button"
									class="mode"
									class:active={platform.settings?.onlineMode === false}
									disabled={onlineModeBusy || !platform.settings}
									onclick={() => void toggleOnlineMode(false)}
								>
									<Unlock class="h-3.5 w-3.5" /> Pirata / Offline
								</button>
							</div>
							<p class="node-note">
								{platform.settings?.onlineMode === false
									? 'Qualquer pessoa com o endereço entra, sem validação de conta. Nomes podem ser falsificados.'
									: 'Cada jogador é validado na Mojang. Contas originais obrigatórias.'}
								{#if platform.running}<strong class="restart-hint">Reinicie para aplicar.</strong>{/if}
							</p>

							<div class="divider"><span>Endereços</span></div>
							{#if platform.running && localhostAddress}
								<div class="addr-row">
									<span class="addr-label">Neste PC</span>
									<code class="addr">{localhostAddress}</code>
									<button type="button" class="copy" title="Copiar endereço local" aria-label="Copiar endereço local" onclick={() => void copyAddress('local', localhostAddress)}>
										{#if addressCopied === 'local'}<Check class="h-3.5 w-3.5" />{:else}<Copy class="h-3.5 w-3.5" />{/if}
									</button>
								</div>
								{#if lanAddress}
									<div class="addr-row">
										<span class="addr-label">Rede local</span>
										<code class="addr">{lanAddress}</code>
										<button type="button" class="copy" title="Copiar endereço da rede local" aria-label="Copiar endereço da rede local" onclick={() => void copyAddress('LAN', lanAddress)}>
											{#if addressCopied === 'LAN'}<Check class="h-3.5 w-3.5" />{:else}<Copy class="h-3.5 w-3.5" />{/if}
										</button>
									</div>
								{:else}
									<p class="node-hint">Nenhum IPv4 de rede local detectado. Confira o Wi-Fi ou o cabo.</p>
								{/if}
								{#if publicAddress}
									<div class="addr-row accent">
										<span class="addr-label">Internet</span>
										<code class="addr">{publicAddress}</code>
										<button type="button" class="copy" title="Copiar endereço público" aria-label="Copiar endereço público" onclick={() => void copyAddress('público', publicAddress)}>
											{#if addressCopied === 'público'}<Check class="h-3.5 w-3.5" />{:else}<Copy class="h-3.5 w-3.5" />{/if}
										</button>
									</div>
								{/if}
							{:else}
								<div class="addr-row">
									<span class="addr-label">Neste PC</span>
									<code class="addr">127.0.0.1:{platform.settings?.port || 25565}</code>
									<button type="button" class="copy" title="Copiar endereço local" aria-label="Copiar endereço local" onclick={() => void copyAddress('local', '127.0.0.1:' + (platform.settings?.port || 25565))}>
										{#if addressCopied === 'local'}<Check class="h-3.5 w-3.5" />{:else}<Copy class="h-3.5 w-3.5" />{/if}
									</button>
								</div>
								<p class="node-hint">O servidor está desligado: o endereço só aceita conexões depois de clicar em Iniciar.</p>
							{/if}

							<div class="divider"><span>Como entrar no jogo</span></div>
							<ol class="join-steps">
								<li>Abra o Minecraft <strong>Java {planJavaMajor()}</strong> na mesma versão <strong>{config?.mcVersion}</strong>.</li>
								{#if platform.loader.kind !== 'vanilla'}
									<li>Instale o mesmo loader: <strong>{software.name} {platform.loader.version}</strong>.</li>
								{/if}
								<li>Multijogador → Adicionar servidor → cole o endereço acima.</li>
								<li>Mesma rede: use o endereço <strong>Rede local</strong>. Outra rede: sala P2P em <strong>Amigos</strong>.</li>
							</ol>

							<div class="divider"><span>Acesso pela internet</span></div>
							<label class="switch-row">
								<span>Abrir porta (UPnP)</span>
								<span class="switch">
									<input
										type="checkbox"
										checked={platform.publicAccess?.enabled ?? false}
										disabled={publicBusy || !platform.running}
										onchange={() => void togglePublicAccess()}
									/>
									<span class="track"><span class="knob"></span></span>
								</span>
							</label>
							{#if platform.publicAccess?.likelyBehindCgnat}
								<p class="node-hint warn">Sua operadora parece usar CGNAT. Compartilhe pelo endereço P2P em Amigos.</p>
							{:else if platform.publicAccess?.enabled}
								<p class="node-hint ok">Porta {platform.publicAccess.port} redirecionada no roteador. Amigos já podem entrar pelo endereço de internet.</p>
							{:else}
								<p class="node-hint">Sem redirecionamento: apenas a sua rede local alcança o servidor.</p>
							{/if}

							<button type="button" class="mini" disabled={diagnosticBusy} onclick={() => void runNetworkDiagnostic()}>
								{#if diagnosticBusy}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{:else}<RefreshCw class="h-3.5 w-3.5" />{/if}
								Testar conexão
							</button>
							{#if platform.diagnostic}
								<div class="tele">
									<div class="tele-row"><span>Resposta Minecraft</span><strong class:good={platform.diagnostic.minecraftResponding}>{platform.diagnostic.minecraftResponding ? 'Sim' : 'Não'}</strong></div>
									<div class="tele-row"><span>Porta TCP</span><strong class:good={platform.diagnostic.tcpReachable}>{platform.diagnostic.tcpReachable ? 'Aberta' : 'Fechada'}</strong></div>
								</div>
									{#if platform.diagnostic.checks?.length}
										<button type="button" class="mini" aria-expanded={showChecks} onclick={() => (showChecks = !showChecks)}>
											{showChecks ? 'Ocultar' : 'Ver'} as {platform.diagnostic.checks.length} verificações
										</button>
										{#if showChecks}
											<ul class="checks">
												{#each platform.diagnostic.checks as check (check.id)}
													<li class="check" class:ok={check.status === 'approved'} class:bad={check.status === 'failed'} class:warn={check.status === 'attention'}>
														<span class="check-label">{check.label}</span>
														<span class="check-status">{check.statusLabel}</span>
													</li>
												{/each}
											</ul>
										{/if}
									{/if}
							{/if}
						</div>
					{/if}

					{#if isOpen && nodeId === 'resources' && config}
						<div class="body-node">
							<div class="dial-wrap">
								<svg viewBox="0 0 120 120" class="dial" role="img" aria-label="RAM alocada do total do sistema">
									<defs>
										<linearGradient id="bezel" x1="0" y1="0" x2="1" y2="1">
											<stop offset="0" stop-color="rgb(255 255 255 / 0.4)"></stop>
											<stop offset="0.45" stop-color="rgb(0 0 0 / 0.4)"></stop>
											<stop offset="1" stop-color="rgb(255 255 255 / 0.22)"></stop>
										</linearGradient>
									</defs>
									<circle cx="60" cy="60" r="54" fill="none" stroke="url(#bezel)" stroke-width="7"></circle>
									<circle cx="60" cy="60" r="44" fill="none" stroke="rgb(0 0 0 / 0.42)" stroke-width="11"></circle>
									<circle
										class="dial-arc"
										cx="60"
										cy="60"
										r="44"
										fill="none"
										stroke="var(--node-theme)"
										stroke-width="11"
										stroke-linecap="round"
										stroke-dasharray={2 * Math.PI * 44}
										stroke-dashoffset={2 * Math.PI * 44 * (1 - gaugePct / 100)}
										transform="rotate(-90 60 60)"
									></circle>
									<text x="60" y="56" text-anchor="middle" class="dial-big">{formatGiB(ramDraft)}</text>
									<text x="60" y="74" text-anchor="middle" class="dial-small">de {formatGiB(platform.totalRamMb || 32768)}</text>
								</svg>
								<div class="spark-wrap">
									<span class="spark-label">CPU do processo</span>
									<svg viewBox="0 0 120 36" class="spark" preserveAspectRatio="none" role="img" aria-label="Histórico de CPU">
										<polyline points={cpuPoints} fill="none" stroke="var(--node-theme)" stroke-width="1.6" stroke-linejoin="round"></polyline>
									</svg>
								</div>
							</div>
							<label class="slider">
								<span>RAM dedicada · {formatGiB(ramDraft)}</span>
								<input type="range" min="1024" max={Math.max(2048, platform.totalRamMb || 32768)} step="512" bind:value={ramDraft} />
							</label>
							<button type="button" class="mini accent" disabled={ramSaving || ramDraft === config.ramMb} onclick={() => void saveRam()}>
								{#if ramSaving}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{/if}
								Aplicar RAM
							</button>
							<div class="tele">
								<div class="tele-row"><span>Heap Java</span><strong>{platform.telemetry?.online ? `${Math.round(platform.telemetry.heapUsedMb ?? 0)} / ${Math.round(platform.telemetry.heapMaxMb ?? 0)} MB` : '—'}</strong></div>
								<div class="tele-row"><span>TPS</span><strong>{platform.telemetry?.online && platform.telemetry.tps !== null ? platform.telemetry.tps.toFixed(1) : '—'}</strong></div>
								<div class="tele-row"><span>Uptime</span><strong>{platform.telemetry?.online && platform.telemetry.uptimeSeconds !== null ? `${Math.floor(platform.telemetry.uptimeSeconds / 60)} min` : '—'}</strong></div>
							</div>
						</div>
					{/if}

					{#if isOpen && nodeId === 'eula' && config}
						<div class="body-node">
							<div class="eula-badge" class:ok={config.eulaAccepted}>
								<ShieldCheck class="h-5 w-5" />
								<span>{config.eulaAccepted ? 'EULA aceito' : 'EULA pendente'}</span>
							</div>
							<p class="node-note">
								{#if config.eulaAccepted}
									O EULA da Mojang foi aceito na criação deste servidor e o arquivo eula.txt reflete isso na primeira inicialização.
								{:else}
									O EULA da Mojang precisa ser aceito para permitir jogadores online. Ele é registrado ao criar o servidor marcando a opção no assistente.
								{/if}
							</p>
							<div class="tele">
								<div class="tele-row"><span>Modo online</span><strong>{platform.settings?.onlineMode === false ? 'Offline (pirata)' : 'Original'}</strong></div>
								<div class="tele-row"><span>Proteção de spawn</span><strong>{platform.settings?.spawnProtection ?? 16} blocos</strong></div>
							</div>
						</div>
					{/if}

					{#if isOpen && nodeId === 'control' && config}
						<div class="body-node">
							{#if platform.activeForSelected && !['stopped', 'failed'].includes(platform.phase)}
								<button type="button" class="start-block stop" disabled={actionBusy || busyAction !== ''} onclick={() => void stop()}>
									{#if busyAction === 'stop' || platform.transition}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Square class="h-4 w-4" />{/if}
									ENCERRAR SERVIDOR
								</button>
							{:else}
								<button type="button" class="start-block" disabled={actionBusy || busyAction !== ''} onclick={() => void start()}>
									{#if busyAction === 'start'}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Play class="h-4 w-4" />{/if}
									INICIAR SERVIDOR
								</button>
							{/if}
							{#if platform.activeForSelected && !['stopped', 'failed'].includes(platform.phase)}
								<div class="control-row">
									<button type="button" class="mini" disabled={busyAction !== ''} onclick={() => void restart()} title="Reinicia mantendo o mundo e os mods">
										<RefreshCw class="h-3.5 w-3.5" /> Reiniciar
									</button>
								</div>
							{/if}
							<div class="control-row">
								<button type="button" class="mini" onclick={() => void openFolder()}>
									<FolderOpen class="h-3.5 w-3.5" /> Pasta do servidor
								</button>
								<button type="button" class="mini" onclick={() => void copyAddress('endereço', localhostAddress || ('127.0.0.1:' + (platform.port ?? platform.settings?.port ?? 25565)))}>
									<Copy class="h-3.5 w-3.5" /> Copiar IP
								</button>
							</div>
							<p class="node-note">
								{platform.selectedState?.error ? platform.selectedState.error : `Fase atual: ${PHASE_LABEL[phaseOf()] ?? phaseOf()}.`}
							</p>
						</div>
					{/if}

					{#if isOpen && nodeId === 'console' && config}
						<div class="body-node">
							<div class="console-lines">
								{#if consoleLines.length === 0}
									<p class="node-note">Sem saída de console ainda. Inicie o servidor para acompanhar.</p>
								{:else}
									{#each consoleLines as line (line.key)}
										<p class="cline {line.tone}">{line.text}</p>
									{/each}
								{/if}
							</div>
							<div class="cmd-row">
								<input
									type="text"
									bind:value={command}
									placeholder="Comando (ex: list)"
									onkeydown={(event) => { if (event.key === 'Enter') void sendCommand(); }}
								/>
								<button type="button" class="mini accent" disabled={sending || command.trim() === ''} onclick={() => void sendCommand()} aria-label="Enviar comando">
									{#if sending}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{:else}<Send class="h-3.5 w-3.5" />{/if}
								</button>
							</div>
						</div>
					{/if}

					{#if isOpen && nodeId === 'players' && config}
						<div class="body-node">
							{#if platform.selectedState}
								{@const players = platform.selectedState.players}
								<p class="node-note">
									{platform.selectedState.playersOnline}/{config.maxPlayers} online · {players.length} registrados
								</p>
								{#if players.length === 0}
									<p class="node-note">Nenhum jogador conectou ainda.</p>
								{:else}
									<ul class="player-list">
										{#each [...players].sort((a, b) => Number(b.online) - Number(a.online)).slice(0, 8) as player (player.uuid)}
											<li class="player-row">
												<span class="pdot" class:on={player.online}></span>
												<span class="pname">{player.name}</span>
												<span class="pmeta">{player.visits} visitas</span>
											</li>
										{/each}
									</ul>
								{/if}
							{:else}
								<p class="node-note">Servidor offline — inicie para ver jogadores.</p>
							{/if}
						</div>
					{/if}

					{#if isOpen && nodeId === 'backups' && config}
						<div class="body-node">
							<button type="button" class="mini accent" disabled={backupCreating} onclick={() => void createBackup()}>
								{#if backupCreating}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{:else}<Plus class="h-3.5 w-3.5" />{/if}
								Criar backup
							</button>
							{#if backups === null}
								<p class="node-note">Carregando backups…</p>
							{:else if backups.length === 0}
								<p class="node-note">Nenhum backup ainda. Crie o primeiro ponto de restauração.</p>
							{:else}
								<ul class="packs">
									{#each backups.slice(0, 5) as backup (backup.id)}
										<li class="pack">
											<span class="pack-info">
												<span class="pack-title">{formatDateTime(backup.createdAt)}</span>
												<span class="pack-meta">{formatBytes(backup.bytes)} · {backup.files} arquivos</span>
											</span>
											<button type="button" class="mini" disabled={backupsBusy} onclick={() => void restoreBackup(backup.id)}>Restaurar</button>
										</li>
									{/each}
								</ul>
							{/if}
						</div>
					{/if}

					{#if isOpen && nodeId === 'config' && config}
						<div class="body-node">
							<label class="field">
								<span>Nome do servidor</span>
								<input
									type="text"
									bind:value={nameDraft}
									oninput={mirrorName}
									onblur={() => void saveName()}
									onkeydown={(event) => { if (event.key === 'Enter') { event.currentTarget.blur(); } }}
								/>
							</label>
							<p class="node-note">Espelhado ao vivo no cartão do servidor.</p>
							<label class="slider">
								<span>Máximo de jogadores · {configDraft.maxPlayers}</span>
								<input type="range" min="2" max="100" step="1" bind:value={configDraft.maxPlayers} />
							</label>
							{#if platform.settings}
								<label class="field">
									<span>Porta</span>
									<input type="number" min="1024" max="65535" bind:value={configDraft.port} />
								</label>
								<label class="field">
									<span>MOTD</span>
									<input type="text" bind:value={configDraft.motd} />
								</label>
								<button type="button" class="mini accent" disabled={configSaving} onclick={() => void saveConfig()}>
									{#if configSaving}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{/if}
									Salvar configuração
								</button>
							{/if}
							<div class="divider"><span>Zona de risco</span></div>
							{#if !deleteOpen}
								<button type="button" class="mini danger" onclick={() => { deleteOpen = true; deleteName = ''; }}>
									<Trash2 class="h-3.5 w-3.5" /> Excluir servidor
								</button>
							{:else}
								<p class="node-note">Apaga a pasta, o mundo e os backups. Digite <strong>{config.name}</strong>.</p>
								<input type="text" class="del-input" bind:value={deleteName} placeholder={config.name} />
								<div class="del-row">
									<button type="button" class="mini" onclick={() => (deleteOpen = false)}>Cancelar</button>
									<button type="button" class="mini danger" disabled={deleting || deleteName !== config.name} onclick={() => void deleteServer()}>
										{#if deleting}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{:else}<Trash2 class="h-3.5 w-3.5" />{/if}
										Excluir
									</button>
								</div>
							{/if}
						</div>
					{/if}
				</div>
			{/each}
		{/if}

		{#if !config}
			<div class="empty">
				<p class="empty-formula">f(x) → servidor</p>
				<div class="empty-actions">
					<button type="button" class="create" onclick={onCreate}>
						<Plus class="h-4 w-4" /> Criar Servidor
					</button>
					<button type="button" class="create ghost" onclick={onFromPack}>
						<Download class="h-4 w-4" /> Servidor com modpack
					</button>
				</div>
				<p class="empty-note">Crie do zero ou comece de um modpack oficial compatível com servidor.</p>
			</div>
		{/if}
	</div>

	{#if servers.length > 0}
		<nav class="tabs" aria-label="Selecionar servidor">
			<button type="button" class="arrow" title="Servidor anterior" onclick={() => stepServer(-1)}><ChevronLeft class="h-4 w-4" /></button>
			{#each servers as server (server.id)}
				<button
					type="button"
					class="tab"
					class:active={server.id === platform.selectedId}
					onclick={() => selectServer(server.id)}
				>{server.name}</button>
			{/each}
			<button type="button" class="arrow" title="Próximo servidor" onclick={() => stepServer(1)}><ChevronRight class="h-4 w-4" /></button>
			<button type="button" class="tab add" title="Criar servidor" onclick={onCreate}><Plus class="h-4 w-4" /></button>
		</nav>
	{/if}

	{#if config}
		<button type="button" class="fab" title="Inspetor de estilo" onclick={() => (inspectorOpen = !inspectorOpen)}>
			<SlidersHorizontal class="h-4 w-4" /> Estilo
		</button>

		{#if inspectorOpen}
			<aside class="inspector">
				<header class="inspector-head">
					<span>Inspetor de estilo</span>
					<button type="button" class="node-x" title="Fechar" onclick={() => (inspectorOpen = false)}><X class="h-3.5 w-3.5" /></button>
				</header>
				<div class="inspector-nodes">
					{#each visibleNodes as nodeId (nodeId)}
						<button type="button" class="pill-btn" class:picked={inspectorNodeId === nodeId} onclick={() => (inspectorNodeId = nodeId)}>
							{NODE_DEFS[nodeId].label}
						</button>
					{/each}
				</div>
				<div class="inspector-body">
					<div class="inspector-row">
						<span>Forma</span>
						<div class="shape-row">
							{#each [['rounded', 'Arredondada'], ['pill', 'Pílula'], ['circle', 'Círculo'], ['custom', 'Personalizada']] as [shape, label] (shape)}
								<button type="button" class="pill-btn" class:picked={styleOf(inspectorNodeId).shape === shape} onclick={() => setShape(shape as NodeStyle['shape'])}>{label}</button>
							{/each}
						</div>
					</div>
					<label class="slider">
						<span>Raio · {styleOf(inspectorNodeId).radius}px</span>
						<input type="range" min="0" max="60" step="1" value={styleOf(inspectorNodeId).radius} oninput={(event) => patchStyle({ shape: 'custom', radius: Number(event.currentTarget.value) })} />
					</label>
					<label class="slider">
						<span>Espessura da borda · {styleOf(inspectorNodeId).stroke}px</span>
						<input type="range" min="0" max="4" step="0.5" value={styleOf(inspectorNodeId).stroke} oninput={(event) => patchStyle({ stroke: Number(event.currentTarget.value) })} />
					</label>
					<label class="slider">
						<span>Opacidade do vidro · {Math.round(styleOf(inspectorNodeId).glass * 100)}%</span>
						<input type="range" min="0.25" max="1" step="0.05" value={styleOf(inspectorNodeId).glass} oninput={(event) => patchStyle({ glass: Number(event.currentTarget.value) })} />
					</label>
					<div class="inspector-row">
						<span>Cor</span>
						<div class="theme-row">
							{#each Object.entries(THEMES) as [themeId, theme] (themeId)}
								<button
									type="button"
									class="dot"
									class:picked={styleOf(inspectorNodeId).theme === themeId}
									style="--dot: {theme.css}"
									title={theme.label}
									aria-label={`Cor ${theme.label}`}
									onclick={() => setTheme(themeId)}
								></button>
							{/each}
						</div>
					</div>
				</div>
			</aside>
		{/if}

		<div class="zoombar">
			<button type="button" title="Aproximar" onclick={() => zoomStep(1.2)}><ZoomIn class="h-4 w-4" /></button>
			<span class="zoom-value">{Math.round(zoom * 100)}%</span>
			<button type="button" title="Afastar" onclick={() => zoomStep(1 / 1.2)}><ZoomOut class="h-4 w-4" /></button>
			<button type="button" title="Ver tudo" aria-label="Ver todos os módulos" onclick={fitView}><Maximize class="h-4 w-4" /></button>
			<button type="button" title="Redefinir vista" onclick={resetView}><RotateCcw class="h-4 w-4" /></button>
		</div>
	{/if}
</div>

<style>
	.canvas {
		position: absolute;
		inset: 0;
		overflow: hidden;
		cursor: grab;
		user-select: none;
		background:
			radial-gradient(1100px 750px at 72% 18%, rgb(var(--amethyst-500) / 0.07), transparent 62%),
			radial-gradient(900px 650px at 18% 82%, rgb(var(--brand-500) / 0.06), transparent 58%),
			radial-gradient(700px 500px at 45% 45%, rgb(var(--brass-500) / 0.03), transparent 60%),
			rgb(var(--bg));
	}
	.canvas::after {
		content: '';
		position: absolute;
		inset: 0;
		pointer-events: none;
		background: linear-gradient(180deg, rgb(0 0 0 / 0.42), rgb(0 0 0 / 0.58));
	}
	.space { position: absolute; inset: 0; pointer-events: none; }
	.layer { position: absolute; inset: 0; transform-origin: 0 0; will-change: transform; }
	.layer.dragging .node { backdrop-filter: none; -webkit-backdrop-filter: none; }
	.layer.dragging .wire { filter: none; opacity: 0.95; }
	.wire-flow {
		fill: none; stroke-width: 3.5; stroke-linecap: round;
		stroke-dasharray: 6 16; opacity: 0.95;
		animation: luxmc-flow 1.6s linear infinite;
	}
	@keyframes luxmc-flow { to { stroke-dashoffset: -44; } }
	.layer.lod-schematic .wire-flow, .layer.lod-map .wire-flow { stroke-width: calc(3.5px * var(--lodpx)); }
	.node.open .body-node, .node.open .body-main { animation: luxmc-body 220ms cubic-bezier(0.16, 1, 0.3, 1); }
	@keyframes luxmc-body { from { opacity: 0; translate: 0 -4px; } to { opacity: 1; translate: 0 0; } }
	.node :is(.mini, .chip, .copy, .mode, .start-block):active { transform: scale(0.97); }
	.node :is(.mini, .chip, .copy, .mode, .start-block) { transition: transform 90ms ease, color 140ms ease, border-color 140ms ease, background-color 140ms ease; }
	@keyframes luxmc-node-in { from { opacity: 0; scale: 0.96; } to { opacity: 1; scale: 1; } }
	.layer:not(.dragging) .node { animation: luxmc-node-in 240ms cubic-bezier(0.16, 1, 0.3, 1); }
	.layer.dragging .dust { display: none; }
	.layer.dragging .node :is(.dial, .spark, .checks, .console-lines) { contain: paint; }
	.dust { position: absolute; left: -200px; top: -200px; width: 3200px; height: 2600px; pointer-events: none; }
	.star { position: absolute; border-radius: 999px; background: rgb(var(--fg)); box-shadow: 0 0 6px rgb(var(--fg) / 0.5); }
	.formula {
		position: absolute;
		font-family: Georgia, 'Times New Roman', serif;
		font-style: italic;
		color: rgb(var(--fg));
		opacity: 0.07;
		white-space: nowrap;
	}
	.guides { position: absolute; left: 0; top: 0; width: 100%; height: 100%; pointer-events: none; overflow: visible; }
	.guides line { stroke: rgb(var(--teal-400) / 0.75); stroke-width: calc(1px * var(--lodpx)); stroke-dasharray: calc(5px * var(--lodpx)) calc(5px * var(--lodpx)); }

	.node.dragging {
		z-index: 5;
		filter: drop-shadow(0 calc(18px * var(--lodpx)) calc(34px * var(--lodpx)) rgb(0 0 0 / 0.55));
		transition: filter 140ms ease;
	}
	.wires { position: absolute; left: 0; top: 0; width: 100%; height: 100%; pointer-events: none; overflow: visible; }
	.wire {
		fill: none;
		stroke-width: 3.5;
		stroke-linecap: round;
		filter: drop-shadow(0 0 4px color-mix(in srgb, currentColor 80%, transparent)) drop-shadow(0 0 13px color-mix(in srgb, currentColor 45%, transparent));
		opacity: 0.88;
		transition: d 420ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.layer.dragging .wire { transition: none; }
	.layer.zooming .wire { transition: none; }
	.layer.zooming .node { transition: none; }
	.layer.lod-schematic .wire, .layer.lod-map .wire { stroke-width: calc(3.5px * var(--lodpx)); }

	.node {
		position: absolute;
		border: var(--node-stroke, 1px) solid color-mix(in srgb, var(--node-theme) 38%, rgb(var(--border)));
		border-radius: var(--node-radius, 26px);
		background:
			linear-gradient(165deg, color-mix(in srgb, var(--node-theme) 14%, transparent) 0%, transparent 60%),
			rgb(var(--bg-elevated) / var(--node-glass, 0.85));
		backdrop-filter: blur(14px);
		-webkit-backdrop-filter: blur(14px);
		box-shadow: 0 18px 40px rgb(0 0 0 / 0.42), inset 0 1px 0 color-mix(in srgb, var(--node-theme) 18%, transparent),
			inset 0 0 0 1px rgb(255 255 255 / 0.03);
		overflow: visible;
		touch-action: none;
		transition: left 420ms cubic-bezier(0.22, 1, 0.36, 1), top 420ms cubic-bezier(0.22, 1, 0.36, 1), border-color 150ms ease, box-shadow 150ms ease;
	}
	.node:hover { border-color: color-mix(in srgb, var(--node-theme) 62%, transparent); box-shadow: 0 22px 48px rgb(0 0 0 / 0.5), 0 0 0 3px color-mix(in srgb, var(--node-theme) 16%, transparent); }
	.node.dragging { cursor: grabbing; transition: border-color 150ms ease, box-shadow 150ms ease; box-shadow: 0 30px 60px rgb(0 0 0 / 0.55), 0 0 0 3px color-mix(in srgb, var(--node-theme) 26%, transparent); }

	.port {
		--pin-r: 6px;
		position: absolute;
		width: calc(var(--pin-r) * 2);
		height: calc(var(--pin-r) * 2);
		border-radius: 999px;
		background: rgb(var(--bg-elevated));
		border: 2.5px solid color-mix(in srgb, var(--node-theme) 85%, transparent);
		box-shadow: 0 0 11px color-mix(in srgb, var(--node-theme) 65%, transparent), inset 0 0 4px color-mix(in srgb, var(--node-theme) 40%, transparent);
		z-index: 3;
	}
	.port.pin-right { right: calc(var(--pin-r) * -1); }
	.port.pin-left { left: calc(var(--pin-r) * -1); }
	.port.pin-top { top: calc(var(--pin-r) * -1); }
	.port.pin-bottom { bottom: calc(var(--pin-r) * -1); }

	.handles {
		position: absolute;
		top: -12px; right: 12px;
		display: flex; gap: 4px;
		z-index: 4;
		opacity: 0;
		transition: opacity 160ms ease;
	}
	.node:hover .handles, .handles:focus-within { opacity: 1; }
	.handles button.handle {
		display: grid; place-items: center;
		width: 22px; height: 22px;
		border: 1px solid color-mix(in srgb, var(--node-theme) 60%, rgb(var(--border-strong)));
		border-radius: 999px;
		background-color: rgb(var(--bg-elevated)) !important;
		color: var(--node-theme);
		box-shadow: 0 4px 12px rgb(0 0 0 / 0.4);
	}
	.handles button.handle:hover { background-color: color-mix(in srgb, var(--node-theme) 22%, rgb(var(--bg-elevated))) !important; }

	.node-head {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 11px 14px;
		cursor: grab;
		border-bottom: 1px solid color-mix(in srgb, var(--node-theme) 28%, rgb(var(--border-strong) / 0.4));
		border-radius: inherit;
		background: transparent;
	}
	.node-head.collapsed { border-bottom-color: transparent; }
	.grip { color: rgb(var(--fg-subtle)); flex: none; }
	.node-icon { display: grid; place-items: center; width: 26px; height: 26px; flex: none; border-radius: 9px; background: color-mix(in srgb, var(--node-theme) 18%, rgb(0 0 0 / 0.3)); color: var(--node-theme); }
	.node-label { flex: 1; font-size: 10.5px; font-weight: 800; letter-spacing: 0.14em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.node-head :global(.node-x) { display: grid; place-items: center; width: 22px; height: 22px; border: none; border-radius: 8px; background: rgb(0 0 0 / 0.28) !important; color: rgb(var(--fg-subtle)); }
	.node-head :global(.node-x:hover) { color: rgb(var(--fg)); background: color-mix(in srgb, var(--node-theme) 24%, rgb(0 0 0 / 0.28)) !important; }

	.body-main { display: flex; flex-direction: column; gap: 10px; padding: 6px 16px 16px; }
	.server-name { font-family: Georgia, 'Times New Roman', serif; font-size: 22px; font-weight: 700; color: rgb(var(--fg)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.server-sub { font-size: 12px; color: rgb(var(--fg-muted)); }
	.main-stats { display: flex; flex-wrap: wrap; gap: 7px; }
	.stat { display: inline-flex; align-items: center; gap: 6px; padding: 5px 10px; border-radius: 999px; font-size: 11.5px; font-weight: 600; color: rgb(var(--fg-muted)); background: rgb(0 0 0 / 0.3); border: 1px solid rgb(var(--border-strong) / 0.7); }
	.stat :global(svg) { color: var(--node-theme); }

	.body-node { display: flex; flex-direction: column; gap: 11px; padding: 6px 14px 14px; max-height: min(58dvh, 520px); overflow-y: auto; scrollbar-width: thin; }
	.node-note { display: flex; align-items: center; gap: 7px; font-size: 11.5px; line-height: 1.55; color: rgb(var(--fg-subtle)); }
	.node-note.bad { color: rgb(var(--danger)); }
	.node-note.busy { color: rgb(var(--fg-muted)); }

	.loader-grid { display: flex; flex-wrap: wrap; gap: 7px; }
	.node button.chip {
		display: inline-flex; align-items: center; gap: 6px;
		padding: 7px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px;
		background-color: rgb(0 0 0 / 0.28) !important; color: rgb(var(--fg-muted));
		font-size: 12px; font-weight: 700;
	}
	.node button.chip:hover:not(:disabled) { color: rgb(var(--fg)); border-color: color-mix(in srgb, var(--node-theme) 65%, transparent); }
	.node button.chip.active { color: var(--node-theme); border-color: color-mix(in srgb, var(--node-theme) 70%, transparent); background-color: color-mix(in srgb, var(--node-theme) 16%, rgb(0 0 0 / 0.28)) !important; box-shadow: 0 0 10px color-mix(in srgb, var(--node-theme) 30%, transparent); }
	.node button.chip:disabled { opacity: 0.6; cursor: not-allowed; }

	.node button.mini {
		display: inline-flex; align-items: center; justify-content: center; gap: 7px;
		align-self: flex-start; min-height: 36px; padding: 0 14px;
		border: 1px solid rgb(var(--border-strong)); border-radius: 12px;
		background-color: rgb(0 0 0 / 0.28) !important; color: rgb(var(--fg-muted));
		font-size: 12.5px; font-weight: 700;
	}
	.node button.mini:hover:not(:disabled) { color: rgb(var(--fg)); border-color: color-mix(in srgb, var(--node-theme) 65%, transparent); }
	.node button.mini.accent { color: var(--node-theme); border-color: color-mix(in srgb, var(--node-theme) 55%, transparent); background-color: color-mix(in srgb, var(--node-theme) 15%, rgb(0 0 0 / 0.28)) !important; }
	.node button.mini:disabled { opacity: 0.5; cursor: not-allowed; }

	.pack-search { display: flex; gap: 6px; }
	.pack-search input { flex: 1; min-height: 32px; padding: 5px 9px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg)); font-size: 11.5px; }
	.pack-sources { display: flex; gap: 5px; }
	.pack-sources .chip { flex: 1; font-size: 10.5px; }
	.packs.scrollable { max-height: 232px; overflow-y: auto; padding-right: 3px; }
	.packs { display: flex; flex-direction: column; gap: 8px; }
	.pack { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border: 1px solid rgb(var(--border-strong) / 0.6); border-radius: 14px; background: rgb(0 0 0 / 0.22); }
	.pack-icon { width: 34px; height: 34px; flex: none; border-radius: 10px; object-fit: cover; }
	.pack-icon.fallback { display: grid; place-items: center; background: rgb(0 0 0 / 0.3); color: var(--node-theme); }
	.pack-info { display: flex; flex-direction: column; min-width: 0; flex: 1; }
	.pack-title { font-size: 12.5px; font-weight: 700; color: rgb(var(--fg)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.pack-meta { font-size: 10.5px; color: rgb(var(--fg-subtle)); }

	.field { display: flex; flex-direction: column; gap: 5px; font-size: 11.5px; color: rgb(var(--fg-muted)); }
	.field input {
		min-height: 34px; padding: 6px 10px;
		border: 1px solid rgb(var(--border-strong)); border-radius: 11px;
		background: rgb(0 0 0 / 0.3); color: rgb(var(--fg)); font-size: 12.5px;
	}
	.field input:focus { border-color: color-mix(in srgb, var(--node-theme) 70%, transparent); outline: none; }
	.slider { display: flex; flex-direction: column; gap: 6px; font-size: 11.5px; color: rgb(var(--fg-muted)); }
	.slider input[type='range'] { width: 100%; accent-color: var(--node-theme); cursor: pointer; }
	.switch-row { display: flex; align-items: center; justify-content: space-between; gap: 10px; font-size: 12.5px; font-weight: 600; color: rgb(var(--fg-muted)); cursor: pointer; }
	.switch { position: relative; display: inline-flex; flex: none; }
	.switch input { position: absolute; inset: 0; width: 100%; height: 100%; margin: 0; opacity: 0; cursor: pointer; }
	.switch .track {
		position: relative;
		display: block;
		width: 36px; height: 20px;
		border-radius: 999px;
		background: rgb(0 0 0 / 0.42);
		border: 1px solid rgb(var(--border-strong));
		transition: background-color 180ms ease, border-color 180ms ease, box-shadow 180ms ease;
	}
	.switch .knob {
		position: absolute;
		top: 2px; left: 2px;
		width: 14px; height: 14px;
		border-radius: 999px;
		background: rgb(var(--fg-subtle));
		transition: transform 180ms cubic-bezier(0.34, 1.4, 0.64, 1), background-color 180ms ease;
	}
	.switch input:checked + .track {
		background: color-mix(in srgb, var(--node-theme) 42%, rgb(0 0 0 / 0.35));
		border-color: color-mix(in srgb, var(--node-theme) 70%, transparent);
		box-shadow: 0 0 10px color-mix(in srgb, var(--node-theme) 35%, transparent);
	}
	.switch input:checked + .track .knob { transform: translateX(16px); background: white; }
	.switch input:focus-visible + .track { outline: 2px solid var(--node-theme); outline-offset: 2px; }
	.switch .track.waiting { opacity: 0.6; }
	.divider { display: flex; align-items: center; gap: 8px; margin-top: 4px; font-size: 10px; font-weight: 800; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.divider::before, .divider::after { content: ''; flex: 1; height: 1px; background: rgb(var(--border-strong) / 0.6); }

	.dial-wrap { display: flex; align-items: center; gap: 12px; }
	.dial { width: 118px; height: 118px; flex: none; }
	.dial-arc { transition: stroke-dashoffset 320ms cubic-bezier(0.34, 1.3, 0.64, 1); filter: drop-shadow(0 0 6px color-mix(in srgb, var(--node-theme) 60%, transparent)); }
	.dial-big { fill: rgb(var(--fg)); font-size: 16px; font-weight: 700; font-family: 'Cascadia Mono', ui-monospace, monospace; }
	.dial-small { fill: rgb(var(--fg-subtle)); font-size: 9.5px; }
	.spark-wrap { display: flex; flex-direction: column; gap: 5px; flex: 1; min-width: 0; }
	.spark-label { font-size: 10.5px; color: rgb(var(--fg-subtle)); }
	.spark { width: 100%; height: 40px; border: 1px solid rgb(var(--border-strong) / 0.6); border-radius: 10px; background: rgb(0 0 0 / 0.28); }
	.net-mode { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; margin-bottom: 8px; }
	.mode {
		display: inline-flex; align-items: center; justify-content: center; gap: 6px;
		padding: 8px 6px; border-radius: 10px;
		border: 1px solid rgb(var(--border));
		background: rgb(var(--fg) / 0.04);
		color: rgb(var(--fg-muted)); font-size: 11px; font-weight: 650;
		transition: color 150ms ease, border-color 150ms ease, background-color 150ms ease;
	}
	.mode:hover:not(:disabled) { color: rgb(var(--fg)); border-color: rgb(var(--teal-400) / 0.5); }
	.mode.active { color: rgb(var(--teal-300)); border-color: rgb(var(--teal-400) / 0.6); background: rgb(var(--teal-500) / 0.14); }
	.mode:disabled { opacity: 0.5; }
	.restart-hint { margin-left: 4px; color: rgb(var(--warning)); }
	.addr-row {
		display: flex; align-items: center; gap: 8px;
		padding: 6px 8px 6px 10px; margin-bottom: 6px;
		border: 1px solid rgb(var(--border)); border-radius: 10px;
		background: rgb(var(--bg-overlay) / 0.55);
	}
	.addr-row.accent { border-color: rgb(var(--teal-400) / 0.45); background: rgb(var(--teal-500) / 0.1); }
	.addr-label { flex: none; font-size: 9.5px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.addr { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-family: 'JetBrains Mono', ui-monospace, monospace; font-size: 11px; color: rgb(var(--fg)); }
	.copy { display: grid; place-items: center; width: 24px; height: 24px; flex: none; border-radius: 7px; border: 1px solid rgb(var(--border)); color: rgb(var(--fg-muted)); background: rgb(var(--fg) / 0.05); }
	.copy:hover { color: rgb(var(--teal-300)); border-color: rgb(var(--teal-400) / 0.55); }
	.checks { display: flex; flex-direction: column; gap: 3px; margin-top: 6px; max-height: 168px; overflow-y: auto; }
	.check { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 3px 7px; border-radius: 7px; background: rgb(var(--fg) / 0.04); font-size: 10px; }
	.check.ok { color: rgb(var(--success)); }
	.check.bad { color: rgb(var(--danger)); }
	.check.warn { color: rgb(var(--warning)); }
	.check-label { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.check-status { flex: none; font-weight: 700; }
	.mini.danger { color: rgb(var(--danger)); border-color: rgb(var(--danger) / 0.4); background: rgb(var(--danger) / 0.08); }
	.del-input { width: 100%; padding: 7px 9px; border: 1px solid rgb(var(--danger) / 0.4); border-radius: 8px; background: rgb(var(--bg-overlay) / 0.7); color: rgb(var(--fg)); font-size: 11.5px; }
	.del-row { display: flex; gap: 6px; }
	.del-row .mini { flex: 1; }
	.join-steps { display: flex; flex-direction: column; gap: 5px; counter-reset: step; padding-left: 0; }
	.join-steps li { position: relative; padding-left: 20px; font-size: 10.5px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.join-steps li::before { counter-increment: step; content: counter(step); position: absolute; left: 0; top: 1px; width: 14px; height: 14px; display: grid; place-items: center; border-radius: 4px; background: rgb(var(--teal-500) / 0.2); color: rgb(var(--teal-300)); font-size: 8.5px; font-weight: 800; }
	.join-steps strong { color: rgb(var(--fg)); font-weight: 600; }
	.node-hint { font-size: 10.5px; line-height: 1.5; color: rgb(var(--fg-subtle)); }
	.node-hint.ok { color: rgb(var(--success)); }
	.node-hint.warn { color: rgb(var(--warning)); }
	.tele-row strong.good { color: rgb(var(--success)); }
	.eula-badge {
		display: inline-flex; align-items: center; gap: 9px;
		align-self: flex-start;
		padding: 9px 14px;
		border: 1px solid rgb(var(--node-theme) / 0.5);
		border-radius: 12px;
		background: color-mix(in srgb, var(--node-theme) 14%, rgb(0 0 0 / 0.28));
		color: var(--node-theme);
		font-size: 13px; font-weight: 700;
	}
	.eula-badge.ok { box-shadow: 0 0 14px color-mix(in srgb, var(--node-theme) 40%, transparent); }
	.tele { display: flex; flex-direction: column; gap: 5px; padding-top: 4px; border-top: 1px dashed rgb(var(--border-strong) / 0.6); }
	.tele-row { display: flex; align-items: center; justify-content: space-between; font-size: 11.5px; color: rgb(var(--fg-subtle)); }
	.tele-row strong { font-variant-numeric: tabular-nums; color: rgb(var(--fg-muted)); font-weight: 700; }
	.control-row { display: flex; gap: 8px; flex-wrap: wrap; }

	.console-lines {
		display: flex; flex-direction: column; gap: 2px;
		max-height: 210px; overflow-y: auto;
		padding: 8px 10px;
		border: 1px solid rgb(var(--border-strong) / 0.6);
		border-radius: 12px;
		background: rgb(0 0 0 / 0.42);
		font-family: 'Cascadia Mono', ui-monospace, monospace;
		font-size: 10px; line-height: 1.5;
	}
	.cline { color: rgb(var(--fg-muted)); overflow-wrap: anywhere; }
	.cline.error { color: rgb(var(--danger)); }
	.cline.warn { color: rgb(var(--warning)); }
	.cmd-row { display: flex; gap: 7px; }
	.cmd-row input {
		flex: 1; min-height: 36px; padding: 6px 10px;
		border: 1px solid rgb(var(--border-strong)); border-radius: 12px;
		background: rgb(0 0 0 / 0.3); color: rgb(var(--fg));
		font-family: 'Cascadia Mono', ui-monospace, monospace; font-size: 11.5px;
	}
	.cmd-row input:focus { border-color: color-mix(in srgb, var(--node-theme) 70%, transparent); outline: none; }
	.player-list { display: flex; flex-direction: column; gap: 6px; }
	.player-row { display: flex; align-items: center; gap: 9px; padding: 7px 10px; border: 1px solid rgb(var(--border-strong) / 0.6); border-radius: 12px; background: rgb(0 0 0 / 0.22); }
	.pdot { width: 8px; height: 8px; flex: none; border-radius: 999px; background: rgb(var(--fg-subtle)); }
	.pdot.on { background: rgb(var(--success)); box-shadow: 0 0 8px rgb(var(--success) / 0.7); }
	.pname { flex: 1; min-width: 0; font-size: 12.5px; font-weight: 700; color: rgb(var(--fg)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.pmeta { font-size: 10.5px; color: rgb(var(--fg-subtle)); }

	.empty {
		position: absolute;
		left: 50%; top: 50%;
		transform: translate(-50%, -50%);
		display: flex; flex-direction: column; align-items: center; gap: 16px;
		text-align: center;
	}
	.empty-actions { display: flex; flex-wrap: wrap; gap: 10px; justify-content: center; }
	.empty button.create.ghost { background: rgb(var(--bg-elevated) / 0.9) !important; border: 1px solid rgb(var(--teal-400) / 0.5); box-shadow: none; color: rgb(var(--teal-300)); }
	.empty button.create.ghost:hover { border-color: rgb(var(--teal-300)); background: rgb(var(--teal-500) / 0.14) !important; }
	.empty-formula { font-family: Georgia, serif; font-style: italic; font-size: 15px; color: rgb(var(--fg)); opacity: 0.25; }
	.empty button.create {
		display: inline-flex; align-items: center; gap: 10px;
		padding: 16px 34px;
		border: 1px solid rgb(var(--amethyst-300) / 0.65);
		border-radius: 999px;
		background-color: rgb(var(--amethyst-500) / 0.9) !important;
		color: white;
		font-family: Georgia, 'Times New Roman', serif;
		font-size: 16px; font-weight: 700; letter-spacing: 0.04em;
		box-shadow: 0 0 34px rgb(var(--amethyst-500) / 0.4);
	}
	.empty button.create:hover { background-color: rgb(var(--amethyst-400) / 0.95) !important; box-shadow: 0 0 44px rgb(var(--amethyst-400) / 0.55); }
	.empty button.create:focus-visible { outline: 2px solid rgb(var(--amethyst-300)); outline-offset: 3px; }
	.empty-note { font-size: 12.5px; color: rgb(var(--fg-subtle)); }

	.tabs {
		position: absolute;
		top: 14px; left: 50%;
		transform: translateX(-50%);
		z-index: 20;
		display: flex; align-items: center; gap: 8px;
		max-width: min(72vw, 760px);
		padding: 7px 10px;
		border: 1px solid rgb(var(--border-strong) / 0.7);
		border-radius: 999px;
		background: rgb(var(--bg-overlay) / 0.72);
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
		box-shadow: 0 14px 34px rgb(0 0 0 / 0.4);
	}
	.tabs button, .zoombar button, .fab, .inspector button.pill-btn, .inspector .dot, .inspector .node-x, .handles button.handle {
		background-color: rgb(0 0 0 / 0.26) !important;
	}
	.tabs button.tab {
		padding: 7px 16px;
		border: 1px solid rgb(var(--border-strong));
		border-radius: 999px;
		color: rgb(var(--fg-muted));
		font-size: 12.5px; font-weight: 700;
		max-width: 170px;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.tabs button.tab:hover { color: rgb(var(--fg)); border-color: rgb(var(--amethyst-400) / 0.55); }
	.tabs button.tab.active {
		color: rgb(var(--amethyst-200));
		border-color: rgb(var(--amethyst-400) / 0.85);
		background-color: rgb(var(--amethyst-500) / 0.18) !important;
		box-shadow: 0 0 16px rgb(var(--amethyst-500) / 0.45);
	}
	.tabs button.tab.add { display: grid; place-items: center; padding: 7px 10px; color: rgb(var(--brass-300)); border-color: rgb(var(--brass-500) / 0.5); }
	.tabs button.arrow { display: grid; place-items: center; width: 28px; height: 28px; border: none; border-radius: 999px; color: rgb(var(--fg-subtle)); }
	.tabs button.arrow:hover { color: rgb(var(--fg)); background-color: rgb(var(--fg) / 0.08) !important; }
	.tabs button:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 2px; }

	.fab {
		position: absolute;
		right: 16px; bottom: 64px;
		z-index: 20;
		display: inline-flex; align-items: center; gap: 8px;
		padding: 10px 16px;
		border: 1px solid rgb(var(--brass-500) / 0.55);
		border-radius: 999px;
		color: rgb(var(--brass-300));
		font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase;
		box-shadow: 0 12px 28px rgb(0 0 0 / 0.4);
	}
	.fab:hover { border-color: rgb(var(--brass-400) / 0.85); }

	.inspector {
		position: absolute;
		right: 16px; bottom: 112px;
		z-index: 21;
		width: 268px;
		display: flex; flex-direction: column;
		border: 1px solid rgb(var(--border-strong) / 0.8);
		border-radius: 18px;
		background: rgb(var(--bg-overlay) / 0.86);
		backdrop-filter: blur(14px);
		-webkit-backdrop-filter: blur(14px);
		box-shadow: 0 20px 46px rgb(0 0 0 / 0.5);
	}
	.inspector-head { display: flex; align-items: center; justify-content: space-between; padding: 11px 13px; border-bottom: 1px solid rgb(var(--border-strong) / 0.6); font-size: 11px; font-weight: 800; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.inspector-nodes { display: flex; flex-wrap: wrap; gap: 5px; padding: 10px 13px 0; }
	.inspector button.pill-btn { padding: 5px 10px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; color: rgb(var(--fg-muted)); font-size: 10.5px; font-weight: 700; }
	.inspector button.pill-btn:hover { color: rgb(var(--fg)); }
	.inspector button.pill-btn.picked { color: rgb(var(--amethyst-200)); border-color: rgb(var(--amethyst-400) / 0.7); background-color: rgb(var(--amethyst-500) / 0.16) !important; }
	.inspector-body { display: flex; flex-direction: column; gap: 11px; padding: 11px 13px 13px; }
	.inspector-row { display: flex; flex-direction: column; gap: 6px; font-size: 11px; color: rgb(var(--fg-subtle)); }
	.shape-row, .theme-row { display: flex; flex-wrap: wrap; gap: 5px; }
	.inspector .dot { width: 20px; height: 20px; border-radius: 7px; background: var(--dot) !important; border: 2px solid rgb(0 0 0 / 0.45); }
	.inspector .dot.picked { border-color: rgb(var(--fg)); box-shadow: 0 0 0 2px color-mix(in srgb, var(--dot) 50%, transparent); }

	.zoombar {
		position: absolute;
		right: 16px; bottom: 16px;
		z-index: 20;
		display: flex; align-items: center; gap: 4px;
		padding: 5px 8px;
		border: 1px solid rgb(var(--border-strong) / 0.7);
		border-radius: 999px;
		background: rgb(var(--bg-overlay) / 0.7);
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
	}
	.zoombar button { display: grid; place-items: center; width: 28px; height: 28px; border: none; border-radius: 999px; color: rgb(var(--fg-muted)); }
	.zoombar button:hover { color: rgb(var(--fg)); background-color: rgb(var(--fg) / 0.08) !important; }
	.zoom-value { min-width: 42px; text-align: center; font-size: 11px; font-variant-numeric: tabular-nums; color: rgb(var(--fg-subtle)); }

	@media (prefers-reduced-motion: reduce) {
		.wire-flow, .layer:not(.dragging) .node, .node.open .body-node, .node.open .body-main { animation: none; }
		.node :is(.mini, .chip, .copy, .mode, .start-block) { transition: none; }
		.node, .tabs button, .dial-arc { transition: none; }
	}

	.layer.lod-schematic .body-node,
	.layer.lod-map .body-node,
	.layer.lod-schematic .body-main,
	.layer.lod-map .body-main { display: none; }
	.brief { display: none; }
	.layer.lod-schematic .brief {
		display: block; padding: 4px calc(10px * var(--lodpx)) calc(8px * var(--lodpx));
		font-size: calc(10px * var(--lodpx)); line-height: 1.35;
		color: rgb(var(--fg-muted)); text-align: center;
	}
	.layer.lod-schematic .node { contain: layout paint; }
	.layer.lod-schematic .node-label { font-size: calc(11px * var(--lodpx)); }
	.layer.lod-schematic .handles { display: none; }
	.layer.lod-schematic .node-head { border-bottom-color: transparent; padding: calc(7px * var(--lodpx)) calc(10px * var(--lodpx)); }
	.layer.lod-schematic .node-head,
	.layer.lod-map .node-head { border-bottom-color: transparent; cursor: default; }
	.layer.lod-schematic .node-label,
	.layer.lod-map .node-label { font-size: calc(11.5px * var(--lodpx)); }
	.layer.lod-map .node { width: auto !important; }
	.layer.lod-map .node-head { justify-content: center; padding: calc(10px * var(--lodpx)) calc(14px * var(--lodpx)); }
	.layer.lod-map .node-label { font-size: calc(12px * var(--lodpx)); letter-spacing: 0.12em; }
	.layer.lod-map .node-icon { width: calc(24px * var(--lodpx)); height: calc(24px * var(--lodpx)); border-radius: calc(8px * var(--lodpx)); }
	.layer.lod-map .node-icon :global(svg) { width: calc(13px * var(--lodpx)); height: calc(13px * var(--lodpx)); }
	.layer.lod-map .node-head :is(.grip, .node-x), .layer.lod-map .handles { display: none; }
	.layer.lod-schematic .port, .layer.lod-map .port { --pin-r: calc(6px * var(--lodpx)); border-width: calc(2.5px * var(--lodpx)); }
</style>
