<script lang="ts">
	import { Info, ExternalLink, HeartHandshake } from 'lucide-svelte';
	import { APP_VERSION } from '$lib/version';

	const year = new Date().getFullYear();
</script>

<div class="grid">
	<header class="head">
		<img src="/branding/luxmc-emblem-128.png" alt="LUXMC" class="emblem" />
		<div>
			<h1 class="title">LUXMC</h1>
			<p class="sub">Seu servidor Minecraft, do seu jeito. · Versão {APP_VERSION}</p>
		</div>
	</header>

	<div class="grid gap-5 lg:grid-cols-2">
		<section class="card">
			<h2 class="card-title"><Info class="h-4 w-4 text-amethyst-400" /> Sobre o gerenciador de servidores</h2>
			<p class="body-text">
				O LUXMC instala e gerencia servidores dedicados de Minecraft Java usando apenas instaladores e arquivos oficiais:
				o servidor Vanilla vem da Mojang, e Fabric, Forge, NeoForge e Quilt são instalados pelos instaladores oficiais de cada projeto.
				Nenhum passo da instalação é simulado — downloads, EULA e inicialização são reais.
			</p>
			<ul class="bullets">
				<li>Os dados de cada servidor ficam em uma pasta própria, dentro da área de dados do LUXMC.</li>
				<li>A EULA do Minecraft só é aceita quando você marca a caixa na criação do servidor.</li>
				<li>A autenticação online (online-mode) permanece ativa para proteger o seu mundo.</li>
				<li>Backups, console, jogadores e configurações são lidos diretamente dos arquivos do servidor.</li>
			</ul>
		</section>

		<section class="card">
			<h2 class="card-title"><ExternalLink class="h-4 w-4 text-brass-400" /> Links oficiais</h2>
			<ul class="links">
				<li><a href="https://www.minecraft.net/en-us/eula" target="_blank" rel="noreferrer">EULA do Minecraft <ExternalLink class="h-3 w-3" /></a></li>
				<li><a href="https://www.minecraft.net/en-us/download/server" target="_blank" rel="noreferrer">Download oficial do servidor <ExternalLink class="h-3 w-3" /></a></li>
				<li><a href="https://meta.fabricmc.net/" target="_blank" rel="noreferrer">Fabric Meta <ExternalLink class="h-3 w-3" /></a></li>
				<li><a href="https://meta.quiltmc.org/" target="_blank" rel="noreferrer">Quilt Meta <ExternalLink class="h-3 w-3" /></a></li>
				<li><a href="https://docs.neoforged.net/user/docs/server/" target="_blank" rel="noreferrer">Documentação do NeoForge <ExternalLink class="h-3 w-3" /></a></li>
			</ul>
		</section>

		<section class="card wide">
			<h2 class="card-title"><HeartHandshake class="h-4 w-4 text-brass-400" /> Avisos</h2>
			<p class="body-text">
				Minecraft é um produto da Mojang Studios / Microsoft. O LUXMC é uma ferramenta independente e não é afiliada à Mojang ou à Microsoft.
				Os nomes Fabric, Forge, NeoForge, Quilt, Paper, Purpur e Spigot pertencem aos seus respectivos projetos comunitários.
			</p>
			<p class="body-text mt-2">© {year} · LUXMC</p>
		</section>
	</div>
</div>

<style>
	.grid { display: grid; gap: 18px; }
	.head { display: flex; align-items: center; gap: 16px; }
	.emblem { width: 64px; height: 64px; object-fit: contain; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 28px; font-weight: 700; letter-spacing: 0.14em; color: rgb(var(--brass-300)); }
	.sub { margin-top: 2px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.card { padding: 16px 18px; border: 1px solid rgb(var(--border-strong)); border-radius: 18px; background: rgb(var(--bg-elevated) / 0.8); }
	.card.wide { grid-column: 1 / -1; }
	.card-title { display: flex; align-items: center; gap: 8px; margin-bottom: 12px; font-size: 12px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.body-text { font-size: 13px; line-height: 1.75; color: rgb(var(--fg-muted)); }
	.bullets { display: flex; flex-direction: column; gap: 6px; margin-top: 10px; }
	.bullets li { font-size: 12.5px; line-height: 1.65; color: rgb(var(--fg-muted)); list-style: none; padding-left: 16px; position: relative; }
	.bullets li::before { content: ''; position: absolute; left: 2px; top: 8px; width: 5px; height: 5px; border-radius: 999px; background: rgb(var(--brass-400)); }
	.links { display: flex; flex-direction: column; gap: 8px; }
	.links a { display: inline-flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 600; color: rgb(var(--brand-300)); text-decoration: none; }
	.links a:hover { text-decoration: underline; text-underline-offset: 3px; }
</style>
