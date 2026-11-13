<script lang="ts">
	import { onMount } from 'svelte';
	import { LoaderCircle, RotateCcw, Save, AlertTriangle, ShieldAlert } from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { saveServerSettings, serverSettings, type ServerSettings } from '$lib/api/localServers';
	import { platform } from '$lib/mcserver/state.svelte';

	let draft = $state<ServerSettings | null>(null);
	let dirty = $state(false);
	let saving = $state(false);

	function patch(values: Partial<ServerSettings>) {
		if (!draft) return;
		draft = { ...draft, ...values };
		dirty = true;
	}

	async function load() {
		if (!platform.selectedId) return;
		try {
			const raw = await serverSettings(platform.selectedId);
			draft = {
				networkCompressionThreshold: 256,
				entityBroadcastRangePercentage: 100,
				maxTickTime: 60000,
				opPermissionLevel: 4,
				...raw
			};
			dirty = false;
		} catch {
			// Recarrega na próxima varredura.
		}
	}

	async function save() {
		if (!draft || !platform.selectedId) return;
		saving = true;
		try {
			await saveServerSettings(platform.selectedId, draft);
			platform.settings = draft;
			dirty = false;
			toast('Configurações salvas em server.properties.', 'success');
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			saving = false;
		}
	}

	async function discard() {
		await load();
		toast('Alterações descartadas.', 'info');
	}

	onMount(() => void load());
	$effect(() => {
		platform.selectedId;
		void load();
	});
</script>

