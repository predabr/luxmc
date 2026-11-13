<script lang="ts">
	import { onMount } from 'svelte';
	import { Puzzle, Upload, Trash2, LoaderCircle, Info, Power, FolderArchive } from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { serverManage, type ServerContent } from '$lib/api/localServers';
	import { platform } from '$lib/mcserver/state.svelte';
	import { supportsMods, softwareByKind } from '$lib/mcserver/software';
	import { formatBytes } from '$lib/mcserver/version';

	let contents = $state<ServerContent[]>([]);
	let loading = $state(true);
	let busyName = $state('');
	let pendingPath = $state('');
	let importKind = $state<'mod' | 'modpack'>('mod');
	let applying = $state(false);

	const modLoader = $derived(supportsMods(platform.loader.kind));
	const software = $derived(softwareByKind(platform.loader.kind));
	const mods = $derived(contents.filter((item) => item.kind === 'mod' || item.kind === 'modpack'));
	const pending = $derived(mods.filter((item) => item.staged));

	async function load() {
		if (!platform.selectedId) return;
		try {
			const result = await serverManage<ServerContent[]>({ action: 'contents', id: platform.selectedId });
			contents = result;
		} catch {
			// Recarrega na próxima varredura.
		} finally {
			loading = false;
		}
	}

	async function pickFile() {
		const { open } = await import('@tauri-apps/plugin-dialog');
		try {
			const selected = await open({
				multiple: false,
				filters: importKind === 'modpack' ? [{ name: 'Modpack', extensions: ['zip', 'mrpack'] }] : [{ name: 'Mod', extensions: ['jar'] }]
			});
			if (typeof selected === 'string' && selected) pendingPath = selected;
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		}
	}

	async function importFile() {
		if (!pendingPath || busyName) return;
		busyName = 'import';
		try {
			await serverManage({ action: 'import_content', id: platform.selectedId, kind: importKind, path: pendingPath });
			toast(importKind === 'modpack' ? 'Modpack importado. Confira as dependências antes de aplicar.' : 'Mod importado. Aplique as alterações para colocá-lo no servidor.', 'success');
			pendingPath = '';
			await load();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyName = '';
		}
	}

	async function remove(name: string) {
		if (busyName) return;
		busyName = name;
		try {
			await serverManage({ action: 'remove_content', id: platform.selectedId, kind: 'mod', name });
			await load();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyName = '';
		}
	}

	async function toggle(name: string, enabled: boolean) {
		if (busyName) return;
		busyName = name;
		try {
			await serverManage({ action: 'toggle_content', id: platform.selectedId, kind: 'mod', name, enabled });
			await load();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyName = '';
		}
	}

	async function apply() {
		if (applying) return;
		applying = true;
		try {
			await serverManage({ action: 'apply_content', id: platform.selectedId });
			toast('Alterações aplicadas. Reinicie o servidor para carregar os mods.', 'success');
			await load();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			applying = false;
		}
	}

	onMount(() => void load());
</script>

