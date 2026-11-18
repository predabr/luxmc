<script lang="ts">
	import { onMount } from 'svelte';
	import { Archive, Plus, Undo2, Trash2, LoaderCircle, AlertTriangle, Download } from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { serverManage, type ServerBackup } from '$lib/api/localServers';
	import { platform } from '$lib/mcserver/state.svelte';
	import { formatDateTime, formatBytes } from '$lib/mcserver/version';

	interface Backup extends ServerBackup {}

	let backups = $state<Backup[]>([]);
	let loading = $state(true);
	let busyId = $state('');
	let creating = $state(false);
	let confirmRestore = $state<Backup | null>(null);
	let confirmDelete = $state<Backup | null>(null);

	async function load() {
		if (!platform.selectedId) return;
		try {
			const result = await serverManage<Backup[]>({ action: 'backups', id: platform.selectedId });
			backups = result;
		} catch {
			// Recarrega na próxima varredura.
		} finally {
			loading = false;
		}
	}

	async function create() {
		if (creating) return;
		creating = true;
		try {
			const backup = await serverManage<Backup>({ action: 'backup', id: platform.selectedId });
			toast(`Backup “${backup.id}” criado (${formatBytes(backup.bytes)}).`, 'success');
			await load();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			creating = false;
		}
	}

	async function restore() {
		const target = confirmRestore;
		if (!target || busyId) return;
		busyId = target.id;
		try {
			await serverManage({ action: 'restore_backup', id: platform.selectedId, backup: target.id });
			toast('Backup restaurado. O mundo voltou ao estado do backup.', 'success');
			confirmRestore = null;
			await load();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyId = '';
		}
	}

	async function remove() {
		const target = confirmDelete;
		if (!target || busyId) return;
		busyId = target.id;
		try {
			await serverManage({ action: 'remove_backup', id: platform.selectedId, backup: target.id });
			confirmDelete = null;
			await load();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyId = '';
		}
	}

	onMount(() => void load());
</script>