<div class="grid">
	<header class="head">
		<div>
			<h1 class="title">Configurações</h1>
			<p class="sub">Propriedades do servidor (server.properties) suportadas por esta versão, com validação e valores seguros.</p>
		</div>
		{#if dirty}<span class="dirty">Alterações não salvas</span>{/if}
	</header>

	{#if !draft}
		<p class="loading"><LoaderCircle class="h-4 w-4 animate-spin" /> Carregando configurações…</p>
	{:else}
		<div class="grid gap-5 lg:grid-cols-2">
			<section class="card">
				<h2 class="card-title">Identidade</h2>
				<label class="field">
					<span>Nome do servidor</span>
					<input type="text" maxlength="64" value={draft.motd} oninput={(e) => patch({ motd: e.currentTarget.value })} />
					<small class="hint">Mensagem exibida na lista de servidores do Minecraft.</small>
				</label>
				<div class="grid grid-cols-2 gap-3 mt-3">
					<label class="field">
						<span>Porta</span>
						<input type="number" min="1024" max="65535" value={draft.port || 25565} oninput={(e) => patch({ port: Number(e.currentTarget.value) })} />
						<small class="hint">Porta 0 significa escolha automática durante a inicialização.</small>
					</label>
					<label class="field">
						<span>Tempo limite de ociosidade (min)</span>
						<input type="number" min="0" max="240" value={draft.idleTimeout} oninput={(e) => patch({ idleTimeout: Number(e.currentTarget.value) })} />
						<small class="hint">0 desativa a expulsão de jogadores parados.</small>
					</label>
				</div>
			</section>

			<section class="card">
				<h2 class="card-title">Acesso e segurança</h2>
				<div class="toggles">
					<label class="toggle-row">
						<input type="checkbox" checked={draft.whitelist} onchange={(e) => patch({ whitelist: e.currentTarget.checked })} />
						<span><strong>Lista de permissões ativa</strong><small>Somente jogadores na whitelist conseguem entrar.</small></span>
					</label>
					<label class="toggle-row">
						<input type="checkbox" checked={draft.onlineMode !== false} onchange={(e) => patch({ onlineMode: e.currentTarget.checked })} />
						<span><strong>Exigir contas originais</strong><small>Desmarque para o modo offline, que aceita contas pirata.</small></span>
					</label>
					<label class="toggle-row">
						<input type="checkbox" checked={draft.economyMode} onchange={(e) => patch({ economyMode: e.currentTarget.checked })} />
						<span><strong>Perfil econômico</strong><small>Rotina de salvamento e otimizações leves enquanto o servidor roda.</small></span>
					</label>
				</div>
				<div class="warn-box">
					<ShieldAlert class="h-4 w-4 shrink-0 text-warning" />
					<p>
						Com contas originais o servidor valida cada jogador nos servidores da Mojang. No modo offline qualquer pessoa com o endereço entra,
						mas nomes podem ser falsificados. Alterações em online-mode exigem reiniciar o servidor.
					</p>
				</div>
			</section>

			<section class="card">
				<h2 class="card-title">Rede e desempenho</h2>
				<div class="grid grid-cols-2 gap-3">
					<label class="field">
						<span>Compressão de rede</span>
						<input type="number" min="-1" max="512" value={draft.networkCompressionThreshold} oninput={(e) => patch({ networkCompressionThreshold: Number(e.currentTarget.value) })} />
						<small class="hint">−1 desativa; 256 é o padrão do Minecraft.</small>
					</label>
					<label class="field">
						<span>Alcance de broadcast (%)</span>
						<input type="number" min="10" max="1000" value={draft.entityBroadcastRangePercentage} oninput={(e) => patch({ entityBroadcastRangePercentage: Number(e.currentTarget.value) })} />
					</label>
					<label class="field">
						<span>Tempo máximo por tick (ms)</span>
						<input type="number" min="1000" max="3600000" value={draft.maxTickTime} oninput={(e) => patch({ maxTickTime: Number(e.currentTarget.value) })} />
						<small class="hint">60000 é o padrão; valores altos evitam watchdogs agressivos.</small>
					</label>
					<label class="field">
						<span>Nível de permissão padrão (op)</span>
						<input type="number" min="0" max="4" value={draft.opPermissionLevel} oninput={(e) => patch({ opPermissionLevel: Number(e.currentTarget.value) })} />
					</label>
				</div>
				<div class="toggles mt-3">
					<label class="toggle-row">
						<input type="checkbox" checked={draft.hideOnlinePlayers} onchange={(e) => patch({ hideOnlinePlayers: e.currentTarget.checked })} />
						<span><strong>Ocultar contagem de jogadores</strong><small>O ping do servidor não revela quantos estão online.</small></span>
					</label>
					<label class="toggle-row">
						<input type="checkbox" checked={draft.listenOnNetwork} onchange={(e) => patch({ listenOnNetwork: e.currentTarget.checked })} />
						<span><strong>Abrir na rede local</strong><small>Permite conexões de outros computadores da sua rede.</small></span>
					</label>
				</div>
			</section>

			<section class="card">
				<h2 class="card-title">Pacote de recursos</h2>
				<label class="field">
					<span>URL do pacote</span>
					<input type="text" placeholder="https://…" value={draft.resourcePackUrl} oninput={(e) => patch({ resourcePackUrl: e.currentTarget.value })} />
				</label>
				<label class="field mt-3">
					<span>SHA-1 do arquivo (opcional)</span>
					<input type="text" class="font-mono" value={draft.resourcePackSha1} oninput={(e) => patch({ resourcePackSha1: e.currentTarget.value })} />
					<small class="hint">Com o hash, o cliente valida a integridade do download.</small>
				</label>
				<label class="toggle-row mt-3">
					<input type="checkbox" checked={draft.requireResourcePack} onchange={(e) => patch({ requireResourcePack: e.currentTarget.checked })} />
					<span><strong>Exigir o pacote</strong><small>Jogadores que recusarem o pacote são desconectados.</small></span>
				</label>
			</section>
		</div>

		<div class="save-bar">
			{#if dirty}
				<p class="restart-note"><AlertTriangle class="h-3.5 w-3.5 text-warning" /> A maioria das propriedades só vale na próxima inicialização do servidor.</p>
			{/if}
			<div class="save-actions">
				<button type="button" class="ghost" disabled={!dirty || saving} onclick={() => void discard()}><RotateCcw class="h-4 w-4" /> Descartar</button>
				<button type="button" class="primary" disabled={!dirty || saving} onclick={() => void save()}>
					{#if saving}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Save class="h-4 w-4" />{/if}
					Salvar
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.grid { display: grid; gap: 18px; }
	.head { display: flex; flex-wrap: wrap; align-items: flex-end; justify-content: space-between; gap: 14px; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: 700; }
	.sub { margin-top: 4px; font-size: 13px; line-height: 1.6; color: rgb(var(--fg-muted)); max-width: 640px; }
	.dirty { padding: 6px 12px; border: 1px solid rgb(var(--warning) / 0.4); border-radius: 999px; background: rgb(var(--warning) / 0.1); font-size: 11.5px; font-weight: 700; color: rgb(var(--warning)); }
	.loading { display: flex; align-items: center; gap: 10px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.card { padding: 16px 18px; border: 1px solid rgb(var(--border-strong)); border-radius: 18px; background: rgb(var(--bg-elevated) / 0.8); }
	.card-title { margin-bottom: 12px; font-size: 12px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.field { display: flex; flex-direction: column; gap: 6px; }
	.field > span { font-size: 10.5px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.field input { min-height: 40px; padding: 8px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; background: rgb(var(--bg-overlay) / 0.7); color: rgb(var(--fg)); font-size: 13.5px; }
	.field input:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 1px; }
	.hint { font-size: 11px; line-height: 1.5; color: rgb(var(--fg-subtle)); }
	.toggles { display: flex; flex-direction: column; gap: 6px; }
	.toggle-row { display: flex; align-items: flex-start; gap: 10px; padding: 8px 10px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.4); cursor: pointer; }
	.toggle-row input { margin-top: 3px; accent-color: rgb(var(--amethyst-400)); }
	.toggle-row strong { font-size: 13px; font-weight: 600; }
	.toggle-row small { display: block; font-size: 11px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.warn-box { display: flex; gap: 10px; margin-top: 12px; padding: 10px 12px; border: 1px solid rgb(var(--warning) / 0.3); border-radius: 14px; background: rgb(var(--warning) / 0.06); }
	.warn-box p { font-size: 12px; line-height: 1.65; color: rgb(var(--fg-muted)); }
	.save-bar { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 12px; position: sticky; bottom: 0; padding: 12px 0 4px; background: rgb(var(--bg) / 0.85); backdrop-filter: blur(8px); }
	.restart-note { display: flex; align-items: center; gap: 8px; font-size: 12.5px; color: rgb(var(--fg-muted)); }
	.save-actions { display: flex; gap: 10px; }
	.ghost { display: inline-flex; align-items: center; gap: 8px; padding: 9px 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; color: rgb(var(--fg-muted)); font-size: 13px; font-weight: 600; background: transparent; }
	.ghost:disabled { opacity: 0.45; }
	.primary { display: inline-flex; align-items: center; gap: 8px; padding: 9px 18px; border: 1px solid rgb(var(--amethyst-300) / 0.5); border-radius: 9px; background: rgb(var(--amethyst-500) / 0.85); color: white; font-size: 13px; font-weight: 700; }
	.primary:disabled { opacity: 0.45; }
</style>