<div class="grid">
	<header class="head">
		<div>
			<h1 class="title">Mods</h1>
			<p class="sub">
				{#if modLoader}
					Mods do servidor {software.name} {platform.loader.version}. Arquivos válidos são verificados antes de entrar na pasta mods.
				{:else}
					Servidores Vanilla não carregam mods. Crie um servidor com Fabric, Forge, NeoForge ou Quilt para gerenciar mods.
				{/if}
			</p>
		</div>
	</header>

	{#if !modLoader}
		<div class="notice" role="status">
			<Info class="h-5 w-5 shrink-0 text-warning" />
			<p>
				Este servidor usa <strong>{software.name}</strong>, que não carrega mods. Para usar mods, crie um servidor com um mod loader na etapa “Software do servidor”.
				Já tem um servidor de mods? Selecione-o na seção <strong>Servidores</strong>.
			</p>
		</div>
	{:else}
		<section class="card">
			<h2 class="card-title"><Upload class="h-4 w-4 text-amethyst-400" /> Adicionar mods</h2>
			<div class="import-row">
				<div class="kind-chips">
					<button type="button" class={importKind === 'mod' ? 'chip active' : 'chip'} onclick={() => (importKind = 'mod')}>Mod (.jar)</button>
					<button type="button" class={importKind === 'modpack' ? 'chip active' : 'chip'} onclick={() => (importKind = 'modpack')}>Modpack server pack (.zip)</button>
				</div>
				<button type="button" class="chip" onclick={() => void pickFile()}><FolderArchive class="h-4 w-4" /> Escolher arquivo…</button>
				{#if pendingPath}
					<span class="path font-mono" title={pendingPath}>{pendingPath.split(/[\\/]/).pop()}</span>
					<button type="button" class="primary" disabled={busyName !== ''} onclick={() => void importFile()}>
						{#if busyName === 'import'}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}
						Importar
					</button>
				{/if}
			</div>
			<p class="note">A importação valida o conteúdo do arquivo antes de aceitá-lo. Mods exclusivos do cliente são recusados automaticamente.</p>
		</section>

		{#if pending.length > 0}
			<div class="pending" role="status">
				<Info class="h-4 w-4 shrink-0 text-brand-300" />
				<span>{pending.length} {pending.length === 1 ? 'arquivo aguardando' : 'arquivos aguardando'} aplicação.</span>
				<button type="button" class="primary" disabled={applying} onclick={() => void apply()}>
					{#if applying}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}
					Aplicar alterações
				</button>
			</div>
		{/if}

		<section class="card">
			<h2 class="card-title">Instalados ({mods.length})</h2>
			{#if loading}
				<p class="loading"><LoaderCircle class="h-4 w-4 animate-spin" /> Lendo a pasta de mods…</p>
			{:else if mods.length === 0}
				<p class="empty-line">Nenhum mod instalado ainda.</p>
			{:else}
				<ul class="files">
					{#each mods as item (item.kind + item.name)}
						<li class="file">
							<div class="file-main">
								<p class="file-name">{item.name}</p>
								<p class="file-sub">{item.kind === 'modpack' ? 'Modpack' : 'Mod'} · {formatBytes(item.bytes)}{item.staged ? ' · aguardando aplicação' : ''}{item.remove ? ' · marcado para remoção' : ''}</p>
							</div>
							<div class="file-actions">
								<button type="button" class="icon-btn" title={item.name.endsWith('.disabled') ? 'Ativar' : 'Desativar'} disabled={busyName !== ''} onclick={() => void toggle(item.name, item.name.endsWith('.disabled'))}>
									<Power class="h-4 w-4" />
								</button>
								<button type="button" class="icon-btn danger" title="Remover" disabled={busyName !== ''} onclick={() => void remove(item.name)}>
									{#if busyName === item.name}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Trash2 class="h-4 w-4" />{/if}
								</button>
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		</section>
	{/if}
</div>

<style>
	.grid { display: grid; gap: 18px; }
	.head { display: flex; flex-wrap: wrap; align-items: flex-end; justify-content: space-between; gap: 14px; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: 700; }
	.sub { margin-top: 4px; font-size: 13px; line-height: 1.6; color: rgb(var(--fg-muted)); max-width: 620px; }
	.notice { display: flex; gap: 12px; padding: 16px 18px; border: 1px solid rgb(var(--warning) / 0.35); border-radius: 18px; background: rgb(var(--warning) / 0.06); font-size: 13.5px; line-height: 1.7; color: rgb(var(--fg-muted)); }
	.card { padding: 16px 18px; border: 1px solid rgb(var(--border-strong)); border-radius: 18px; background: rgb(var(--bg-elevated) / 0.8); }
	.card-title { display: flex; align-items: center; gap: 8px; margin-bottom: 12px; font-size: 12px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.import-row { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; }
	.kind-chips { display: flex; gap: 6px; }
	.chip { display: inline-flex; align-items: center; gap: 8px; padding: 8px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg-muted)); font-size: 12px; font-weight: 600; }
	.chip:hover, .chip.active { color: rgb(var(--fg)); border-color: rgb(var(--amethyst-400) / 0.6); }
	.chip:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 2px; }
	.path { max-width: 280px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11.5px; color: rgb(var(--fg-muted)); }
	.primary { display: inline-flex; align-items: center; gap: 8px; padding: 8px 16px; border: 1px solid rgb(var(--amethyst-300) / 0.5); border-radius: 9px; background: rgb(var(--amethyst-500) / 0.85); color: white; font-size: 12.5px; font-weight: 700; }
	.primary:disabled { opacity: 0.5; }
	.note { margin-top: 10px; font-size: 11.5px; line-height: 1.6; color: rgb(var(--fg-subtle)); }
	.pending { display: flex; align-items: center; gap: 10px; padding: 10px 14px; border: 1px solid rgb(var(--brand-400) / 0.35); border-radius: 16px; background: rgb(var(--brand-500) / 0.07); font-size: 13px; color: rgb(var(--fg-muted)); }
	.pending .primary { margin-left: auto; }
	.loading { display: flex; align-items: center; gap: 10px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.empty-line { font-size: 13px; color: rgb(var(--fg-subtle)); }
	.files { display: flex; flex-direction: column; gap: 6px; max-height: 420px; overflow-y: auto; padding-right: 4px; }
	.file { display: flex; align-items: center; gap: 14px; padding: 9px 12px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.35); }
	.file-main { min-width: 0; flex: 1; }
	.file-name { font-size: 13px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.file-sub { font-size: 11px; color: rgb(var(--fg-subtle)); }
	.file-actions { display: flex; gap: 6px; flex: none; }
	.icon-btn { display: grid; place-items: center; width: 32px; height: 32px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; color: rgb(var(--fg-muted)); background: rgb(var(--bg-subtle)); }
	.icon-btn:hover:not(:disabled) { color: rgb(var(--fg)); border-color: rgb(var(--brass-500) / 0.55); }
	.icon-btn.danger:hover:not(:disabled) { color: rgb(var(--danger)); border-color: rgb(var(--danger) / 0.5); }
	.icon-btn:disabled { opacity: 0.45; cursor: not-allowed; }
</style>
