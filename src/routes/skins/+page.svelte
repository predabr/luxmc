<script lang="ts">
    import { skinAvatar, createSkinAvatar, inferSkinModelType, classifyTexture, textureCanvas } from "$lib/utils/textureImage";
	import { onDestroy, onMount, untrack } from "svelte";
	import { deepLinks } from "$lib/stores/deepLinks.svelte";
	import { authChangeSkin } from "$lib/api/auth";
	import {
		RefreshCw,
		Upload,
		Search,
		RotateCcw,
		Play,
		Pause,
		Layers,
		CheckCircle2,
		Share2,
		Pencil,
		Plus,
		Check,
		Info,
		ChevronDown,
		ChevronUp,
		Move,
		Trash2,
		X,
		ArrowLeft,
		ChevronRight,
		Shirt
	} from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { open } from "@tauri-apps/plugin-dialog";

	import SkinViewer3D from "$lib/components/ui/SkinViewer3D.svelte";
	import Skin2DPreview from "$lib/components/ui/Skin2DPreview.svelte";
	import { activeSkinStore, saveAppearance, type CapeType } from "$lib/stores/skin.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { getCapePreviewDataUrl, getFullCapeDataUrl, migrateGeneratedCape } from "$lib/utils/capeTextures";
	import { loadTextureImage, normalizeCape } from "$lib/utils/textureImage";
	import { goto } from "$app/navigation";
	import { authResolveTexture, authReadLocalTexture } from "$lib/api/auth";
	import { fade, slide, fly } from "svelte/transition";

	let isUpdating = $state(false);
	let skinType = $state<"steve" | "alex">("steve");
	let isRotating = $state(true);
	let activeAnimation = $state<"idle" | "walk" | "run" | "fly" | "none">("walk");
	let viewerRef = $state<SkinViewer3D | null>(null);

	let searchNick = $state("");
	let isSearchingNick = $state(false);
	let selectedCape = $state<CapeType>("luxmc");
	let customCapeDataUrl = $state("");
    let customCapePreview = $state("");
	let showEditModal = $state(false);
	let appliedSelection = $state("");
	let hydrated = $state(false);
    let draftOwner = $state("");
	let saving = $state(false);
	let saveError = $state("");

	let savedSkinsExpanded = $state(true);
	type SavedSkinItem = { id: string; name: string; url: string; model: "steve" | "alex" };
	const defaultSavedSkins: SavedSkinItem[] = [];
	let savedSkins = $state<SavedSkinItem[]>(defaultSavedSkins);

	let selectedSkinId = $state("");
	let selectedSkinNick = $state("Nenhuma skin salva");
	let editingSkinId = $state<string | null>(null);
	let editingSkinName = $state("");

	const isMicrosoft = $derived(
		Boolean(
			account.value?.minecraftToken && account.value.minecraftToken.length > 100 &&
			!account.value?.id.startsWith("offline_") &&
			!account.value?.id.startsWith("offline-")
		)
	);

	const username = $derived(account.value?.username || "Steve");

	let previewSkinUrl = $state("");

	const currentSkinUrl = $derived(
		previewSkinUrl ||
		activeSkinStore.current.skinUrl ||
		account.value?.skinUrl ||
		"/steve.png"
	);

	const selection = $derived(JSON.stringify([currentSkinUrl, skinType, selectedCape, customCapeDataUrl]));
	const hasPendingChange = $derived(hydrated && selection !== appliedSelection);

	const capeList: { id: CapeType; name: string; desc: string }[] = [
		{ id: "none", name: "Nenhuma", desc: "Sem capa" },
		{ id: "luxmc", name: "Luxmc Oficial", desc: "Ouro & Obsidiana" },
		{ id: "optifine", name: "OptiFine OF", desc: "Clássica vermelha" },
		{ id: "migrator", name: "Migração", desc: "Ouro Mojang" },
		{ id: "cherry", name: "Flor de cerejeira", desc: "Flor de Cerejeira" },
		{ id: "vanilla", name: "Capa Vanilla", desc: "Edição Especial" },
		{ id: "minecon2011", name: "Minecon 2011", desc: "Creeper Vermelho" },
		{ id: "minecon2012", name: "Minecon 2012", desc: "Picareta Noturna" },
		{ id: "minecon2013", name: "Minecon 2013", desc: "Pistão Redstone" },
		{ id: "minecon2015", name: "Minecon 2015", desc: "Golem de Ferro" },
		{ id: "minecon2016", name: "Minecon 2016", desc: "Enderman Roxo" },
		{ id: "tiktok", name: "TikTok", desc: "Comemorativa" },
		{ id: "twitch", name: "Twitch", desc: "Roxa Glitch" },
		{ id: "custom", name: "Customizada", desc: "Arquivo .PNG local" },
	];

	onMount(() => {
		if (activeSkinStore.current.type) {
			skinType = activeSkinStore.current.type;
		}
		if (activeSkinStore.current.capeType) {
			selectedCape = activeSkinStore.current.capeType;
		}
		if (activeSkinStore.current.customCapeUrl) {
			customCapeDataUrl = activeSkinStore.current.customCapeUrl;
		}
		if (activeSkinStore.current.skinUrl) {
			previewSkinUrl = activeSkinStore.current.skinUrl;
		}
		try {
			const saved = localStorage.getItem("luxmc_saved_skins");
			if (saved) {
				const parsed = JSON.parse(saved);
				if (Array.isArray(parsed)) {
                    savedSkins = parsed.filter((item): item is SavedSkinItem => item && typeof item.id === "string" && typeof item.name === "string" && typeof item.url === "string" &&
                        item.id !== "default_steve" && item.id !== "default_alex");
                    if (savedSkins.length !== parsed.length && !localStorage.getItem("luxmc_saved_skins_before_cleanup")) localStorage.setItem("luxmc_saved_skins_before_cleanup", saved);
                    localStorage.setItem("luxmc_saved_skins", JSON.stringify(savedSkins));
                }
			}
		} catch {}
        if (account.value) {
            previewSkinUrl = account.value.skinUrl || "/steve.png";
            skinType = account.value.skinVariant?.toLowerCase() === "slim" ? "alex" : "steve";
            selectedCape = account.value.capeUrl ? "custom" : "none";
            customCapeDataUrl = account.value.capeUrl || "";
        }
		const appliedAccountSkin = account.value?.skinUrl || activeSkinStore.current.skinUrl || "/steve.png";
		const appliedAccountModel = account.value ? (account.value.skinVariant?.toLowerCase() === "slim" ? "alex" : "steve") : skinType;
		const appliedAccountCape = account.value ? (account.value.capeUrl ? "custom" : "none") : selectedCape;
		const appliedAccountCapeUrl = account.value?.capeUrl || customCapeDataUrl;
		const accountAppearance = JSON.stringify([account.value?.skinUrl || "", account.value?.skinVariant || "", account.value?.capeUrl || ""]);
		const initialSkin = previewSkinUrl || account.value?.skinUrl || activeSkinStore.current.skinUrl || "/steve.png";
		if (initialSkin && initialSkin !== "/steve.png" && initialSkin !== "/alex.png") {
			void loadTextureImage(initialSkin, new AbortController().signal).then(img => {
				const canvas = textureCanvas(img);
				const inferred = inferSkinModelType(canvas);
				if (!account.value?.skinVariant) {
					skinType = inferred;
				}
			}).catch(() => {});
		}
		let persistedSkinId = "";
		try {
			persistedSkinId = localStorage.getItem("luxmc_selected_skin_id") || "";
		} catch {}

		if (persistedSkinId) {
			const savedMatch = savedSkins.find(s => s.id === persistedSkinId);
			if (savedMatch) {
				selectedSkinId = savedMatch.id;
				selectedSkinNick = savedMatch.name;
				previewSkinUrl = savedMatch.url;
				skinType = savedMatch.model;
			}

		if (!savedSkins.some(skin => skin.id === selectedSkinId)) {
			const matchSaved = savedSkins.find(s => s.url === initialSkin);
			if (matchSaved) {
				selectedSkinId = matchSaved.id;
				selectedSkinNick = matchSaved.name;
			} else if (savedSkins.length > 0) {
					selectedSkinId = savedSkins[0].id;
					selectedSkinNick = savedSkins[0].name;
					previewSkinUrl = savedSkins[0].url;
					skinType = savedSkins[0].model;
			} else {
				selectedSkinId = "";
				selectedSkinNick = "Nenhuma skin salva";
			}
			}
		}
        appliedSelection = JSON.stringify([appliedAccountSkin, appliedAccountModel, appliedAccountCape, appliedAccountCapeUrl]);
        draftOwner = account.value?.id || "local";
        try {
            const savedApplication = JSON.parse(localStorage.getItem(`luxmc_applied_skin_selection:${draftOwner}`) || "null");
            if (savedApplication?.accountAppearance === accountAppearance && Array.isArray(savedApplication.selection)) {
                appliedSelection = JSON.stringify(savedApplication.selection);
            }
        } catch {}
        try {
            const draft = JSON.parse(sessionStorage.getItem(`luxmc_skin_draft:${draftOwner}`) || "null");
            if (draft && typeof draft.skinUrl === "string" && /^(data:image\/png;base64,|https:\/\/|\/)/.test(draft.skinUrl) &&
                (draft.model === "steve" || draft.model === "alex") && capeList.some(cape => cape.id === draft.cape) &&
                typeof draft.customCapeUrl === "string" && typeof draft.id === "string" && typeof draft.name === "string" && savedSkins.some(saved => saved.id === draft.id)) {
                previewSkinUrl = draft.skinUrl;
                skinType = draft.model;
                selectedCape = draft.cape;
                customCapeDataUrl = draft.customCapeUrl;
                selectedSkinId = draft.id;
                selectedSkinNick = draft.name;
            }
        } catch {}
        hydrated = true;
	});

    $effect(() => {
        if (!hydrated || draftOwner !== (account.value?.id || "local")) return;
        const key = `luxmc_skin_draft:${draftOwner}`;
        try {
            if (!hasPendingChange) sessionStorage.removeItem(key);
            else sessionStorage.setItem(key, JSON.stringify({ skinUrl: currentSkinUrl, model: skinType, cape: selectedCape, customCapeUrl: customCapeDataUrl, id: selectedSkinId, name: selectedSkinNick }));
        } catch {}
    });

    $effect(() => {
        const skin = deepLinks.skin;
        if (!skin) return;
        untrack(() => {
            const saved: SavedSkinItem = { id: crypto.randomUUID(), name: "Skin da Web", url: skin.url, model: skin.model === "slim" ? "alex" : "steve" };
            savedSkins = [saved, ...savedSkins];
            try { localStorage.setItem("luxmc_saved_skins", JSON.stringify(savedSkins)); } catch {}
            selectValidatedSkin(saved, saved.model);
            deepLinks.skin = null;
            toast("Skin recebida do portal web!", "info");
        });
    });

    onDestroy(() => viewerRef?.dispose());

    $effect(() => {
        if (!hydrated) return;
        const source = currentSkinUrl;
        const model = skinType;
        if (!source) return;
        const controller = new AbortController();
        void createSkinAvatar(source, controller.signal, model).then(head => {
            if (!controller.signal.aborted && head) {
                activeSkinStore.setSkin({ avatarUrl: head });
            }
        }).catch(() => {});
        return () => controller.abort();
    });

    async function selectSavedSkin(skin: SavedSkinItem) {
        importController?.abort();
        const controller = new AbortController();
        importController = controller;
        try {
            const image = await loadTextureImage(skin.url, controller.signal);
            if (controller.signal.aborted) return;
            const canvas = textureCanvas(image);
            if (classifyTexture(canvas, skin.name) === "cape") {
                customCapeDataUrl = normalizeCape(image).toDataURL("image/png");
                selectedCape = "custom";
                removeSavedSkin(skin.id);
                toast("Capa identificada e atribuída automaticamente à aba de Capas!", "success");
                return;
            }
            selectValidatedSkin(skin, (skin.model as string === "alex" || skin.model as string === "slim") ? "alex" : (skin.model as string === "steve" || skin.model as string === "classic") ? "steve" : inferSkinModelType(canvas));
        } catch (error) {
            if (!controller.signal.aborted) toast("Não foi possível selecionar a skin: " + String(error), "error");
        }
    }

    function selectValidatedSkin(skin: SavedSkinItem, model: "steve" | "alex") {
        selectedSkinId = skin.id;
        selectedSkinNick = skin.name;
        skinType = model;
        previewSkinUrl = skin.url;
        try {
            localStorage.setItem("luxmc_selected_skin_id", skin.id);
            localStorage.setItem("luxmc_selected_skin_nick", skin.name);
        } catch {}
    }

	function removeSavedSkin(id: string) {
		savedSkins = savedSkins.filter(s => s.id !== id);
		if (selectedSkinId === id) {
			if (savedSkins.length > 0) {
				void selectSavedSkin(savedSkins[0]);
			} else {
				selectedSkinId = "";
				selectedSkinNick = "Nenhuma skin salva";
				previewSkinUrl = "";
			}
		}
		try {
			localStorage.setItem("luxmc_saved_skins", JSON.stringify(savedSkins));
		} catch {}
	}

	function startRenameSkin(skin: SavedSkinItem) {
		editingSkinId = skin.id;
		editingSkinName = skin.name;
	}

	function saveRenameSkin(id: string) {
		const name = editingSkinName.trim();
		if (name) {
			savedSkins = savedSkins.map(s => s.id === id ? { ...s, name } : s);
			if (selectedSkinId === id) {
				selectedSkinNick = name;
			}
			try {
				localStorage.setItem("luxmc_saved_skins", JSON.stringify(savedSkins));
			} catch {}
		}
		editingSkinId = null;
		editingSkinName = "";
	}

	function setModelType(model: "steve" | "alex") {
		skinType = model;
		if (savedSkins.some(skin => skin.id === selectedSkinId)) {
			savedSkins = savedSkins.map(s => s.id === selectedSkinId ? { ...s, model } : s);
			try {
				localStorage.setItem("luxmc_saved_skins", JSON.stringify(savedSkins));
			} catch {}
		}
	}

    let importController: AbortController | null = null;
    onMount(() => () => importController?.abort());
    async function handleAddSkinFile() {
        try {
            const selected = await open({ multiple: false, filters: [{ name: "Skin ou Capa (PNG, WebP)", extensions: ["png", "webp", "jpeg", "jpg"] }] });
            if (!selected || typeof selected !== "string") return;
            importController?.abort();
            const controller = new AbortController();
            importController = controller;
            const texture = await authReadLocalTexture(selected);
            if (controller.signal.aborted) return;
            const image = await loadTextureImage(texture, controller.signal);
            if (controller.signal.aborted) return;
            const canvas = textureCanvas(image);
            const filename = selected.split(/[/\\]/).pop() || "";
            if (classifyTexture(canvas, filename) === "cape") {
                customCapeDataUrl = normalizeCape(image).toDataURL("image/png");
                selectedCape = "custom";
                toast("Capa identificada e atribuída automaticamente à aba de Capas!", "success");
                return;
            }
            const name = filename.replace(/\.(png|webp|jpe?g)$/i, "").replace(/[-_]+/g, " ").trim() || `Skin ${savedSkins.length + 1}`;
            const model = inferSkinModelType(canvas);
            const url = texture;
            const skin: SavedSkinItem = { id: crypto.randomUUID(), name, url, model };
            const nextSkins = [skin, ...savedSkins];
            localStorage.setItem("luxmc_saved_skins", JSON.stringify(nextSkins));
            if (controller.signal.aborted) return;
            savedSkins = nextSkins;
            selectValidatedSkin(skin, model);
            toast(`Skin "${name}" adicionada (${model === "alex" ? "Fino / Alex" : "Clássico / Steve"})!`, "success");
        } catch (error) {
            if (!(error instanceof DOMException && error.name === "AbortError")) toast("Erro ao carregar textura: " + String(error), "error");
        }
    }

    let searchController: AbortController | null = null;
    onMount(() => () => searchController?.abort());
    async function handleSearchNick() {
        const nick = searchNick.trim();
        if (!/^[A-Za-z0-9_]{3,16}$/.test(nick)) { toast("Use um nickname de 3 a 16 letras, números ou _.", "info"); return; }
        searchController?.abort();
        const controller = new AbortController();
        searchController = controller;
        isSearchingNick = true;
        try {
            const url = `https://mineskin.eu/skin/${encodeURIComponent(nick)}`;
            const resolved = await authResolveTexture(url);
            const image = await loadTextureImage(resolved, controller.signal);
            if (controller.signal.aborted) return;
            const canvas = textureCanvas(image);
            const model = inferSkinModelType(canvas);
            const saved: SavedSkinItem = { id: crypto.randomUUID(), name: nick, url: resolved, model };
            savedSkins = [saved, ...savedSkins];
            localStorage.setItem("luxmc_saved_skins", JSON.stringify(savedSkins));
            selectValidatedSkin(saved, model);

        } catch (error) { if (!controller.signal.aborted) toast(String(error), "error"); }
        finally { if (!controller.signal.aborted) isSearchingNick = false; }
    }

	async function handleCustomCapeUpload() {
		try {
			const selected = await open({
				multiple: false,
				filters: [{ name: "Capa do Minecraft (PNG, WebP)", extensions: ["png", "webp", "jpeg", "jpg"] }]
			});
			if (!selected || typeof selected !== "string") return;

            importController?.abort();
            const controller = new AbortController();
            importController = controller;
            const image = await loadTextureImage(await authReadLocalTexture(selected), controller.signal);
            customCapeDataUrl = normalizeCape(image).toDataURL("image/png");
			selectedCape = "custom";
			toast("Capa personalizada carregada!", "success");
		} catch (e) {
			toast("Erro ao carregar capa personalizada: " + String(e), "error");
		}
	}

    $effect(() => {
        const source = customCapeDataUrl;
        customCapePreview = "";
        if (!source) return;
        const controller = new AbortController();
        void loadTextureImage(source, controller.signal).then(image => {
            if (controller.signal.aborted) return;
            const full = normalizeCape(image);
            const scale = full.width / 64;
            const canvas = document.createElement("canvas");
            canvas.width = 40; canvas.height = 64;
            const context = canvas.getContext("2d");
            if (!context) return;
            context.imageSmoothingEnabled = false;
            context.drawImage(full, scale, scale, 10 * scale, 16 * scale, 0, 0, 40, 64);
            customCapePreview = canvas.toDataURL("image/png");
        }).catch(() => {});
        return () => controller.abort();
    });

    $effect(() => {
        if (!hydrated || selectedCape !== "custom" || !customCapeDataUrl) return;
        const source = customCapeDataUrl;
        const controller = new AbortController();
        void migrateGeneratedCape(source, controller.signal).then(corrected => {
            if (!controller.signal.aborted && corrected !== source) customCapeDataUrl = corrected;
        }).catch(() => {});
        return () => controller.abort();
    });

    async function applyAppearance() {
        if (!hydrated || saving || !hasPendingChange) return;
        const applied = selection;
        const source = currentSkinUrl;
        const model = skinType;
        const cape = selectedCape;
        const capeSource = cape === "custom" ? customCapeDataUrl : cape === "none" ? "" : getFullCapeDataUrl(cape);
        const controller = new AbortController();
        saving = true;
        saveError = "";
        const normalize = async (url: string, isCape = false) => {
            if (url.startsWith("https://")) url = await authResolveTexture(url);
            const image = await loadTextureImage(url, controller.signal);
            if (isCape) return normalizeCape(image).toDataURL("image/png");
            const canvas = document.createElement("canvas");
            canvas.width = image.naturalWidth;
            canvas.height = image.naturalHeight;
            if (canvas.width > 2048 || canvas.height > 2048) throw new Error("Textura muito grande");
            const ctx = canvas.getContext("2d");
            if (!ctx) throw new Error("Canvas indisponível");
            ctx.drawImage(image, 0, 0);
            if (classifyTexture(canvas) !== "skin") throw new Error("Esta textura é uma capa. Importe-a na seção de Capas.");
            return url.startsWith("data:image/png;base64,") ? url : canvas.toDataURL("image/png");
        };
        await Promise.all([normalize(source), capeSource ? normalize(capeSource, true) : Promise.resolve(null)]).then(async ([skin, capeUrl]) => {
            if (controller.signal.aborted) return;
            const avatarUrl = await createSkinAvatar(skin, controller.signal, model);
            if (controller.signal.aborted) return;
            await saveAppearance(skin, model === "alex" ? "slim" : "classic", capeUrl, avatarUrl);
            if (controller.signal.aborted) return;
            appliedSelection = applied;
            try {
                localStorage.setItem(`luxmc_applied_skin_selection:${account.value?.id || "local"}`, JSON.stringify({
                    accountAppearance: JSON.stringify([account.value?.skinUrl || "", account.value?.skinVariant || "", account.value?.capeUrl || ""]),
                    selection: JSON.parse(applied)
                }));
            } catch {}
            toast("Skin e capa aplicadas. As alterações serão usadas na próxima abertura do jogo.", "success");
            activeSkinStore.setSkin({ id: selectedSkinId, name: selectedSkinNick, avatarUrl, skinUrl: skin, type: model, capeType: cape, hasCape: cape !== "none", customCapeUrl: capeUrl || "" });
            try {
                localStorage.setItem("luxmc_selected_skin_id", selectedSkinId);
                localStorage.setItem("luxmc_selected_skin_nick", selectedSkinNick);
            } catch {}
        }).catch(error => {
            if (!controller.signal.aborted) { saveError = String(error); toast(saveError, "error"); }
        }).finally(() => { if (!controller.signal.aborted) saving = false; });
    }

    async function handleApplyToAccount(): Promise<void> {
        if (!account.value || isUpdating || saving) return;
        if (!isMicrosoft) { toast("Entre com Microsoft para sincronizar com o perfil oficial. Sua aparência local já foi salva.", "info"); return; }
        isUpdating = true;
        try {
            await authChangeSkin(account.value.id, skinType === "alex" ? "slim" : "classic", account.value.skinUrl || currentSkinUrl);

            toast("Skin sincronizada com o perfil oficial.", "success");
        } catch (error) { toast(String(error), "error"); }
        finally { isUpdating = false; }
    }

	async function handleSyncWithWeb(): Promise<void> {
		const nick = selectedSkinNick || username;
		const webUrl = `https://luxmc-r92.pages.dev/skins.html?nick=${encodeURIComponent(nick)}&model=${skinType}&cape=${selectedCape}`;
		try {
			await openUrl(webUrl);
			toast("Abrindo Estúdio de Skins no navegador...", "info");
		} catch (error) {
			toast(String(error), "error");
		}
	}
