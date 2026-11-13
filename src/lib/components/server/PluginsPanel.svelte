<script lang="ts">
	import { Plug, Info, Compass } from 'lucide-svelte';
	import { softwareByKind, supportsPlugins } from '$lib/mcserver/software';
	import { platform } from '$lib/mcserver/state.svelte';

	const software = $derived(softwareByKind(platform.loader.kind));
	const pluginCapable = $derived(supportsPlugins(platform.loader.kind));
</script>

<div class="grid">
	<header class="head">
		<div>
			<h1 class="title">Plugins</h1>
			<p class="sub">Gerenciamento de plugins Bukkit/Spigot para servidores compatíveis.</p>
		</div>
	</header>

	{#if pluginCapable}
		<div class="notice" role="status">
			<Compass class="h-5 w-5 shrink-0 text-brand-300" />
			<p>Este servidor roda {software.name}. A instalação de plugins por catálogo será habilitada em uma próxima versão do LUXMC; você já pode copiar arquivos .jar diretamente para a pasta <code class="font-mono">plugins</code> pelo gerenciador de arquivos.</p>
		</div>
	{:else}
		<div class="notice" role="status">
			<Info class="h-5 w-5 shrink-0 text-warning" />
			<div>
				<p>
					Este servidor usa <strong>{software.name}</strong>, que não carrega plugins Bukkit/Spigot. Mods vão na seção <strong>Mods</strong>.
				</p>
				<p class="mt-2">
					Servidores de plugins (Paper, Purpur, Spigot) ainda não podem ser criados nesta versão do LUXMC. Quando o suporte chegar, esta seção permitirá instalar, atualizar e remover plugins com verificação de compatibilidade.
				</p>
			</div>
		</div>
	{/if}
</div>

<style>
	.grid { display: grid; gap: 18px; }
	.head { display: flex; flex-wrap: wrap; align-items: flex-end; justify-content: space-between; gap: 14px; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: 700; }
	.sub { margin-top: 4px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.notice { display: flex; gap: 12px; padding: 18px 20px; border: 1px solid rgb(var(--border-strong)); border-radius: 18px; background: rgb(var(--bg-elevated) / 0.8); font-size: 13.5px; line-height: 1.75; color: rgb(var(--fg-muted)); }
	.notice code { padding: 1px 6px; border-radius: 6px; background: rgb(var(--fg) / 0.08); font-size: 12px; }
</style>
