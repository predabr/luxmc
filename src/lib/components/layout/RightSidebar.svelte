<script lang="ts">
	import { Server, Play, Loader2, ExternalLink, ArrowRight, Newspaper, RefreshCw } from "lucide-svelte";
	import { goto } from "$app/navigation";
	import { toast } from "$lib/stores/toasts.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { launchGame } from "$lib/api";

	let connectingServer = $state<string | null>(null);

	const quickJoinServers = [
		{ id: "mush", name: "MushMC", host: "jogar.mush.com.br", port: 25565, badge: "BR", desc: "Bed Wars · PvP · Duels" },
		{ id: "hypixel", name: "Hypixel", host: "mc.hypixel.net", port: 25565, badge: "US", desc: "SkyBlock · BedWars" }
	];

	type NewsItem = {
		id: string;
		title: string;
		description: string;
		tag: string;
		tagColor: string;
	};

	const newsItems: NewsItem[] = [
		{
			id: "1",
			title: "Luxmc v1.7.1 Lançado",
			description: "Correções de memória, novos widgets na Home e design premium da sidebar.",
			tag: "Release",
			tagColor: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
		},
		{
			id: "2",
			title: "Chat P2P Universal",
			description: "Converse com amigos em tempo real sem necessidade de mods externos.",
			tag: "Feature",
			tagColor: "bg-sky-500/10 text-sky-400 border-sky-500/20"
		},
		{
			id: "3",
			title: "Otimizador de JVM Automático",
			description: "Flags de performance geradas automaticamente com base na RAM disponível.",
			tag: "Otimização",
			tagColor: "bg-purple-500/10 text-purple-400 border-purple-500/20"
		}
	];

	async function handleQuickJoin(host: string, port = 25565) {
		const inst = profiles.active || profiles.list[0];
		if (!inst) { toast("Selecione uma instância primeiro", "warning"); return; }
		if (!account.value) { toast("Faça login primeiro", "warning"); return; }
		connectingServer = host;
		try {
			await launchGame({
				versionId: inst.mcVersion,
				accountId: account.value.uuid,
				profileId: inst.id,
				enableVulkan: inst.useVulkan ?? false,
				skinUrl: activeSkinStore.current.skinUrl || account.value.skinUrl || null,
				skinVariant: activeSkinStore.current.type === "alex" ? "slim" : "classic",
				capeUrl: activeSkinStore.current.customCapeUrl || account.value.capeUrl || null,
				serverIp: host,
				serverPort: port
			});
			toast(`🎮 Conectando a ${host}...`, "success");
		} catch (e) {
			toast("Falha: " + String(e), "error");
		} finally {
			connectingServer = null;
		}
	}
</script>

<aside class="w-[300px] shrink-0 h-full flex flex-col gap-5 overflow-y-auto custom-scrollbar pr-1 pb-6 select-none">

	<div class="space-y-2.5">
		<div class="flex items-center justify-between">
			<h2 class="text-[11px] font-bold text-white/60 uppercase tracking-widest flex items-center gap-1.5">
				<Server class="w-3 h-3 text-brand-500" /> Conexão Rápida
			</h2>
			<span class="text-[9px] font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20 flex items-center gap-1">
				<span class="w-1 h-1 rounded-full bg-emerald-400 animate-pulse"></span> 1-Clique
			</span>
		</div>

		{#each quickJoinServers as srv}
			<div class="flex items-center justify-between bg-bg-elevated hover:bg-bg-subtle p-2.5 rounded-xl border border-white/5 hover:border-white/10 transition-all group">
				<div class="flex items-center gap-2 min-w-0">
					<div class="w-7 h-7 rounded-lg bg-black/40 border border-white/10 flex items-center justify-center font-black text-[10px] text-brand-500 shrink-0">
						{srv.badge}
					</div>
					<div class="min-w-0">
						<h4 class="text-xs font-semibold text-white truncate">{srv.name}</h4>
						<p class="text-[10px] text-white/35 truncate">{srv.desc}</p>
					</div>
				</div>
				<button
					type="button"
					class="px-2.5 py-1 rounded-lg bg-brand-500/10 hover:bg-brand-500 text-brand-500 hover:text-black font-bold text-[10px] transition-all flex items-center gap-1 shrink-0 cursor-pointer disabled:opacity-40"
					disabled={connectingServer === srv.host}
					onclick={() => handleQuickJoin(srv.host, srv.port)}
				>
					{#if connectingServer === srv.host}
						<Loader2 class="w-2.5 h-2.5 animate-spin" />
					{:else}
						<Play class="w-2.5 h-2.5 fill-current" />
					{/if}
					<span>Entrar</span>
				</button>
			</div>
		{/each}
	</div>

	<div class="space-y-2.5">
		<div class="flex items-center justify-between">
			<h2 class="text-[11px] font-bold text-white/60 uppercase tracking-widest flex items-center gap-1.5">
				<Newspaper class="w-3 h-3 text-brand-500" /> Notícias & Updates
			</h2>
			<a
				href="https://github.com/predabr/luxmc/releases"
				target="_blank"
				rel="noopener noreferrer"
				class="text-[10px] text-white/35 hover:text-brand-400 flex items-center gap-1 transition-colors"
			>
				Ver tudo <ExternalLink class="w-3 h-3" />
			</a>
		</div>

		<div class="flex flex-col gap-2">
			{#each newsItems as item (item.id)}
				<div class="p-2.5 rounded-xl bg-bg-elevated hover:bg-bg-subtle border border-white/5 hover:border-white/10 transition-all">
					<div class="flex items-center gap-1.5 mb-1">
						<span class="text-[9px] font-bold px-1.5 py-0.5 rounded-full border {item.tagColor}">
							{item.tag}
						</span>
					</div>
					<h4 class="text-xs font-semibold text-white leading-tight">{item.title}</h4>
					<p class="text-[10px] text-white/40 mt-0.5 leading-relaxed">{item.description}</p>
				</div>
			{/each}
		</div>
	</div>

	<div class="rounded-xl bg-bg-elevated border border-white/5 p-3 flex items-center justify-between hover:border-brand-500/30 transition-all cursor-pointer group" role="button" tabindex="0" onclick={() => goto("/friends")} onkeydown={(e) => { if (e.key === 'Enter') goto('/friends'); }}>
		<div class="flex items-center gap-2.5">
			<div class="w-7 h-7 rounded-lg bg-brand-500/10 border border-brand-500/20 flex items-center justify-center text-brand-500">
				<RefreshCw class="w-3.5 h-3.5" />
			</div>
			<div>
				<h3 class="text-xs font-semibold text-white group-hover:text-brand-400 transition-colors">Chat & Amigos</h3>
				<p class="text-[10px] text-white/35">P2P Universal · Sem Hamachi</p>
			</div>
		</div>
		<ArrowRight class="w-3.5 h-3.5 text-white/20 group-hover:text-brand-400 -rotate-45 transition-all" />
	</div>

</aside>
