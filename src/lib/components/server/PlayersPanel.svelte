<script lang="ts">
	import { onMount } from 'svelte';
	import { Users, ShieldCheck, ShieldBan, ShieldOff, UserX, ArrowUpCircle, ArrowDownCircle, LoaderCircle, Info } from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { serverManage, moderateServer, type ServerPlayer } from '$lib/api/localServers';
	import { platform } from '$lib/mcserver/state.svelte';

	interface PlayerRecord extends ServerPlayer {}

	interface Lists {
		whitelist: Array<{ name: string; uuid: string }>;
		ops: Array<{ name: string; uuid: string; level: number }>;
		bans: Array<{ name: string; reason: string; source: string }>;
	}

	let records = $state<PlayerRecord[]>([]);
	let lists = $state<Lists>({ whitelist: [], ops: [], bans: [] });
	let loading = $state(true);
	let busyName = $state('');
	let confirmBan = $state<PlayerRecord | null>(null);
	let banReason = $state('');

	async function load() {
		if (!platform.selectedId) return;
		try {
			const result = await serverManage<{ players: PlayerRecord[]; whitelist: Lists['whitelist']; ops: Lists['ops']; bans: Lists['bans'] }>({ action: 'players', id: platform.selectedId });
			records = result.players ?? [];
			lists = { whitelist: result.whitelist ?? [], ops: result.ops ?? [], bans: result.bans ?? [] };
		} catch {
			// Recarrega na próxima varredura.
		} finally {
			loading = false;
		}
	}

	async function act(command: string, name: string) {
		if (busyName) return;
		busyName = name + command;
		try {
			await moderateServer(platform.selectedId, command, name);
			toast(`Comando ${command} enviado para ${name}.`, 'success');
			await load();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyName = '';
		}
	}

	async function ban() {
		const target = confirmBan;
		if (!target) return;
		busyName = target.name + 'ban';
		try {
			await moderateServer(platform.selectedId, 'ban', target.name, banReason.trim());
			toast(`${target.name} foi banido do servidor.`, 'success');
			confirmBan = null;
			banReason = '';
			await load();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyName = '';
		}
	}

	const online = $derived(records.filter((record) => record.online));
	const offline = $derived(records.filter((record) => !record.online));
	const maxPlayers = $derived(platform.config?.maxPlayers ?? 0);

	onMount(() => {
		void load();
		const timer = setInterval(() => void load(), 5000);
		return () => clearInterval(timer);
	});
</script>

