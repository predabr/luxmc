<script lang="ts">
	import { onMount } from "svelte";
	import { Network, Copy, LogOut, ShieldCheck, Link2, LoaderCircle, Check, Users, ArrowRight } from "lucide-svelte";
	import { button } from "$lib/components/ui/button";
	import { account } from "$lib/stores/account.svelte";
	import { virtualLanConnect, virtualLanStatus, virtualLanStop, virtualLanWorlds, virtualLanWorldPort, virtualLanPrepareWorld, type VirtualLanStatus, type VirtualLanWorlds } from "$lib/api/virtualLan";
	import { RELEASE_REVISION } from "$lib/utils/updateVersion";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { joinWorld } from "$lib/utils/directJoin";
	import { toast } from "$lib/stores/toasts.svelte";
	import { deepLinks } from "$lib/stores/deepLinks.svelte";
	import { parseDeepLink } from "$lib/utils/deepLink";
	let status = $state<VirtualLanStatus | null>(null);
	let invitation = $state("");
	let name = $state(account.value?.username ?? "Jogador");
	let busy = $state(false);
	let failure = $state("");
	let worlds = $state<VirtualLanWorlds>({ worlds: [], localWorld: null, error: null });
	let manualPort = $state("");
	let selectedProfile = $state(profiles.active?.id ?? "");
	let showPort = $state(false);
	let advanced = $state(false);
	let joining = $state(false);
	const preparing = $derived((busy && !status?.invitation) || !!status?.preparation);
	const stage = $derived(status?.preparation?.stage ?? "checking");
	const steps = ["Conferir arquivos", "Preparar componentes", "Testar inicialização", "Conectar computadores"];
	const stepIndex = $derived(stage === "checking" ? 0 : ["downloading", "verifying", "installing"].includes(stage) ? 1 : stage === "testing" ? 2 : 3);
	const stageLabel = $derived(({ checking: "Conferindo a rede", downloading: "Baixando componentes", verifying: "Verificando integridade", installing: "Preparando os arquivos", testing: "Testando a inicialização", authorizing: "Aguardando autorização do Windows" })[stage]);
	const percentage = $derived(status?.preparation?.totalBytes ? Math.min(100, Math.round(status.preparation.downloadedBytes / status.preparation.totalBytes * 100)) : null);
	let disposed = false;
	let timer: ReturnType<typeof setTimeout>;
	$effect(() => { if (deepLinks.lan) { invitation = deepLinks.lan; deepLinks.lan = null; } });

	onMount(() => {
		async function refresh() {
			try { const next = await virtualLanStatus(); if (!disposed) status = next; if (next.active) { const found = await virtualLanWorlds(); if (!disposed && found) worlds = found; } }
			catch (error) { if (!disposed) failure = String(error); }
			if (!disposed) timer = setTimeout(refresh, document.hidden ? 10000 : busy ? 750 : 3000);
		}
		void refresh();
		return () => { disposed = true; clearTimeout(timer); };
	});
	async function run(action: () => Promise<void>) {
		if (busy) return;
		busy = true; failure = "";
		try { await action(); const next = await virtualLanStatus(); if (!disposed) status = next; }
		catch (error) { if (!disposed) failure = String(error); }
		finally { if (!disposed) busy = false; }
	}
	async function copy(value: string) { try { await navigator.clipboard.writeText(value); toast("Copiado", "success"); } catch (error) { failure = String(error); } }
	async function enterWorld(id: string) {
		if (joining) return;
		joining = true; failure = "";
		try { const address = await virtualLanPrepareWorld(id); if (appState.isGameRunning) await copy(address); else await joinWorld(address, undefined, selectedProfile); }
		catch (error) { failure = String(error); }
		finally { joining = false; }
	}
	function inviteValue() {
		const value = invitation.trim();
		if (/^\d+$/.test(value)) throw new Error("Essa é uma porta do Minecraft. Peça ao amigo para clicar em Convidar amigo no Luxmc e cole o convite completo aqui.");
		if (!value.startsWith("luxmc://")) return value;
		const parsed = parseDeepLink(value);
		if (parsed.kind !== "lan") throw new Error("Peça um convite da rede LAN virtual.");
		return parsed.invitation;
	}
