<script lang="ts">
	import { onMount } from "svelte";
	import { 
		RefreshCw, 
		Sparkles, 
		Upload, 
		ExternalLink, 
		Shirt, 
		Check, 
		Shield, 
		Globe, 
		Info 
	} from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import SkinViewer3D from "$lib/components/ui/SkinViewer3D.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	let isUpdating = $state(false);
	let skinType = $state<"steve" | "alex">("steve");
	let isRotating = $state(true);

	const isMicrosoft = $derived(
		Boolean(
			account.value?.minecraftToken &&
			!account.value?.id.startsWith("offline_") &&
			!account.value?.id.startsWith("offline-")
		)
	);

	const username = $derived(account.value?.username || "Jogador");
	const currentSkinUrl = $derived(
		activeSkinStore.current.skinUrl ||
		account.value?.skinUrl ||
		`https://minotar.net/skin/${username}`
	);

	onMount(() => {
		if (activeSkinStore.current.type) {
			skinType = activeSkinStore.current.type;
		}
	});

	async function handleRefresh() {
		isUpdating = true;
		try {
			const skinUrl = account.value?.skinUrl || `https://minotar.net/skin/${username}?t=${Date.now()}`;
			activeSkinStore.setSkin({
				id: account.value?.uuid || "offline",
				name: username,
				url: `https://mc-heads.net/body/${username}/300`,
				skinUrl,
				avatarUrl: `https://mc-heads.net/avatar/${username}/100`,
				type: skinType,
				hasCape: Boolean(account.value?.capeUrl),
				capeType: account.value?.capeUrl ? "custom" : "none",
				customCapeUrl: account.value?.capeUrl || ""
			});
			toast("Visual da skin atualizado!", "success");
		} catch (e) {
			toast("Erro ao atualizar skin: " + String(e), "error");
		} finally {
			setTimeout(() => { isUpdating = false; }, 350);
		}
	}

	async function handleLocalUpload() {
		try {
			const selected = await open({
				multiple: false,
				filters: [{ name: "Minecraft Skin PNG", extensions: ["png"] }]
			});
			if (!selected || typeof selected !== "string") return;

			const dataUrl = convertFileSrc(selected);
			activeSkinStore.setSkin({
				id: "custom-" + Date.now(),
				name: "Skin Local",
				url: dataUrl,
				skinUrl: dataUrl,
				avatarUrl: dataUrl,
				type: skinType
			});
			toast("Skin local carregada com sucesso!", "success");
		} catch (e) {
			toast("Erro ao carregar arquivo de skin: " + String(e), "error");
		}
	}
</script>

