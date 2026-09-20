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

			const isSlim = activeSkinStore.current.type === "alex";
			const armW = isSlim ? 3 : 4;

			if (acc.id === "sunglasses") {
				// Front lenses
				ctx.fillStyle = "rgba(15, 15, 20, 1.0)";
				ctx.fillRect(40, 9, 8, 3);
				// White reflections
				ctx.fillStyle = "rgba(240, 240, 255, 0.9)";
				ctx.fillRect(41, 10, 1, 1);
				ctx.fillRect(45, 10, 1, 1);
				// Frame bridge
				ctx.fillStyle = "rgba(30, 30, 35, 1.0)";
				ctx.fillRect(43, 9, 2, 1);
				// Right side temple/arm
				ctx.fillRect(35, 9, 5, 2);
				// Left side temple/arm
				ctx.fillRect(48, 9, 5, 2);
			} else if (acc.id === "headphones") {
				// Headband on top of head
				ctx.fillStyle = "#1e272e";
				ctx.fillRect(40, 3, 8, 2);
				ctx.fillStyle = "#00d2d3";
				ctx.fillRect(41, 3, 6, 1);

				// Right side band & ear cup
				ctx.fillStyle = "#1e272e";
				ctx.fillRect(35, 8, 2, 2);
				ctx.fillRect(34, 10, 4, 4);
				ctx.fillStyle = "#00d2d3";
				ctx.fillRect(35, 11, 2, 2);

				// Left side band & ear cup
				ctx.fillStyle = "#1e272e";
				ctx.fillRect(49, 8, 2, 2);
				ctx.fillRect(48, 10, 4, 4);
				ctx.fillStyle = "#00d2d3";
				ctx.fillRect(49, 11, 2, 2);

				// Mic boom on left face & front
				ctx.fillStyle = "#2c3e50";
				ctx.fillRect(48, 13, 3, 1);
				ctx.fillRect(40, 13, 2, 1);
				ctx.fillStyle = "#ff4757";
				ctx.fillRect(41, 13, 1, 1);
			} else if (acc.id === "crown") {
				// 360 headband (Right, Front, Left, Back)
				ctx.fillStyle = "#f1c40f";
				ctx.fillRect(32, 7, 8, 2);
				ctx.fillRect(40, 7, 8, 2);
				ctx.fillRect(48, 7, 8, 2);
				ctx.fillRect(56, 7, 8, 2);

				// Crown peaks on all sides
				ctx.fillStyle = "#f39c12";
				ctx.fillRect(32, 6, 1, 1);
				ctx.fillRect(35, 6, 2, 1);
				ctx.fillRect(39, 6, 1, 1);

				ctx.fillRect(40, 6, 1, 1);
				ctx.fillRect(43, 6, 2, 1);
				ctx.fillRect(47, 6, 1, 1);

				ctx.fillRect(48, 6, 1, 1);
				ctx.fillRect(51, 6, 2, 1);
				ctx.fillRect(55, 6, 1, 1);

				ctx.fillRect(56, 6, 1, 1);
				ctx.fillRect(59, 6, 2, 1);
				ctx.fillRect(63, 6, 1, 1);

				// Jewels
				ctx.fillStyle = "#e74c3c";
				ctx.fillRect(43, 7, 2, 1);
				ctx.fillStyle = "#2ecc71";
				ctx.fillRect(35, 7, 2, 1);
				ctx.fillStyle = "#3498db";
				ctx.fillRect(51, 7, 2, 1);
				ctx.fillStyle = "#9b59b6";
				ctx.fillRect(59, 7, 2, 1);
			} else if (acc.id === "bandana") {
				// 360 wrap
				ctx.fillStyle = "#e74c3c";
				ctx.fillRect(32, 8, 8, 2);
				ctx.fillRect(40, 8, 8, 2);
				ctx.fillRect(48, 8, 8, 2);
				ctx.fillRect(56, 8, 8, 2);

				// Front emblem
				ctx.fillStyle = "#bdc3c7";
				ctx.fillRect(43, 8, 2, 2);
				ctx.fillStyle = "#2c3e50";
				ctx.fillRect(43, 8, 1, 1);

				// Back knot and tails
				ctx.fillStyle = "#c0392b";
				ctx.fillRect(59, 10, 2, 3);
				ctx.fillStyle = "#962d22";
				ctx.fillRect(60, 13, 1, 2);
			} else if (acc.id === "jacket") {
				const jacketBase = "#1e272e";
				const jacketAccent = "#00d2d3";
				const jacketDark = "#141a1f";

				// Torso Layer 2: Top (20, 32, 8, 4), Bottom (28, 32, 8, 4), Right (16, 36, 4, 12), Front (20, 36, 8, 12), Left (28, 36, 4, 12), Back (32, 36, 8, 12)
				ctx.fillStyle = jacketBase;
				ctx.fillRect(20, 32, 8, 4);
				ctx.fillRect(28, 32, 8, 4);
				ctx.fillRect(16, 36, 4, 12);
				ctx.fillRect(20, 36, 8, 12);
				ctx.fillRect(28, 36, 4, 12);
				ctx.fillRect(32, 36, 8, 12);

				// Front zipper and pockets
				ctx.fillStyle = jacketAccent;
				ctx.fillRect(23, 37, 2, 10);
				ctx.fillStyle = jacketDark;
				ctx.fillRect(21, 43, 2, 2);
				ctx.fillRect(25, 43, 2, 2);

				// Back gamer stripe
				ctx.fillStyle = jacketAccent;
				ctx.fillRect(35, 39, 2, 5);

				// Right Sleeve Layer 2: Top (44, 32, armW, 4), Bottom (44+armW, 32, armW, 4), Right (40, 36, 4, 12), Front (44, 36, armW, 12), Left (44+armW, 36, 4, 12), Back (48+armW, 36, armW, 12)
				ctx.fillStyle = jacketBase;
				ctx.fillRect(44, 32, armW, 4);
				ctx.fillRect(44 + armW, 32, armW, 4);
				ctx.fillRect(40, 36, 4, 12);
				ctx.fillRect(44, 36, armW, 12);
				ctx.fillRect(44 + armW, 36, 4, 12);
				ctx.fillRect(48 + armW, 36, armW, 12);

				// Right sleeve accent stripe
				ctx.fillStyle = jacketAccent;
				ctx.fillRect(41, 38, 2, 8);

				// Left Sleeve Layer 2: Top (52, 48, armW, 4), Bottom (52+armW, 48, armW, 4), Right (48, 52, 4, 12), Front (52, 52, armW, 12), Left (52+armW, 52, 4, 12), Back (56+armW, 52, armW, 12)
				ctx.fillStyle = jacketBase;
				ctx.fillRect(52, 48, armW, 4);
				ctx.fillRect(52 + armW, 48, armW, 4);
				ctx.fillRect(48, 52, 4, 12);
				ctx.fillRect(52, 52, armW, 12);
				ctx.fillRect(52 + armW, 52, 4, 12);
				ctx.fillRect(56 + armW, 52, armW, 12);

				// Left sleeve accent stripe
				ctx.fillStyle = jacketAccent;
				ctx.fillRect(52 + armW + 1, 54, 2, 8);
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