</script>

<section class="mx-auto max-w-5xl space-y-6" aria-label="Rede LAN virtual" aria-busy={preparing}>
	<p class="text-right font-mono text-xs text-fg-muted">Luxmc 3.5.0 · revisão {RELEASE_REVISION}</p>
	<header class="rounded-3xl border border-border bg-bg-elevated/95 shadow-soft relative overflow-hidden p-6 sm:p-8"><div class="pointer-events-none absolute -right-12 -top-16 h-64 w-64 rounded-full bg-brand-500/10 blur-3xl"></div><div class="relative space-y-4"><span class="inline-flex items-center gap-2 text-xs font-semibold uppercase tracking-widest text-brand-500"><Network class="h-4 w-4" />MULTIPLAYER</span><h1 class="text-3xl font-semibold tracking-tight text-fg sm:text-4xl">Seu mundo fica melhor com amigos.</h1><p class="max-w-2xl text-sm leading-relaxed text-fg-muted">Crie uma rede privada para jogar juntos. O Luxmc prepara os arquivos automaticamente e mantém a conexão ao trocar de aba.</p><span class="inline-flex items-center gap-2 rounded-full border border-border bg-bg/40 px-4 py-2 text-xs text-fg-muted"><ShieldCheck class="h-4 w-4 text-success" />Convite privado · 24 horas</span></div></header>
	{#if failure}<p role="alert" class="rounded-xl border border-danger/30 bg-danger/5 p-4 text-sm text-danger">{failure}</p>{/if}
	{#if status && !status.supported}
		<p class="rounded-3xl border border-border bg-bg-elevated/95 shadow-soft p-6 text-sm text-fg-muted">A rede virtual integrada está disponível no Windows de 64 bits nesta versão.</p>
	{:else if !status}
		<p role="status" class="p-6 text-sm text-fg-muted">Verificando os componentes de rede…</p>
	{:else if preparing}
		<div class="rounded-3xl border border-border bg-bg-elevated/95 shadow-soft space-y-6 p-6 sm:p-8" role="status" aria-live="polite">
			<div class="flex items-center gap-4"><div class="rounded-2xl bg-brand-500/10 p-4"><LoaderCircle class="h-7 w-7 animate-spin text-brand-500 motion-reduce:animate-none" /></div><div><h2 class="text-xl font-semibold text-fg">{stageLabel}</h2><p class="mt-1 text-sm text-fg-muted">Na primeira conexão, o Luxmc prepara tudo para você.</p></div></div>
			{#if stage === "downloading"}<div class="space-y-2"><div class="flex justify-between text-xs text-fg-muted"><span>{((status.preparation?.downloadedBytes ?? 0) / 1048576).toFixed(1)} MB recebidos</span><span>{percentage === null ? "Preparando download…" : `${percentage}%`}</span></div><div role="progressbar" aria-label="Download dos componentes de rede" aria-valuemin="0" aria-valuemax="100" aria-valuenow={percentage ?? undefined} class="h-2 overflow-hidden rounded-full bg-bg-subtle"><div class="h-full rounded-full bg-brand-500 transition-[width] duration-300 motion-reduce:transition-none" style:width={percentage === null ? "8%" : `${percentage}%`}></div></div></div>{/if}
			<ol class="grid gap-3 sm:grid-cols-4">{#each steps as step, index}<li class="flex items-center gap-2 rounded-xl border p-3 text-xs {index <= stepIndex ? 'border-brand-500/30 bg-brand-500/5 text-fg' : 'border-border text-fg-muted'}">{#if index < stepIndex}<Check class="h-4 w-4 text-success" />{:else}<span class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-bg/50 font-mono">{index + 1}</span>{/if}{step}</li>{/each}</ol>
			<p class="text-xs leading-relaxed text-fg-muted">Se o Windows pedir autorização, confirme para criar o adaptador virtual. A proteção do computador continua ativa.</p>
		</div>
	{:else if status.invitation}
		<div class="rounded-3xl border border-border bg-bg-elevated/95 shadow-soft space-y-5 p-6">
			<div class="flex flex-wrap items-center justify-between gap-4"><div><h2 class="font-semibold text-fg">{status.active ? "Sua sala está pronta" : status.error ? "A conexão precisa de atenção" : "Conectando sua sala"}</h2><p class="mt-2 text-sm text-fg-muted">{status.peers.length ? `${status.peers.length} amigo(s) na sala` : "Convide sua turma para começar"}</p></div><button class={button({ variant: "secondary" })} disabled={busy} onclick={() => run(virtualLanStop)}><LogOut class="h-4 w-4" />Sair da sala</button></div>
			{#if status.error}<p role="status" class="text-sm text-warning">{status.error}</p>{/if}
			<button class={button({ variant: "primary" })} disabled={busy || !status.active} onclick={() => copy(`luxmc://join/lan?${new URLSearchParams({ invitation: status!.invitation! })}`)}><Copy class="h-4 w-4" />Convidar amigo</button>
			<p class="text-xs text-fg-muted">Compartilhe apenas com quem você quer permitir na rede. O convite vale por 24 horas; crie outra rede para trocar o acesso.</p>
			{#if status.invitation}
				<div class="space-y-4 rounded-2xl border border-brand-500/20 bg-brand-500/5 p-5">
					<h3 class="text-lg font-semibold text-fg">Mundos para jogar</h3>
					{#if worlds.localWorld}<p class="text-sm text-success">Seu mundo está aberto e sendo compartilhado: {worlds.localWorld.motd}</p>{:else}<p class="text-sm leading-relaxed text-fg-muted">Quem vai hospedar abre o mundo no Minecraft e escolhe <strong class="text-fg">Abrir para LAN</strong>. O Luxmc encontra a porta automaticamente.</p>{/if}
					{#if worlds.error}<p role="status" class="text-sm text-warning">{worlds.error}</p>{/if}
					{#if !worlds.worlds.length}<p class="text-sm text-fg-muted">Aguardando um amigo abrir o mundo. Assim que o jogo responder, ele aparece aqui e na lista LAN do Minecraft.</p>{/if}
					{#if worlds.worlds.length && !appState.isGameRunning}<label class="block space-y-2 text-sm text-fg">Instância para jogar<select class="w-full rounded-xl border border-border bg-bg-elevated p-3" bind:value={selectedProfile}><option value="">Escolha sua instância</option>{#each profiles.list as profile (profile.id)}<option value={profile.id}>{profile.name} · {profile.mcVersion} · {profile.loader}</option>{/each}</select></label>{/if}
					{#each worlds.worlds as world (world.id)}<div class="flex flex-wrap items-center justify-between gap-4 rounded-xl border border-border bg-bg/30 p-4"><div><p class="font-medium text-fg">{world.name}</p><p class="mt-1 text-xs text-fg-muted">{world.owner} · Minecraft {world.version}{world.available === false ? " · Reconectando…" : ""}{world.latencyMs === null ? "" : ` · ${world.latencyMs} ms`}</p></div>{#if appState.isGameRunning}<button class={button({ variant: "secondary" })} onclick={() => enterWorld(world.id)}><Copy class="h-4 w-4" />Copiar acesso ao mundo</button>{:else}<button class={button({ variant: "primary" })} disabled={!selectedProfile || appState.isLaunching || joining || world.available === false} onclick={() => enterWorld(world.id)}><ArrowRight class="h-4 w-4" />Jogar com amigo</button>{/if}</div>{/each}
					{#if appState.isGameRunning && worlds.worlds.length}<p class="text-xs leading-relaxed text-fg-muted">Neste computador, abra Multijogador e escolha o mundo [Luxmc]. Se ele não aparecer, clique em Copiar acesso ao mundo aqui e cole em Conexão direta no Minecraft deste mesmo PC.</p>{/if}
					<button class={button({ variant: "ghost", size: "sm" })} aria-expanded={advanced} onclick={() => advanced = !advanced}>Ajuda de conexão</button>
					{#if advanced}<p class="text-xs text-fg-muted">Endereço da sala: <span class="select-text font-mono">{status.address}</span></p><button class={button({ variant: "ghost", size: "sm" })} onclick={() => showPort = !showPort}>{showPort ? "Fechar alternativa" : "Meu mundo não foi encontrado"}</button>
					{#if showPort}<div class="space-y-3"><p class="text-xs leading-relaxed text-fg-muted">No computador que hospeda, informe a porta exibida no chat ao abrir para LAN. O Luxmc verifica o jogo antes de compartilhar.</p><label class="block text-sm text-fg">Porta do meu mundo<input class="ml-3 w-28 rounded-lg border border-border bg-bg-elevated p-2 font-mono" bind:value={manualPort} inputmode="numeric" maxlength="5" /></label><button class={button({ variant: "secondary", size: "sm" })} disabled={!/^\d{1,5}$/.test(manualPort) || Number(manualPort) < 1 || Number(manualPort) > 65535} onclick={() => run(() => virtualLanWorldPort(Number(manualPort)))}>Verificar e compartilhar</button></div>{/if}
					{/if}<p class="text-xs text-fg-muted">Os computadores precisam usar a mesma edição, versão e os mesmos mods. Este fluxo de descoberta atende ao Minecraft Java.</p>
				</div>
			{/if}
			<div class="border-t border-border pt-5"><h3 class="flex items-center gap-2 font-medium text-fg"><Users class="h-4 w-4 text-brand-500" />Computadores na rede<span class="rounded-full bg-bg-subtle px-2 py-0.5 text-xs text-fg-muted">{status.peers.length}</span></h3>{#if !status.peers.length}<p class="mt-3 text-sm text-fg-muted">Seu amigo aparecerá aqui quando conectar com o mesmo convite.</p>{/if}<ul class="mt-3 grid gap-3 sm:grid-cols-2">{#each status.peers as peer (peer.address)}<li class="flex items-center justify-between gap-3 rounded-xl border border-border bg-bg/30 p-4"><span class="truncate text-sm text-fg">{peer.name}</span><span class="flex items-center gap-2 text-xs text-success"><span class="h-1.5 w-1.5 rounded-full bg-success"></span>Na sala</span></li>{/each}</ul></div>
			<div class="rounded-xl bg-bg-subtle p-4 text-sm leading-relaxed text-fg-muted"><p><strong class="text-fg">1.</strong> Crie a rede e compartilhe o convite do Luxmc.</p><p class="mt-2"><strong class="text-fg">2.</strong> Abra seu mundo para LAN no Minecraft.</p><p class="mt-2"><strong class="text-fg">3.</strong> Seu amigo clica em Jogar com amigo. Mantenha o Luxmc aberto durante a partida.</p></div>
		</div>
	{:else}
		<div class="rounded-3xl border border-border bg-bg-elevated/95 shadow-soft p-6 sm:p-8"><div class="mt-6 grid gap-6 lg:grid-cols-2"><div class="flex flex-col items-start rounded-2xl border border-brand-500/20 bg-brand-500/5 p-5 sm:p-6"><Network class="mb-4 h-7 w-7 text-brand-500" /><h2 class="text-lg font-semibold text-fg">O mundo é seu.</h2><p class="mb-6 mt-2 text-sm leading-relaxed text-fg-muted">O Luxmc baixa, verifica e testa os arquivos necessários. Você só precisa criar a rede e convidar seus amigos.</p><button class={button({ variant: "primary" })} disabled={!name.trim()} onclick={() => run(() => virtualLanConnect(name))}><Network class="h-4 w-4" />Criar sala<ArrowRight class="h-4 w-4" /></button></div><div class="space-y-4 rounded-2xl border border-border bg-bg/20 p-5 sm:p-6"><Link2 class="h-7 w-7 text-fg-muted" /><h2 class="text-lg font-semibold text-fg">Sua turma te espera.</h2><label class="block space-y-2 text-sm text-fg">Convite de um amigo<textarea class="min-h-24 w-full rounded-xl border border-border bg-bg-elevated p-3 font-mono text-xs" bind:value={invitation} maxlength="1024" placeholder="Cole o convite da rede LAN"></textarea></label><button class={button({ variant: "secondary" })} disabled={!name.trim() || !invitation.trim()} onclick={() => run(() => virtualLanConnect(name, inviteValue()))}><Link2 class="h-4 w-4" />Entrar na sala</button></div></div></div>
	{/if}
</section>