<div class="grid">
	<header class="head">
		<div>
			<h1 class="title">Backups</h1>
			<p class="sub">Cópias de segurança do servidor. Restaure com cuidado: restaurar sobrescreve o mundo atual.</p>
		</div>
		<button type="button" class="primary" disabled={creating} onclick={() => void create()}>
			{#if creating}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Plus class="h-4 w-4" />{/if}
			Criar backup
		</button>
	</header>

	{#if loading}
		<p class="loading"><LoaderCircle class="h-4 w-4 animate-spin" /> Lendo backups…</p>
	{:else if backups.length === 0}
		<div class="empty">
			<Archive class="h-8 w-8 text-fg-subtle" />
			<p>Nenhum backup ainda. Crie o primeiro antes de fazer mudanças grandes no mundo.</p>
		</div>
	{:else}
		<ul class="list">
			{#each backups as backup (backup.id)}
				<li class="row">
					<div class="row-main">
						<p class="row-name font-mono">{backup.id}</p>
						<p class="row-sub">{formatDateTime(backup.createdAt)} · {formatBytes(backup.bytes)} · {backup.files} {backup.files === 1 ? 'arquivo' : 'arquivos'}</p>
					</div>
					<div class="row-actions">
						<button type="button" class="chip" disabled={busyId !== ''} onclick={() => { confirmRestore = backup; }}>
							<Undo2 class="h-3.5 w-3.5" /> Restaurar
						</button>
						<button type="button" class="icon-btn danger" title="Excluir backup" disabled={busyId !== ''} onclick={() => { confirmDelete = backup; }}>
							<Trash2 class="h-4 w-4" />
						</button>
					</div>
				</li>
			{/each}
		</ul>
	{/if}
</div>

{#if confirmRestore}
	<dialog class="confirm text-fg" open>
		<div class="confirm-card">
			<header class="confirm-head"><AlertTriangle class="h-5 w-5 text-warning" /><h2>Restaurar backup?</h2></header>
			<p class="confirm-text">
				O mundo e os arquivos do servidor voltam ao estado de <strong class="font-mono">{confirmRestore.id}</strong> ({formatDateTime(confirmRestore.createdAt)}).
				Tudo que foi construído depois disso será perdido. Recomendamos criar um backup novo antes de restaurar.
			</p>
			<footer class="confirm-footer">
				<button type="button" class="ghost" onclick={() => (confirmRestore = null)}>Cancelar</button>
				<button type="button" class="danger" disabled={busyId !== ''} onclick={() => void restore()}>
					{#if busyId === confirmRestore.id}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}
					Restaurar e sobrescrever
				</button>
			</footer>
		</div>
	</dialog>
{/if}

{#if confirmDelete}
	<dialog class="confirm text-fg" open>
		<div class="confirm-card">
			<header class="confirm-head"><AlertTriangle class="h-5 w-5 text-warning" /><h2>Excluir backup?</h2></header>
			<p class="confirm-text">O backup <strong class="font-mono">{confirmDelete.id}</strong> ({formatBytes(confirmDelete.bytes)}) será apagado definitivamente.</p>
			<footer class="confirm-footer">
				<button type="button" class="ghost" onclick={() => (confirmDelete = null)}>Cancelar</button>
				<button type="button" class="danger" disabled={busyId !== ''} onclick={() => void remove()}>
					{#if busyId === confirmDelete.id}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}
					Excluir backup
				</button>
			</footer>
		</div>
	</dialog>
{/if}

<style>
	.grid { display: grid; gap: 18px; }
	.head { display: flex; flex-wrap: wrap; align-items: flex-end; justify-content: space-between; gap: 14px; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: 700; }
	.sub { margin-top: 4px; font-size: 13px; line-height: 1.6; color: rgb(var(--fg-muted)); max-width: 560px; }
	.primary { display: inline-flex; align-items: center; gap: 8px; padding: 10px 18px; border: 1px solid rgb(var(--amethyst-300) / 0.5); border-radius: 14px; background: rgb(var(--amethyst-500) / 0.85); color: white; font-size: 13px; font-weight: 700; }
	.primary:disabled { opacity: 0.5; cursor: not-allowed; }
	.loading { display: flex; align-items: center; gap: 10px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.empty { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 56px 0; color: rgb(var(--fg-muted)); text-align: center; font-size: 13.5px; }
	.list { display: flex; flex-direction: column; gap: 8px; }
	.row { display: flex; align-items: center; justify-content: space-between; gap: 14px; padding: 13px 16px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: rgb(var(--bg-elevated) / 0.8); }
	.row-main { min-width: 0; }
	.row-name { font-size: 13px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.row-sub { margin-top: 2px; font-size: 11.5px; color: rgb(var(--fg-subtle)); }
	.row-actions { display: flex; align-items: center; gap: 8px; flex: none; }
	.chip { display: inline-flex; align-items: center; gap: 7px; padding: 8px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg-muted)); font-size: 12px; font-weight: 600; }
	.chip:hover:not(:disabled) { color: rgb(var(--fg)); border-color: rgb(var(--brass-500) / 0.55); }
	.chip:disabled { opacity: 0.45; cursor: not-allowed; }
	.icon-btn { display: grid; place-items: center; width: 32px; height: 32px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; color: rgb(var(--fg-muted)); background: rgb(var(--bg-subtle)); }
	.icon-btn.danger:hover:not(:disabled) { color: rgb(var(--danger)); border-color: rgb(var(--danger) / 0.5); }
	.icon-btn:disabled { opacity: 0.45; cursor: not-allowed; }
	.confirm { margin: auto; padding: 0; border: none; background: transparent; color: rgb(var(--fg)); }
	.confirm::backdrop { background: rgb(var(--bg-overlay) / 0.8); }
	.confirm-card { width: min(470px, calc(100vw - 40px)); padding: 22px 24px; border: 1px solid rgb(var(--warning) / 0.4); border-radius: 16px; background: rgb(var(--bg-elevated)); box-shadow: 0 24px 64px rgb(0 0 0 / 0.5); }
	.confirm-head { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
	.confirm-head h2 { font-size: 17px; font-weight: 700; }
	.confirm-text { font-size: 13px; line-height: 1.7; color: rgb(var(--fg-muted)); }
	.confirm-footer { display: flex; justify-content: flex-end; gap: 10px; margin-top: 18px; }
	.confirm-footer .ghost { padding: 9px 16px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; color: rgb(var(--fg-muted)); font-size: 13px; font-weight: 600; }
	.confirm-footer .danger { display: inline-flex; align-items: center; gap: 8px; padding: 9px 16px; border: 1px solid rgb(var(--danger) / 0.5); border-radius: 9px; background: rgb(var(--danger) / 0.14); color: rgb(var(--danger)); font-size: 13px; font-weight: 700; }
</style>
