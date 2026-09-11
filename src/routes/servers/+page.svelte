<script lang="ts">
	import { fade } from "svelte/transition";
	import { onMount, onDestroy } from "svelte";
	import { Search, RefreshCw } from "lucide-svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { serverPing } from "$lib/api";
	import { servers, type ServerEntry } from "$lib/data/servers";
	import ServerCard from "$lib/components/servers/ServerCard.svelte";
	import AddServerModal from "$lib/components/servers/AddServerModal.svelte";

	let searchQuery = $state("");
	let selectedTag = $state<string | null>(null);
	let selectedRegion = $state<string>("all");
	let liveServerData = $state<Record<string, { online: number; max: number; ping: number }>>({});
	let pingInterval: ReturnType<typeof setInterval> | null = null;
	let showAddServerModal = $state(false);

	const tags = $derived.by(() => {
		const tagMap: Record<string, number> = {};
		for (const srv of servers) {
			for (const b of srv.badges) {
				tagMap[b] = (tagMap[b] || 0) + 1;
			}
		}
		return Object.entries(tagMap)
			.map(([name, count]) => ({ name, count }))
			.sort((a, b) => b.count - a.count)
			.slice(0, 16);
	});

	const filteredServers = $derived(
		servers.filter((srv) => {
			const matchesSearch =
				srv.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
				srv.address.toLowerCase().includes(searchQuery.toLowerCase()) ||
				srv.badges.some(b => b.toLowerCase().includes(searchQuery.toLowerCase()));

			const matchesTag = !selectedTag || srv.badges.includes(selectedTag);

			const matchesRegion =
				selectedRegion === "all" ? true :
				selectedRegion === "br" ? srv.badges.includes("Brasil") :
				!srv.badges.includes("Brasil");

			return matchesSearch && matchesTag && matchesRegion;
		})
	);

	function copyServerIp(ip: string) {
		navigator.clipboard.writeText(ip);
		toast(`IP ${ip} copiado para a área de transferência!`, "success");
	}

	let isRefreshingPings = $state(false);

	async function refreshPings() {
		if (isRefreshingPings) return;
		isRefreshingPings = true;
		const targets = filteredServers.slice(0, 15);
		await Promise.allSettled(
			targets.map(async (s) => {
				try {
					const data = await serverPing(s.address, 25565);
					if (data) {
						liveServerData[s.id] = {
							online: data.playersOnline,
							max: data.playersMax || 1000,
							ping: data.latencyMs ?? 32,
						};
					}
				} catch {
					// Keep fallback
				}
			})
		);
		isRefreshingPings = false;
		toast("Pings dos servidores atualizados com sucesso!", "success");
	}

	onMount(() => {
		void refreshPings();
		pingInterval = setInterval(refreshPings, 45000);
	});

	onDestroy(() => {
		if (pingInterval) clearInterval(pingInterval);
	});
</script>