</script>

<div class="h-full flex flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-24">

	<header class="flex items-center justify-between gap-4 py-1">
		<div class="flex items-center gap-3">
			<div class="flex items-center gap-1 bg-bg/35 backdrop-blur-xl border border-fg/10 rounded-xl p-1 shadow-sm">
				<button
					type="button"
					class="p-1.5 rounded-lg text-fg/40 hover:text-fg hover:bg-fg/5 transition-colors cursor-pointer"
					title="Voltar"
					onclick={() => history.back()}
				>
					<ArrowLeft class="w-3.5 h-3.5" />
				</button>
				<button
					type="button"
					class="p-1.5 rounded-lg text-fg/40 hover:text-fg hover:bg-fg/5 transition-colors cursor-pointer"
					title="Avançar"
					onclick={() => history.forward()}
				>
					<ChevronRight class="w-3.5 h-3.5" />
				</button>
			</div>

			<div class="flex items-center gap-2 text-xs font-bold text-fg/80">
				<Shirt class="w-3.5 h-3.5 text-fg/60" />
				<span class="text-fg font-extrabold">Skins e capas</span>
			</div>
		</div>

		<div class="flex items-center gap-2.5">
			<button
				type="button"
				onclick={handleSyncWithWeb}
				class="flex items-center gap-2 px-3.5 py-1.5 rounded-xl bg-bg/35 backdrop-blur-xl hover:bg-fg/10 text-fg/80 hover:text-fg text-xs font-bold border border-fg/10 transition-all cursor-pointer shadow-sm"
				title="Abrir no NameMC / Web Studio"
			>
				<Share2 class="w-3.5 h-3.5 text-brand-400" />
				<span>Web Studio</span>
			</button>
		</div>
	</header>

    <div class="sticky top-0 z-20 shrink-0 flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-fg/10 bg-bg/35 p-4 shadow-elevated backdrop-blur-xl">
        <p class="text-sm text-fg-muted" role="status">{saving ? "Aplicando aparência…" : saveError ? "Falha ao aplicar: " + saveError : hasPendingChange ? "Você tem alterações não aplicadas." : "Sua aparência está aplicada."}</p>
        <div class="flex gap-2">
            {#if isMicrosoft}<button class="luxmc-control" disabled={saving || isUpdating || hasPendingChange} onclick={handleApplyToAccount}>{isUpdating ? "Sincronizando…" : "Sincronizar com Microsoft"}</button>{/if}
            <button class="rounded-xl bg-brand-500 px-5 py-2.5 text-sm font-semibold text-brand-foreground disabled:opacity-50" disabled={saving || !hasPendingChange} onclick={applyAppearance}>{saving ? "Aplicando…" : "Aplicar skin e capa"}</button>
        </div>
    </div>


	<div class="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">

		<div class="lg:col-span-4 flex flex-col items-center gap-4 lg:sticky lg:top-2">

			<div class="w-full h-[520px] rounded-3xl bg-bg/35 backdrop-blur-xl border border-fg/10 relative overflow-hidden shadow-2xl flex flex-col items-center justify-center p-4">

				<div class="absolute top-3 left-3 right-3 flex items-center justify-between z-10 pointer-events-none">
					<div class="flex items-center gap-1.5 min-w-0">
						<span class="text-[11px] font-black uppercase tracking-wider text-fg/70 bg-fg/[0.06] border border-fg/10 px-2.5 py-1 rounded-full max-w-[130px] truncate" title={selectedSkinNick || "Skin"}>
							{selectedSkinNick || "Skin"}
						</span>
						<span class="text-[11px] font-black uppercase tracking-wider text-fg/50 bg-fg/[0.04] border border-fg/10 px-2.5 py-1 rounded-full shrink-0">
							{skinType === "alex" ? "Fino (3 px)" : "Clássico (4 px)"}
						</span>
					</div>

					<button
						type="button"
						onclick={() => isRotating = !isRotating}
						class="p-2 rounded-xl bg-black/40 hover:bg-black/60 border border-fg/10 text-fg/70 hover:text-fg pointer-events-auto transition-all cursor-pointer shadow-sm"
						title={isRotating ? "Pausar rotação" : "Ativar rotação"}
					>
						{#if isRotating}
							<Pause class="w-3.5 h-3.5 text-brand-400" />
						{:else}
							<Play class="w-3.5 h-3.5" />
						{/if}
					</button>
				</div>

				<SkinViewer3D
					bind:this={viewerRef}
					skinUrl={currentSkinUrl || "/steve.png"}
					slim={skinType === "alex"}
					cape={selectedCape}
					customCapeUrl={customCapeDataUrl}
					animation={activeAnimation}
					autoRotate={isRotating}
					className="w-full h-full"
				/>
			</div>

			<div class="flex flex-col items-center gap-2.5 w-full">
				<div class="flex items-center gap-1.5 text-xs text-fg/40 font-medium">
					<Move class="w-3.5 h-3.5" />
					<span>Arraste para girar</span>
				</div>

				<div class="flex items-center gap-1 bg-bg/35 backdrop-blur-xl border border-fg/10 rounded-xl p-1 shadow-sm">
					{#each [
						{ id: "idle" as const, label: "Parado" },
						{ id: "walk" as const, label: "Andar" },
						{ id: "run" as const, label: "Correr" },
						{ id: "fly" as const, label: "Voar" }
					] as anim}
						<button
							type="button"
							onclick={() => activeAnimation = anim.id}
							class="px-3 py-1 rounded-lg text-[11px] font-bold transition-all cursor-pointer {activeAnimation === anim.id ? 'bg-brand-500 text-brand-foreground shadow-sm' : 'text-fg/50 hover:text-fg hover:bg-fg/[0.06]'}"
						>
							{anim.label}
						</button>
					{/each}
				</div>

				<div class="flex items-center gap-1 bg-bg/35 backdrop-blur-xl border border-fg/10 rounded-xl p-1 shadow-sm">
					<button
						type="button"
						onclick={() => viewerRef?.setFrontView()}
						class="px-2.5 py-1 rounded-lg text-[10px] font-bold text-fg/60 hover:text-fg hover:bg-fg/[0.06] transition-all cursor-pointer"
						title="Ver de frente"
					>
						Frente
					</button>
					<button
						type="button"
						onclick={() => viewerRef?.setBackView()}
						class="px-2.5 py-1 rounded-lg text-[10px] font-bold text-fg/60 hover:text-fg hover:bg-fg/[0.06] transition-all cursor-pointer"
						title="Ver de costas (capa)"
					>
						Costas
					</button>
					<button
						type="button"
						onclick={() => viewerRef?.setIsometricView()}
						class="px-2.5 py-1 rounded-lg text-[10px] font-bold text-fg/60 hover:text-fg hover:bg-fg/[0.06] transition-all cursor-pointer"
						title="Visão 3D isométrica"
					>
						3D
					</button>
					<button
						type="button"
						onclick={() => viewerRef?.resetCamera()}
						class="p-1 rounded-lg text-fg/50 hover:text-fg hover:bg-fg/[0.06] transition-all cursor-pointer"
						title="Resetar câmera"
					>
						<RotateCcw class="w-3 h-3" />
					</button>
				</div>

				<button
					type="button"
					onclick={() => showEditModal = true}
					class="w-full max-w-[200px] py-2 px-4 rounded-xl bg-bg/35 backdrop-blur-xl hover:bg-fg/10 text-fg border border-fg/10 hover:border-fg/20 text-xs font-bold transition-all flex items-center justify-center gap-2 cursor-pointer shadow-sm active:scale-[0.98]"
				>
					<Pencil class="w-3.5 h-3.5 text-fg/70" />
					<span>Editar skin</span>
				</button>
			</div>


		</div>

		<div class="lg:col-span-8 flex flex-col gap-6">

            <section class="rounded-2xl p-4 space-y-3 bg-bg/35 backdrop-blur-xl border border-fg/10 shadow-sm" aria-label="Capas">
                <div class="flex items-center justify-between gap-3">
                    <h2 class="font-semibold text-fg">Capas</h2>
                    <button class="luxmc-control" onclick={handleCustomCapeUpload}>
                        <Upload class="inline h-4 w-4 mr-2" />Importar PNG
                    </button>
                </div>
                <div class="flex gap-2.5 overflow-x-auto custom-scrollbar pb-2.5 snap-x snap-mandatory scroll-smooth px-0.5">
                    {#each capeList.filter(item => item.id !== "custom") as item}
                        <button class="shrink-0 snap-start w-24 rounded-xl border p-2 text-xs text-fg flex flex-col items-center gap-2 transition-all cursor-pointer {selectedCape === item.id ? 'border-brand-400 bg-brand-500/15 shadow-sm ring-1 ring-brand-500' : 'border-fg/10 bg-fg/[0.03] hover:bg-fg/[0.08]'}" aria-pressed={selectedCape === item.id} onclick={() => selectedCape = item.id}>
                            {#if item.id !== "none"}
                                <img class="h-14 w-9 object-contain [image-rendering:pixelated]" src={getCapePreviewDataUrl(item.id)} alt={item.name} />
                            {:else}
                                <span class="h-14 flex items-center text-fg/40 font-medium">Sem capa</span>
                            {/if}
                            <span class="truncate w-full text-center">{item.name}</span>
                        </button>
                    {/each}
                    {#if customCapeDataUrl}
                        <button class="shrink-0 snap-start w-24 rounded-xl border p-2 text-xs text-fg flex flex-col items-center gap-2 transition-all cursor-pointer {selectedCape === 'custom' ? 'border-brand-400 bg-brand-500/15 shadow-sm ring-1 ring-brand-500' : 'border-fg/10 bg-fg/[0.03] hover:bg-fg/[0.08]'}" aria-pressed={selectedCape === "custom"} onclick={() => selectedCape = "custom"}>
                            {#if customCapePreview}
                                <img class="h-14 w-9 object-contain [image-rendering:pixelated]" src={customCapePreview} alt="Capa importada" />
                            {:else}
                                <span class="h-14 flex items-center text-fg/40 font-medium">PNG</span>
                            {/if}
                            <span class="truncate w-full text-center">Capa importada</span>
                        </button>
                    {/if}
                </div>
            </section>

			<section class="space-y-3">
				<button
					type="button"
					onclick={() => savedSkinsExpanded = !savedSkinsExpanded}
					class="flex items-center gap-2 text-sm font-black text-fg hover:text-brand-400 transition-colors cursor-pointer"
				>
					{#if savedSkinsExpanded}
						<ChevronUp class="w-4 h-4 text-fg/50" />
					{:else}
						<ChevronDown class="w-4 h-4 text-fg/50" />
					{/if}
					<span>Skins salvas</span>
				</button>

				{#if savedSkinsExpanded}
					<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 gap-3.5" transition:slide={{ duration: 150 }}>

						<button
							type="button"
							onclick={handleAddSkinFile}
							class="rounded-2xl border-2 border-dashed border-fg/15 hover:border-brand-500 bg-fg/[0.01] hover:bg-fg/[0.03] p-5 flex flex-col items-center justify-center text-center gap-2 transition-all cursor-pointer min-h-[170px] group shadow-sm"
						>
							<div class="w-10 h-10 rounded-full bg-fg/[0.04] border border-fg/10 group-hover:border-brand-500 flex items-center justify-center text-fg/60 group-hover:text-brand-400 transition-colors">
								<Plus class="w-5 h-5 stroke-[2.5]" />
							</div>
							<div>
								<span class="text-xs font-bold text-fg block">Adicionar skin</span>
								<span class="text-[11px] text-fg/40 block mt-0.5">Arraste e solte</span>
							</div>
						</button>

						{#each savedSkins as s (s.id)}
							{@const isSelected = selectedSkinId === s.id}
							<div
								role="button"
								tabindex="0"
								onclick={() => selectSavedSkin(s)}
								onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") selectSavedSkin(s); }}
								class="rounded-2xl bg-bg/35 backdrop-blur-xl hover:bg-fg/10 border transition-all p-3 flex flex-col items-center justify-between relative cursor-pointer group min-h-[170px] shadow-sm {isSelected ? 'border-brand-500 ring-1 ring-brand-500' : 'border-fg/10 hover:border-fg/20'}"
							>
								{#if isSelected}
									<div class="absolute top-2.5 right-2.5 w-5 h-5 rounded-full bg-black border border-white flex items-center justify-center text-white shadow-md z-10">
										<Check class="w-3 h-3 stroke-[3]" />
									</div>
								{/if}

								<div class="w-full flex-1 flex items-center justify-center my-1 overflow-hidden">
									<Skin2DPreview
										src={s.url}
										alt={s.name}
										model={s.model}
										className="h-28"
									/>
								</div>

								<div class="w-full flex items-center justify-between pt-2 border-t border-fg/[0.04] gap-1">
									{#if editingSkinId === s.id}
										<!-- svelte-ignore a11y_autofocus -->
										<input
											type="text"
											bind:value={editingSkinName}
											onkeydown={(e) => {
												e.stopPropagation();
												if (e.key === 'Enter') saveRenameSkin(s.id);
												if (e.key === 'Escape') editingSkinId = null;
											}}
											onclick={(e) => e.stopPropagation()}
											class="w-full bg-black/40 border border-brand-500 rounded px-1.5 py-0.5 text-xs text-fg outline-none"
											autofocus
											onblur={() => saveRenameSkin(s.id)}
										/>
									{:else}
										<span class="text-xs font-bold text-fg truncate flex-1">{s.name}</span>
										<div class="flex items-center gap-0.5 shrink-0">
											<button
												type="button"
												onclick={(e) => {
													e.stopPropagation();
													startRenameSkin(s);
												}}
												class="text-fg/30 hover:text-brand-400 transition-colors p-1 cursor-pointer"
												title="Renomear skin"
											>
												<Pencil class="w-3 h-3" />
											</button>
											<button
												type="button"
												onclick={(e) => {
													e.stopPropagation();
													removeSavedSkin(s.id);
												}}
												class="text-fg/30 hover:text-red-400 transition-colors p-1 cursor-pointer"
												title="Remover skin salva"
											>
												<Trash2 class="w-3 h-3" />
											</button>
										</div>
									{/if}
								</div>
							</div>
						{/each}
						{#if savedSkins.length === 0}
							<div class="col-span-full min-h-[170px] rounded-2xl border border-fg/10 bg-fg/[0.025] p-6 text-center flex flex-col items-center justify-center gap-2">
								<Shirt class="w-7 h-7 text-brand-400" />
								<p class="text-sm font-bold text-fg">Sua coleção está pronta para começar</p>
								<p class="max-w-sm text-xs text-fg/50">Importe sua primeira skin personalizada em PNG, no modelo Steve ou Alex.</p>
								<button type="button" onclick={handleAddSkinFile} class="mt-1 text-xs font-bold text-brand-400 hover:text-brand-300">Importar skin PNG</button>
							</div>
						{/if}

					</div>
				{/if}
			</section>

		</div>

	</div>


	{#if showEditModal}
		<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md" in:fade={{ duration: 150 }}>
			<div class="w-full max-w-xl rounded-3xl bg-bg/35 backdrop-blur-xl border border-fg/10 p-6 shadow-2xl space-y-5" in:fly={{ y: 20, duration: 200 }}>
				<div class="flex items-center justify-between border-b border-fg/[0.06] pb-3">
					<div class="flex items-center gap-2.5">
						<Pencil class="w-4 h-4 text-brand-400" />
						<h3 class="text-sm font-extrabold text-fg">Editar Configurações da Skin</h3>
					</div>
					<button
						type="button"
						class="text-fg/40 hover:text-fg text-xs cursor-pointer p-1"
						onclick={() => showEditModal = false}
					>
						<X class="w-4 h-4" />
					</button>
				</div>

				<div class="space-y-2">
					<span class="text-xs font-bold text-fg/70 block">Modelo dos Braços</span>
					<div class="grid grid-cols-2 gap-2">
						<button
							type="button"
							onclick={() => setModelType("steve")}
							class="p-3 rounded-2xl border text-left transition-all cursor-pointer {skinType === 'steve' ? 'bg-brand-500/15 border-brand-500 text-brand-400' : 'bg-fg/[0.02] border-fg/10 text-fg/60 hover:text-fg'}"
						>
							<div class="text-xs font-extrabold">Clássico (Steve)</div>
							<div class="text-[10px] opacity-70">Braços normais com 4 pixels</div>
						</button>
						<button
							type="button"
							onclick={() => setModelType("alex")}
							class="p-3 rounded-2xl border text-left transition-all cursor-pointer {skinType === 'alex' ? 'bg-brand-500/15 border-brand-500 text-brand-400' : 'bg-fg/[0.02] border-fg/10 text-fg/60 hover:text-fg'}"
						>
							<div class="text-xs font-extrabold">Fino (Alex)</div>
							<div class="text-[10px] opacity-70">Braços finos com 3 pixels</div>
						</button>
					</div>
				</div>

				<div class="space-y-2">
					<label for="search-nickname-input" class="text-xs font-bold text-fg/70 block">Buscar por Nickname (Mojang / NameMC)</label>
					<div class="flex items-center gap-2">
						<input
							id="search-nickname-input"
							type="text"
							placeholder="Ex: Technoblade, Dream, MumboJumbo..."
							bind:value={searchNick}
							onkeydown={(e) => e.key === 'Enter' && handleSearchNick()}
							class="flex-1 bg-black/40 border border-fg/10 rounded-xl px-3.5 py-2 text-xs text-fg placeholder-fg/30 outline-none focus:border-brand-500"
						/>
						<button
							type="button"
							onclick={handleSearchNick}
							disabled={isSearchingNick}
							class="px-4 py-2 rounded-xl bg-brand-500 hover:bg-brand-600 text-brand-foreground font-black text-xs cursor-pointer disabled:opacity-50"
						>
							{#if isSearchingNick}
								<RefreshCw class="w-3.5 h-3.5 animate-spin" />
							{:else}
								<span>Buscar</span>
							{/if}
						</button>
					</div>
				</div>

				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<span class="text-xs font-bold text-fg/70 block">Escolha de Capa HD</span>
						{#if true}
							<button
								type="button"
								onclick={handleCustomCapeUpload}
								class="text-[11px] font-bold text-brand-400 hover:underline flex items-center gap-1 cursor-pointer"
							>
								<Upload class="w-3 h-3" /> Importar capa PNG
							</button>
						{/if}
					</div>

					<div class="grid grid-cols-3 sm:grid-cols-4 gap-2 max-h-[160px] overflow-y-auto custom-scrollbar pr-1">
						{#each capeList as c}
							{@const isCapeSelected = selectedCape === c.id}
							{@const previewUrl = c.id !== "none" && c.id !== "custom" ? getCapePreviewDataUrl(c.id) : null}
							<button
								type="button"
								onclick={() => {
									if (c.id === "custom" && !customCapeDataUrl) {
										handleCustomCapeUpload();
									} else {
										selectedCape = c.id;
									}
								}}
								class="p-2 rounded-xl border transition-all cursor-pointer flex flex-col items-center text-center gap-1 relative overflow-hidden {isCapeSelected ? 'bg-brand-500/15 border-brand-500' : 'bg-fg/[0.02] border-fg/10 hover:border-fg/20'}"
							>
								<div class="w-8 h-12 rounded-lg bg-black/40 border border-fg/10 flex items-center justify-center overflow-hidden shrink-0 shadow-inner">
									{#if previewUrl}
										<img
											src={previewUrl}
											alt={c.name}
											class="w-full h-full object-contain [image-rendering:pixelated]"
										/>
									{:else if c.id === "custom" && customCapeDataUrl}
										<img
											src={customCapePreview || customCapeDataUrl}
											alt="Capa personalizada"
											class="w-full h-full object-cover [image-rendering:pixelated]"
										/>
									{:else if c.id === "custom"}
										<Upload class="w-4 h-4 text-fg/50" />
									{:else}
										<span class="text-[9px] text-fg/30 font-bold uppercase">Sem capa</span>
									{/if}
								</div>
								<span class="text-[10px] font-bold truncate block text-fg w-full">{c.name}</span>
							</button>
						{/each}
					</div>
				</div>

				<div class="flex items-center justify-end gap-2 pt-2 border-t border-fg/[0.06]">
					<button
						type="button"
						onclick={() => showEditModal = false}
						class="px-5 py-2 rounded-xl bg-brand-500 hover:bg-brand-600 text-brand-foreground text-xs font-black cursor-pointer"
					>
						Pronto
					</button>
				</div>
			</div>
		</div>
	{/if}

</div>