<div class="grid">
	<header class="head">
		<div>
			<h1 class="title">Jogadores</h1>
			<p class="sub">Quem está online agora, quem já visitou o mundo e quem tem acesso administrativo.</p>
		</div>
		<span class="capacity"><Users class="h-4 w-4" /> {online.length} / {maxPlayers} conectados</span>
	</header>

	{#if loading}
		<p class="loading"><LoaderCircle class="h-4 w-4 animate-spin" /> Lendo registros do servidor…</p>
	{:else}
		<div class="grid gap-5 lg:grid-cols-2">
			<section class="card">
				<h2 class="card-title">Online agora ({online.length})</h2>
				{#if online.length === 0}
					<p class="empty-line">Nenhum jogador conectado.</p>
				{:else}
					<ul class="people">
						{#each online as player (player.uuid)}
							<li class="person">
								<span class="avatar" aria-hidden="true">{player.name.slice(0, 2).toUpperCase()}</span>
								<div class="person-main">
									<p class="person-name">{player.name}</p>
									<p class="person-sub font-mono">{player.uuid}</p>
								</div>
								<div class="person-actions">
									<button type="button" class="icon-btn" title="Expulsar (kick)" disabled={busyName !== ''} onclick={() => void act('kick', player.name)}>
										<UserX class="h-4 w-4" />
									</button>
									<button type="button" class="icon-btn" title="Banir" disabled={busyName !== ''} onclick={() => { confirmBan = player; banReason = ''; }}>
										<ShieldBan class="h-4 w-4" />
									</button>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<section class="card">
				<h2 class="card-title">Operadores ({lists.ops.length})</h2>
				{#if lists.ops.length === 0}
					<p class="empty-line">Nenhum operador definido.</p>
				{:else}
					<ul class="people">
						{#each lists.ops as op (op.uuid ?? op.name)}
							<li class="person">
								<span class="avatar op" aria-hidden="true">{op.name.slice(0, 2).toUpperCase()}</span>
								<div class="person-main">
									<p class="person-name">{op.name}</p>
									<p class="person-sub">{op.level ? `Nível de permissão ${op.level}` : 'Operador'}</p>
								</div>
								<button type="button" class="icon-btn" title="Remover operador (deop)" disabled={busyName !== ''} onclick={() => void act('deop', op.name)}>
									<ArrowDownCircle class="h-4 w-4" />
								</button>
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<section class="card">
				<h2 class="card-title">Visitantes ({offline.length})</h2>
				{#if offline.length === 0}
					<p class="empty-line">Ninguém entrou neste mundo ainda.</p>
				{:else}
					<ul class="people scrollable">
						{#each offline as player (player.uuid)}
							<li class="person">
								<span class="avatar" aria-hidden="true">{player.name.slice(0, 2).toUpperCase()}</span>
								<div class="person-main">
									<p class="person-name">{player.name} <span class="visits">{player.visits} {player.visits === 1 ? 'visita' : 'visitas'}</span></p>
									<p class="person-sub font-mono">{player.uuid}</p>
								</div>
								<div class="person-actions">
									<button type="button" class="icon-btn" title="Tornar operador (op)" disabled={busyName !== ''} onclick={() => void act('op', player.name)}>
										<ArrowUpCircle class="h-4 w-4" />
									</button>
									<button type="button" class="icon-btn" title="Banir" disabled={busyName !== ''} onclick={() => { confirmBan = player; banReason = ''; }}>
										<ShieldBan class="h-4 w-4" />
									</button>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<section class="card">
				<h2 class="card-title">Banidos ({lists.bans.length})</h2>
				{#if lists.bans.length === 0}
					<p class="empty-line">Nenhum jogador banido.</p>
				{:else}
					<ul class="people scrollable">
						{#each lists.bans as ban (ban.name)}
							<li class="person">
								<span class="avatar banned" aria-hidden="true">{ban.name.slice(0, 2).toUpperCase()}</span>
								<div class="person-main">
									<p class="person-name">{ban.name}</p>
									<p class="person-sub">{ban.reason || 'Sem motivo informado'}</p>
								</div>
								<button type="button" class="icon-btn" title="Desbanir (pardon)" disabled={busyName !== ''} onclick={() => void act('pardon', ban.name)}>
									<ShieldOff class="h-4 w-4" />
								</button>
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<section class="card wide">
				<h2 class="card-title"><ShieldCheck class="h-4 w-4 text-brass-400" /> Lista de permissões (whitelist)</h2>
				{#if (platform.settings?.whitelist ?? false) === false}
					<p class="note-inline"><Info class="h-4 w-4 shrink-0" /> A lista de permissões está desativada nas configurações. Ative-a na seção Configurações para restringir a entrada do servidor.</p>
				{/if}
				{#if lists.whitelist.length === 0}
					<p class="empty-line">Nenhum jogador na lista.</p>
				{:else}
					<ul class="people scrollable">
						{#each lists.whitelist as entry (entry.uuid ?? entry.name)}
							<li class="person">
								<span class="avatar" aria-hidden="true">{entry.name.slice(0, 2).toUpperCase()}</span>
								<div class="person-main">
									<p class="person-name">{entry.name}</p>
									<p class="person-sub font-mono">{entry.uuid}</p>
								</div>
								<button type="button" class="icon-btn" title="Remover da lista" disabled={busyName !== ''} onclick={() => void act('whitelist remove', entry.name)}>
									<ShieldOff class="h-4 w-4" />
								</button>
							</li>
						{/each}
					</ul>
				{/if}
			</section>
		</div>
	{/if}
</div>

{#if confirmBan}
	<dialog class="confirm text-fg" open>
		<div class="confirm-card">
			<header class="confirm-head">
				<ShieldBan class="h-5 w-5 text-danger" />
				<h2>Banir {confirmBan.name}?</h2>
			</header>
			<p class="confirm-text">O jogador será impedido de entrar neste servidor. Você pode desfazer depois na lista de banidos.</p>
			<label class="confirm-field">
				<span>Motivo (opcional, visível ao jogador)</span>
				<input type="text" maxlength="120" bind:value={banReason} placeholder="Ex.: Comportamento impróprio" />
			</label>
			<footer class="confirm-footer">
				<button type="button" class="ghost" onclick={() => (confirmBan = null)}>Cancelar</button>
				<button type="button" class="danger" disabled={busyName !== ''} onclick={() => void ban()}>Banir jogador</button>
			</footer>
		</div>
	</dialog>
{/if}

<style>
	.grid { display: grid; gap: 18px; }
	.head { display: flex; flex-wrap: wrap; align-items: flex-end; justify-content: space-between; gap: 14px; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: 700; }
	.sub { margin-top: 4px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.capacity { display: inline-flex; align-items: center; gap: 8px; padding: 8px 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; font-size: 12.5px; font-weight: 600; color: rgb(var(--fg-muted)); }
	.loading { display: flex; align-items: center; gap: 10px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.card { padding: 16px 18px; border: 1px solid rgb(var(--border-strong)); border-radius: 18px; background: rgb(var(--bg-elevated) / 0.8); }
	.card.wide { grid-column: 1 / -1; }
	.card-title { margin-bottom: 12px; font-size: 12px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-muted)); display: flex; align-items: center; gap: 8px; }
	.people { display: flex; flex-direction: column; gap: 6px; }
	.people.scrollable { max-height: 280px; overflow-y: auto; padding-right: 4px; }
	.person { display: flex; align-items: center; gap: 12px; padding: 8px 10px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.4); }
	.avatar { display: grid; place-items: center; width: 32px; height: 32px; flex: none; border-radius: 8px; background: rgb(var(--amethyst-500) / 0.18); color: rgb(var(--amethyst-300)); font-size: 11px; font-weight: 800; }
	.avatar.op { background: rgb(var(--brass-500) / 0.2); color: rgb(var(--brass-300)); }
	.avatar.banned { background: rgb(var(--danger) / 0.18); color: rgb(var(--danger)); }
	.person-main { min-width: 0; flex: 1; }
	.person-name { display: flex; align-items: center; gap: 8px; font-size: 13.5px; font-weight: 600; }
	.visits { font-size: 10.5px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.person-sub { font-size: 10.5px; line-height: 1.5; color: rgb(var(--fg-subtle)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.person-actions { display: flex; gap: 6px; flex: none; }
	.icon-btn { display: grid; place-items: center; width: 32px; height: 32px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; color: rgb(var(--fg-muted)); background: rgb(var(--bg-subtle)); }
	.icon-btn:hover:not(:disabled) { color: rgb(var(--fg)); border-color: rgb(var(--brass-500) / 0.55); }
	.icon-btn:disabled { opacity: 0.45; cursor: not-allowed; }
	.icon-btn:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 2px; }
	.empty-line { font-size: 13px; color: rgb(var(--fg-subtle)); }
	.note-inline { display: flex; align-items: center; gap: 9px; margin-bottom: 10px; font-size: 12.5px; line-height: 1.6; color: rgb(var(--fg-muted)); }
	.confirm { margin: auto; padding: 0; border: none; background: transparent; color: rgb(var(--fg)); }
	.confirm::backdrop { background: rgb(var(--bg-overlay) / 0.8); }
	.confirm-card { width: min(460px, calc(100vw - 40px)); padding: 22px 24px; border: 1px solid rgb(var(--danger) / 0.4); border-radius: 16px; background: rgb(var(--bg-elevated)); box-shadow: 0 24px 64px rgb(0 0 0 / 0.5); }
	.confirm-head { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
	.confirm-head h2 { font-size: 17px; font-weight: 700; }
	.confirm-text { font-size: 13px; line-height: 1.7; color: rgb(var(--fg-muted)); }
	.confirm-field { display: flex; flex-direction: column; gap: 6px; margin-top: 14px; font-size: 12px; color: rgb(var(--fg-muted)); }
	.confirm-field input { min-height: 40px; padding: 8px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; background: rgb(var(--bg-overlay) / 0.7); color: rgb(var(--fg)); font-size: 13.5px; }
	.confirm-footer { display: flex; justify-content: flex-end; gap: 10px; margin-top: 18px; }
	.confirm-footer .ghost { padding: 9px 16px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; color: rgb(var(--fg-muted)); font-size: 13px; font-weight: 600; }
	.confirm-footer .danger { padding: 9px 16px; border: 1px solid rgb(var(--danger) / 0.5); border-radius: 9px; background: rgb(var(--danger) / 0.14); color: rgb(var(--danger)); font-size: 13px; font-weight: 700; }
</style>