<div class="h-full flex flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-10">
	
	<div class="flex items-center justify-between">
		<div>
			<div class="flex items-center gap-2.5">
				<h1 class="text-2xl font-black text-white tracking-tight">Personalização</h1>
				<span class="text-[11px] font-semibold text-white/50 bg-white/10 px-2 py-0.5 rounded-md">Beta</span>
			</div>
			<p class="text-xs text-white/50 mt-1">Personalize a sua experiência no Minecraft</p>
		</div>

		<button
			type="button"
			onclick={handleRefresh}
			disabled={isUpdating}
			class="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 active:scale-95 text-white text-xs font-bold border border-white/10 transition-all cursor-pointer shadow-sm disabled:opacity-50"
		>
			<RefreshCw class="w-3.5 h-3.5 {isUpdating ? 'animate-spin text-emerald-400' : 'text-white/70'}" />
			<span>Atualizar</span>
		</button>
	</div>

	<div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
		
		<div class="lg:col-span-4 flex flex-col gap-3">
			
			<button
				type="button"
				onclick={() => openUrl("https://luxmc.app/skins")}
				class="w-full flex items-center justify-between p-3.5 rounded-2xl bg-[#14151a] hover:bg-[#1c1d24] border border-white/5 text-white text-xs font-bold transition-all group cursor-pointer shadow-md"
			>
				<div class="flex items-center gap-2.5">
					<Sparkles class="w-4 h-4 text-emerald-400 group-hover:rotate-12 transition-transform" />
					<span>Descubra mais skins no site oficial</span>
				</div>
				<ExternalLink class="w-3.5 h-3.5 text-white/40 group-hover:text-white transition-colors" />
			</button>

			<div class="w-full h-[460px] rounded-3xl bg-[#111216] border border-white/5 relative overflow-hidden shadow-inner flex flex-col items-center justify-center p-2">
				<SkinViewer3D
					skinUrl={currentSkinUrl}
					slim={skinType === "alex"}
					cape={activeSkinStore.current.capeType}
					customCapeUrl={activeSkinStore.current.customCapeUrl}
					autoRotate={isRotating}
					className="w-full h-full"
				/>

				<div class="absolute bottom-3 left-3 right-3 flex items-center justify-between bg-black/60 backdrop-blur-sm border border-white/10 rounded-2xl p-1.5 shadow-lg">
					<div class="flex items-center gap-1">
						<button
							type="button"
							onclick={() => skinType = "steve"}
							class="px-2.5 py-1 rounded-xl text-[11px] font-bold transition-all cursor-pointer {skinType === 'steve' ? 'bg-emerald-500 text-black shadow-sm' : 'text-white/60 hover:text-white'}"
						>
							Steve (4px)
						</button>
						<button
							type="button"
							onclick={() => skinType = "alex"}
							class="px-2.5 py-1 rounded-xl text-[11px] font-bold transition-all cursor-pointer {skinType === 'alex' ? 'bg-emerald-500 text-black shadow-sm' : 'text-white/60 hover:text-white'}"
						>
							Alex (3px)
						</button>
					</div>

					<button
						type="button"
						onclick={handleLocalUpload}
						title="Testar arquivo PNG local"
						class="flex items-center gap-1 px-2.5 py-1 rounded-xl bg-white/10 hover:bg-white/20 text-white text-[11px] font-bold transition-all cursor-pointer"
					>
						<Upload class="w-3 h-3 text-emerald-400" />
						<span>Carregar PNG</span>
					</button>
				</div>
			</div>
		</div>

		<div class="lg:col-span-8 flex flex-col gap-4">
			
			<div class="p-6 rounded-3xl bg-[#14151a] border border-white/5 shadow-xl flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
				<div class="space-y-1.5">
					<h2 class="text-sm font-black text-white">A edição de skins está disponível para contas Microsoft</h2>
					<p class="text-xs text-white/50 leading-relaxed">Para definir uma skin numa conta offline, por favor use o website.</p>
				</div>

				<button
					type="button"
					onclick={() => openUrl("https://luxmc.app/skins")}
					class="px-5 py-2.5 rounded-xl bg-[#525fde] hover:bg-[#4350ce] text-white text-xs font-bold transition-all shadow-lg hover:shadow-[#525fde]/25 cursor-pointer flex items-center gap-2 shrink-0 active:scale-95"
				>
					<span>Defina a sua skin em luxmc.app</span>
					<ExternalLink class="w-3.5 h-3.5" />
				</button>
			</div>

			<div class="p-6 rounded-3xl bg-[#111216] border border-white/5 space-y-4 shadow-sm">
				<div class="flex items-center gap-2.5 text-xs font-bold text-white/70 uppercase tracking-wider">
					<Info class="w-4 h-4 text-emerald-400" />
					<span>Como funciona o sistema de skins do Luxmc</span>
				</div>

				<div class="grid grid-cols-1 md:grid-cols-2 gap-3.5 pt-1">
					<div class="p-4 rounded-2xl bg-[#16171d] border border-white/5 space-y-1.5">
						<div class="flex items-center gap-2 text-xs font-bold text-white">
							<Shield class="w-3.5 h-3.5 text-sky-400" /> Contas Microsoft
						</div>
						<p class="text-[11px] text-white/45 leading-relaxed">
							Sua skin e capa oficiais do Minecraft são sincronizadas diretamente dos servidores da Mojang.
						</p>
					</div>

					<div class="p-4 rounded-2xl bg-[#16171d] border border-white/5 space-y-1.5">
						<div class="flex items-center gap-2 text-xs font-bold text-white">
							<Globe class="w-3.5 h-3.5 text-emerald-400" /> Contas Offline / Luxmc
						</div>
						<p class="text-[11px] text-white/45 leading-relaxed">
							No nosso website você cria sua conta gratuitamente, faz o upload da sua skin e escolhe sua capa. O jogo carregará sua textura automaticamente!
						</p>
					</div>
				</div>

				<div class="pt-2 flex items-center justify-between text-xs text-white/40 border-t border-white/5">
					<span>Jogador conectado: <strong class="text-white">{username}</strong> ({isMicrosoft ? "Microsoft Online" : "Conta Offline"})</span>
					<span class="font-mono text-[10px] text-emerald-400">Motor 3D Ativo · 60 FPS</span>
				</div>
			</div>

		</div>

	</div>

</div>
