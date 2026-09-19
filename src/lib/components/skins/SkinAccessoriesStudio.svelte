<script lang="ts">
	import { fade } from "svelte/transition";
	import {
		Sparkles,
		Check,
		RotateCcw,
		Glasses,
		Crown,
		Headphones,
		Smile,
		Sliders
	} from "lucide-svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { skinsSave, type SaveSkinPayload } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let { onSkinUpdated }: { onSkinUpdated?: () => void } = $props();

	type Accessory = {
		id: string;
		name: string;
		category: string;
		desc: string;
		icon: typeof import("lucide-svelte").Circle;
		badge: string;
		color: string;
	};

	const accessories: Accessory[] = [
		{
			id: "headphones",
			name: "Headset Gamer RGB",
			category: "Áudio",
			desc: "Fones circumaurais com detalhes neon nos lados da cabeça.",
			icon: Headphones,
			badge: "Popular",
			color: "text-blue-400 bg-blue-500/10 border-blue-500/30"
		},
		{
			id: "sunglasses",
			name: "Óculos Escuros Pixel",
			category: "Estilo",
			desc: "Lentes escuras clássicas estilo Thug Life na face externa.",
			icon: Glasses,
			badge: "Clássico",
			color: "text-amber-400 bg-amber-500/10 border-amber-500/30"
		},
		{
			id: "crown",
			name: "Coroa Dourada Real",
			category: "Realeza",
			desc: "Coroa de ouro com detalhes de rubi no topo da cabeça.",
			icon: Crown,
			badge: "Lendário",
			color: "text-yellow-400 bg-yellow-500/10 border-yellow-500/30"
		},
		{
			id: "bandana",
			name: "Bandana Ninja Vermelha",
			category: "Ação",
			desc: "Faixa de combate estilo anime ao redor da testa.",
			icon: Sparkles,
			badge: "Combate",
			color: "text-rose-400 bg-rose-500/10 border-rose-500/30"
		},
		{
			id: "jacket",
			name: "Casaco Gamer Preto",
			category: "Vestuário",
			desc: "Jaqueta esportiva estilosa na camada de overlay do tronco e braços.",
			icon: Sliders,
			badge: "Novo",
			color: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30"
		}
	];

	let selectedAccessory = $state<string | null>(null);
	let isApplying = $state(false);

	async function applyAccessory(acc: Accessory) {
		const currentUrl = activeSkinStore.current.skinUrl;
		if (!currentUrl) {
			toast("Selecione uma skin primeiro para adicionar acessórios", "warning");
			return;
		}

		isApplying = true;
		try {
			const img = new Image();
			img.crossOrigin = "anonymous";
			img.src = currentUrl;
			await new Promise((resolve, reject) => {
				img.onload = resolve;
				img.onerror = reject;
			});

			const canvas = document.createElement("canvas");
			canvas.width = 64;
			canvas.height = 64;
			const ctx = canvas.getContext("2d");
			if (!ctx) return;

			ctx.imageSmoothingEnabled = false;
			ctx.drawImage(img, 0, 0, 64, 64);

			if (acc.id === "sunglasses") {
				ctx.fillStyle = "rgba(15, 15, 20, 1.0)";
				ctx.fillRect(40, 9, 8, 3);
				ctx.fillStyle = "rgba(240, 240, 255, 0.9)";
				ctx.fillRect(41, 10, 1, 1);
				ctx.fillRect(45, 10, 1, 1);
				ctx.fillStyle = "rgba(30, 30, 35, 1.0)";
				ctx.fillRect(38, 9, 2, 1);
				ctx.fillRect(48, 9, 2, 1);
			} else if (acc.id === "headphones") {
				ctx.fillStyle = "rgba(35, 35, 40, 1.0)";
				ctx.fillRect(42, 3, 4, 2);
				ctx.fillStyle = "rgba(25, 25, 30, 1.0)";
				ctx.fillRect(34, 10, 4, 4);
				ctx.fillStyle = "#00d2d3";
				ctx.fillRect(35, 11, 2, 2);

				ctx.fillStyle = "rgba(25, 25, 30, 1.0)";
				ctx.fillRect(50, 10, 4, 4);
				ctx.fillStyle = "#00d2d3";
				ctx.fillRect(51, 11, 2, 2);
			} else if (acc.id === "crown") {
				ctx.fillStyle = "#f1c40f";
				ctx.fillRect(40, 7, 8, 2);
				ctx.fillRect(32, 7, 8, 2);
				ctx.fillRect(48, 7, 8, 2);
				ctx.fillRect(56, 7, 8, 2);

				ctx.fillStyle = "#e74c3c";
				ctx.fillRect(42, 7, 1, 1);
				ctx.fillRect(45, 7, 1, 1);
				ctx.fillStyle = "#f39c12";
				ctx.fillRect(40, 6, 1, 1);
				ctx.fillRect(43, 6, 2, 1);
				ctx.fillRect(47, 6, 1, 1);
			} else if (acc.id === "bandana") {
				ctx.fillStyle = "#e74c3c";
				ctx.fillRect(40, 8, 8, 2);
				ctx.fillRect(32, 8, 8, 2);
				ctx.fillRect(48, 8, 8, 2);
				ctx.fillRect(56, 8, 8, 2);
				ctx.fillStyle = "#bdc3c7";
				ctx.fillRect(43, 8, 2, 2);
			} else if (acc.id === "jacket") {
				ctx.fillStyle = "#1e272e";
				ctx.fillRect(20, 36, 8, 12);
				ctx.fillRect(44, 36, 4, 12);
				ctx.fillRect(52, 52, 4, 12);
				ctx.fillStyle = "#00d2d3";
				ctx.fillRect(23, 38, 2, 8);
			}

			const newSkinDataUrl = canvas.toDataURL("image/png");
			activeSkinStore.setSkin({ skinUrl: newSkinDataUrl, url: newSkinDataUrl, avatarUrl: newSkinDataUrl, custom: true });
			const skinPayload: SaveSkinPayload = {
				id: "skin_" + Date.now(),
				name: `${activeSkinStore.current.type === 'alex' ? 'Alex' : 'Steve'} com ${acc.name}`,
				modelType: activeSkinStore.current.type,
				skinUrl: newSkinDataUrl,
				avatarUrl: newSkinDataUrl,
				isCustom: true
			};
			await skinsSave(skinPayload);

			selectedAccessory = acc.id;
			playSound("click");
			toast(`${acc.name} adicionado à sua skin!`, "success");
			if (onSkinUpdated) onSkinUpdated();
		} catch (e) {
			toast("Falha ao adicionar acessório: " + String(e), "error");
		} finally {
			isApplying = false;
		}
	}
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between border-b border-fg/5 pb-3">
		<div>
			<h3 class="text-xs font-black text-fg uppercase tracking-wider flex items-center gap-2">
				<Sparkles class="w-3.5 h-3.5 text-brand-400" />
				Estúdio de Acessórios 3D
			</h3>
			<p class="text-[11px] text-fg/40 mt-0.5">
				Estampe acessórios visuais diretamente na 2ª camada da sua skin sem precisar de softwares de edição
			</p>
		</div>
	</div>

	<div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3">
		{#each accessories as acc}
			{@const Icon = acc.icon}
			<div class="p-4 rounded-2xl bg-bg-elevated border border-fg/5 hover:border-fg/15 transition-all flex flex-col justify-between group shadow-sm">
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<div class="w-9 h-9 rounded-xl flex items-center justify-center {acc.color} shadow-sm">
							<Icon class="w-4 h-4" />
						</div>
						<span class="text-[9px] font-extrabold uppercase px-2 py-0.5 rounded-full {acc.color}">
							{acc.badge}
						</span>
					</div>

					<div>
						<h4 class="text-xs font-bold text-fg group-hover:text-amber-300 transition-colors">{acc.name}</h4>
						<p class="text-[10px] text-fg/40 leading-relaxed mt-1">{acc.desc}</p>
					</div>
				</div>

				<button
					type="button"
					class="mt-4 w-full py-2 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center justify-center gap-1.5 active:scale-[0.98] {selectedAccessory === acc.id ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' : 'bg-fg/5 hover:bg-fg/10 text-fg/80 hover:text-fg border border-fg/10'}"
					onclick={() => applyAccessory(acc)}
					disabled={isApplying}
				>
					{#if selectedAccessory === acc.id}
						<Check class="w-3.5 h-3.5 text-emerald-400" />
						<span>Aplicado</span>
					{:else}
						<Sparkles class="w-3.5 h-3.5 text-brand-400" />
						<span>Aplicar na Skin</span>
					{/if}
				</button>
			</div>
		{/each}
	</div>
</div>