<div class="flex gap-8 h-full w-full select-none" in:fade={{ duration: 300 }}>

	<!-- Main Server Browser Content -->
	<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-6">

		<!-- Header -->
		<div class="flex items-center justify-between mt-1">
			<div>
				<div class="flex items-center gap-3">
					<h1 class="text-3xl font-extrabold text-white tracking-tight">Servidores de Minecraft</h1>
					<span class="bg-brand-500/20 text-brand-500 text-[10px] font-black px-2.5 py-1 rounded-lg uppercase tracking-wide border border-brand-500/30">
						{filteredServers.length} Servidores Ativos
					</span>
				</div>
				<p class="text-white/50 text-xs mt-0.5">Diretório completo de redes brasileiras e mundiais com ping em tempo real e cópia de IP com 1 clique</p>
			</div>

			<div class="flex items-center gap-2">
				<Button
					variant="outline"
					class="border-white/10 bg-[#1e1f23] hover:bg-white/10 text-white gap-2 rounded-xl text-xs px-4 py-2 cursor-pointer"
					disabled={isRefreshingPings}
					onclick={refreshPings}
				>
					<RefreshCw class="w-3.5 h-3.5 {isRefreshingPings ? 'animate-spin text-brand-500' : ''}" />
					{isRefreshingPings ? "Atualizando..." : "Atualizar Pings"}
				</Button>
				<Button
					variant="outline"
					class="border-brand-500/30 bg-brand-500/10 hover:bg-brand-500/20 text-brand-500 gap-2 rounded-xl text-xs px-4 py-2 cursor-pointer"
					onclick={() => showAddServerModal = true}
				>
					+ Adicionar Servidor
				</Button>
			</div>
		</div>

		<!-- Search & Region Filters -->
		<div class="flex items-center gap-3">
			<div class="relative flex-1">
				<Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40" />
				<input
					type="text"
					placeholder="Buscar servidor por nome, IP ou modo de jogo (ex: Mush, Hypixel, Bedwars, Pixelmon)..."
					bind:value={searchQuery}
					class="w-full bg-[#18191c] border border-white/10 rounded-2xl pl-10 pr-4 py-2.5 text-xs text-white placeholder-white/40 focus:outline-none focus:border-brand-500 transition-all shadow-inner"
				/>
			</div>

			<!-- Region Quick Selector -->
			<div class="flex items-center bg-[#18191c] p-1 rounded-full border border-white/10">
				<button
					type="button"
					class="px-4 py-1.5 rounded-full text-xs font-bold transition-all cursor-pointer {selectedRegion === 'all' ? 'text-black font-black' : 'text-white/50 hover:text-white'}"
					style={selectedRegion === 'all' ? 'background-color: var(--accent-color, #e2b86b);' : ''}
					onclick={() => selectedRegion = 'all'}
				>
					Todos ({servers.length})
				</button>
				<button
					type="button"
					class="px-4 py-1.5 rounded-full text-xs font-bold transition-all cursor-pointer {selectedRegion === 'br' ? 'text-black font-black' : 'text-white/50 hover:text-white'}"
					style={selectedRegion === 'br' ? 'background-color: var(--accent-color, #e2b86b);' : ''}
					onclick={() => selectedRegion = 'br'}
				>
					🇧🇷 Brasil (5)
				</button>
				<button
					type="button"
					class="px-4 py-1.5 rounded-full text-xs font-bold transition-all cursor-pointer {selectedRegion === 'intl' ? 'text-black font-black' : 'text-white/50 hover:text-white'}"
					style={selectedRegion === 'intl' ? 'background-color: var(--accent-color, #e2b86b);' : ''}
					onclick={() => selectedRegion = 'intl'}
				>
					🌎 Mundial (20)
				</button>
			</div>
		</div>

		<!-- Tags Filter Chips -->
		<div class="flex flex-wrap gap-1.5 items-center">
			<span class="text-[11px] font-bold text-white/40 uppercase mr-1">Filtrar por:</span>
			<button
				type="button"
				class="px-3.5 py-1 rounded-full text-[11px] font-bold transition-all cursor-pointer {!selectedTag ? 'text-black font-black' : 'bg-[#18191c] text-white/50 hover:text-white border border-white/5'}"
				style={!selectedTag ? 'background-color: var(--accent-color, #e2b86b);' : ''}
				onclick={() => selectedTag = null}
			>
				Todos
			</button>
			{#each tags as tag}
				<button
					type="button"
					class="px-3.5 py-1 rounded-full text-[11px] font-bold transition-all cursor-pointer {selectedTag === tag.name ? 'text-black font-black' : 'bg-[#18191c] text-white/50 hover:text-white border border-white/5'}"
					style={selectedTag === tag.name ? 'background-color: var(--accent-color, #e2b86b);' : ''}
					onclick={() => selectedTag = selectedTag === tag.name ? null : tag.name}
				>
					{tag.name} <span class="opacity-50 text-[10px]">({tag.count})</span>
				</button>
			{/each}
		</div>

		<!-- Server List Cards -->
		<div class="flex flex-col gap-3 pb-6">
			{#each filteredServers as srv (srv.id + srv.rank)}
				<ServerCard
					server={srv}
					liveData={liveServerData[srv.id]}
					onCopyIp={copyServerIp}
				/>
			{/each}
		</div>

	</div>

</div>

<AddServerModal open={showAddServerModal} onclose={() => showAddServerModal = false} />
