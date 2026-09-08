<script lang="ts">
	import { Newspaper, RefreshCw, Sparkles, ArrowRight, Zap, ShieldCheck, Calendar } from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const luxmcNews = [
		{
			id: "news-0",
			title: "Minecraft 1.21.4: O Jardim Pálido e o Creaking",
			date: "Hoje às 10:15",
			tag: "OFICIAL",
			desc: "Explore o bioma sombrio Pale Garden, a nova madeira pálida e enfrente o Creaking com suporte imediato no Luxmc.",
			img: "/vanilla_banner.png"
		},
		{
			id: "news-1",
			title: "Luxmc v0.2.0: Motor Wayland & Vulkan Zero-Lag",
			date: "Hoje às 09:30",
			tag: "LUXMC",
			desc: "Desempenho gráfico acelerado no Linux, motor assíncrono em Rust e renderizador WebGL 3D ultra-fluido.",
			img: "/logo.png"
		},
		{
			id: "news-2",
			title: "Fabulously Optimized 1.21.4 Disponível",
			date: "02 de Set, 2026",
			tag: "MODPACK",
			desc: "Otimização completa com Sodium 0.6, Iris Shaders e taxa máxima de FPS em placas AMD, Intel e Nvidia.",
			img: "/modpack_fo.webp"
		},
		{
			id: "news-3",
			title: "Chat P2P Direto Entre PCs Ativado",
			date: "28 de Ago, 2026",
			tag: "REDE",
			desc: "Conecte-se diretamente com seus amigos de LAN ou IP direto para trocar mensagens sem intermediários.",
			img: "/news_1.jpg"
		},
		{
			id: "news-4",
			title: "Cobblemon 1.6 & Better MC Atualizados",
			date: "24 de Ago, 2026",
			tag: "MODS",
			desc: "Centenas de novas criaturas, dimensões e masmorras prontas para jogar com um clique na Central de Conteúdo.",
			img: "/modpack_cobblemon.webp"
		}
	];

	function refreshNews() {
		toast("Notícias do Luxmc atualizadas!", "success");
	}
</script>

<aside class="w-[340px] shrink-0 h-full flex flex-col overflow-y-auto custom-scrollbar pr-1 pb-4 select-none space-y-5">
	
	<!-- Top: Notícias Section -->
	<div class="space-y-3">
		<div class="flex items-center justify-between pb-1 border-b border-white/5">
			<h2 class="text-xs font-black text-white uppercase tracking-wider flex items-center gap-2">
				<Newspaper class="w-3.5 h-3.5 text-brand-500" /> Notícias & Comunidade
			</h2>
			<button 
				type="button"
				class="text-white/40 hover:text-white transition-colors p-1.5 rounded-full hover:bg-white/10 cursor-pointer" 
				title="Atualizar Notícias" 
				onclick={refreshNews}
			>
				<RefreshCw class="w-3.5 h-3.5" />
			</button>
		</div>

		<div class="flex flex-col gap-3">
			{#each luxmcNews as news}
				<div 
					class="flex flex-col gap-2 bg-[#18191c] hover:bg-[#1f2025] p-3 rounded-2xl transition-all border border-white/5 hover:border-white/15 cursor-pointer group shadow-sm"
					onclick={() => toast(news.title, "info")}
					role="button"
					tabindex="0"
					onkeydown={(e) => { if (e.key === 'Enter') toast(news.title, "info"); }}
				>
					<div class="flex gap-3 items-start">
						<div class="w-20 h-16 rounded-xl bg-black/60 border border-white/5 overflow-hidden shrink-0 relative">
							<img src={news.img} class="w-full h-full object-cover opacity-85 group-hover:scale-110 transition-transform duration-300" alt="News" />
							<span class="absolute bottom-1 left-1 bg-black/80 text-[7px] font-black text-brand-500 px-2 py-0.5 rounded-full border border-white/10 uppercase">
								{news.tag}
							</span>
						</div>
						<div class="flex-1 min-w-0">
							<h4 class="text-xs font-extrabold text-white leading-snug group-hover:text-brand-500 transition-colors">
								{news.title}
							</h4>
							<span class="text-[9px] font-medium text-white/40 mt-1 flex items-center gap-1">
								<Calendar class="w-2.5 h-2.5" /> {news.date}
							</span>
						</div>
					</div>

					<p class="text-[11px] text-white/50 leading-relaxed line-clamp-2 pl-0.5">
						{news.desc}
					</p>
				</div>
			{/each}
		</div>
	</div>

	<!-- Bottom: Luxmc Community Hub -->
	<div class="space-y-3 pt-2">
		<!-- Promo Card: Luxmc Hub -->
		<div class="rounded-2xl bg-gradient-to-br from-[#1c1d22] to-[#141518] border border-white/10 p-4 relative overflow-hidden group cursor-pointer hover:border-brand-500/40 transition-all shadow-md">
			<div class="absolute -right-3 -top-3 w-16 h-16 bg-brand-500/10 rounded-full blur-xl group-hover:bg-brand-500/20 transition-all"></div>
			
			<div class="flex items-center justify-between relative z-10">
				<h3 class="text-xs font-black text-white flex items-center gap-1.5">
					<Zap class="w-3.5 h-3.5 text-brand-500" /> Luxmc Community
				</h3>
				<div class="w-6 h-6 rounded-full bg-white/5 flex items-center justify-center group-hover:bg-brand-500 group-hover:text-black transition-all">
					<ArrowRight class="w-3 h-3 -rotate-45" />
				</div>
			</div>
			<p class="text-[11px] text-white/60 mt-2 leading-relaxed relative z-10">
				Launcher Linux 100% livre, rápido e com otimização gráfica para extrair o máximo de FPS no seu setup.
			</p>
		</div>

		<!-- Bottom Social Icons -->
		<div class="flex items-center justify-center gap-5 text-white/30 pt-1 pb-2">
			<a href="https://discord.com" target="_blank" class="hover:text-white transition-colors" title="Discord">
				<svg class="w-4 h-4 fill-current" viewBox="0 0 24 24"><path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994.021-.041.001-.09-.041-.106a13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.061 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.893.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.028zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/></svg>
			</a>
			<a href="https://x.com" target="_blank" class="hover:text-white transition-colors" title="X">
				<svg class="w-3.5 h-3.5 fill-current" viewBox="0 0 24 24"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>
			</a>
			<a href="https://youtube.com" target="_blank" class="hover:text-white transition-colors" title="YouTube">
				<svg class="w-4 h-4 fill-current" viewBox="0 0 24 24"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>
			</a>
		</div>
	</div>

</aside>
