<script lang="ts">
	import { Users, Copy } from "lucide-svelte";
	import type { ServerEntry } from "$lib/data/servers";
	import ServerPingIndicator from "./ServerPingIndicator.svelte";

	type Props = {
		server: ServerEntry;
		liveData?: { online: number; max: number; ping: number };
		onCopyIp: (ip: string) => void;
	};

	let { server, liveData, onCopyIp }: Props = $props();
</script>

<div
	class="flex items-center justify-between bg-[#18191c] hover:bg-[#1f2025] border border-white/5 hover:border-brand-500/40 p-4 rounded-3xl transition-all cursor-pointer group shadow-sm"
	onclick={() => onCopyIp(server.address)}
	role="button"
	tabindex="0"
	onkeydown={(e) => { if (e.key === 'Enter') onCopyIp(server.address); }}
	title="Clique para copiar o IP para a área de transferência"
>
	<div class="flex items-center gap-4 min-w-0">
		<span class="text-sm font-black text-white/30 group-hover:text-brand-500 w-6 text-center">
			#{server.rank}
		</span>

		<div class="h-13 w-13 rounded-2xl bg-black/50 border border-white/10 flex items-center justify-center shrink-0 p-2 group-hover:scale-105 transition-transform shadow-md">
			<img src={server.logo} alt={server.name} class="w-full h-full object-contain rounded-lg" loading="lazy" decoding="async" />
		</div>

		<div class="min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-extrabold text-white text-sm group-hover:text-brand-500 transition-colors truncate">
					{server.name}
				</h3>
				<button
					type="button"
					class="p-1 rounded-lg hover:bg-white/10 text-white/30 hover:text-white transition-all cursor-pointer"
					onclick={(e) => { e.stopPropagation(); onCopyIp(server.address); }}
					title="Copiar IP"
				>
					<Copy class="w-3 h-3" />
				</button>
			</div>

			<div class="flex items-center gap-2 text-xs text-white/40 mt-1">
				<span class="font-mono text-emerald-400 font-bold flex items-center gap-1">
					<Users class="w-3 h-3" />
					{liveData?.online ?? server.online} online
				</span>
				<span>•</span>
				<span class="font-mono text-white/60">{server.address}</span>
				<span>•</span>
				<span>{server.version}</span>
			</div>

			<div class="flex gap-1.5 mt-2.5">
				{#each server.badges as badge}
					<span class="bg-white/5 text-white/70 text-[9px] font-bold px-2.5 py-0.5 rounded-lg border border-white/5 uppercase tracking-wide">
						{badge}
					</span>
				{/each}
			</div>
		</div>
	</div>

	<div class="flex items-center gap-4 shrink-0 pl-4">
		<div class="hidden md:flex h-14 w-64 rounded-2xl bg-gradient-to-r {server.bannerColor} border border-white/10 items-center justify-center p-3 text-center shadow-inner relative overflow-hidden group-hover:brightness-110 transition-all">
			<div class="absolute inset-0 bg-black/25 backdrop-blur-[1px]"></div>
			<span class="relative z-10 font-black text-white text-[11px] tracking-wider uppercase drop-shadow-md truncate px-2">
				{server.bannerText}
			</span>
		</div>

		<div class="text-right">
			<ServerPingIndicator ping={liveData?.ping ?? 24} />
			<span class="text-white/30 text-[10px] block mt-0.5 group-hover:text-brand-500 transition-colors font-bold">
				Copiar IP
			</span>
		</div>
	</div>
</div>
