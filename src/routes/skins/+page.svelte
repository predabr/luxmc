<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import Button from "$lib/components/ui/Button.svelte";
    import NameMcPicker from "$lib/components/skins/NameMcPicker.svelte";
    import type { NameMcSkin } from "$lib/api/namemc";
	import { backOut, quintOut } from "svelte/easing";
    import { skinAvatar, createSkinAvatar, inferSkinModelType, classifyTexture, textureCanvas } from "$lib/utils/textureImage";
	import { onDestroy, onMount, untrack, tick } from "svelte";
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
		Shirt,
        Globe2,
        ArrowUpRight
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

	let syncError = $state("");
	let skinType = $state<"steve" | "alex">("steve");
	let isRotating = $state(true);
	let activeAnimation = $state<"idle" | "walk" | "run" | "fly" | "none">("walk");
	let viewerRef = $state<SkinViewer3D | null>(null);

	let searchNick = $state("");
	let isSearchingNick = $state(false);
	let selectedCape = $state<CapeType>("none");
    let showNameMc = $state(false);
    let nameMcPicker = $state<NameMcPicker | null>(null);
	let customCapeDataUrl = $state("");
    let customCapePreview = $state("");
	let showEditModal = $state(false);
	let appliedSelection = $state("");
	let hydrated = $state(false);
    let draftOwner = $state("");
	let saving = $state(false);
	let saveError = $state("");

	let savedSkinsExpanded = $state(true);
	type SavedSkinItem = { id: string; name: string; url: string; model: "steve" | "alex"; capeType?: CapeType; capeUrl?: string };
	const defaultSavedSkins: SavedSkinItem[] = [];
	let savedSkins = $state<SavedSkinItem[]>(defaultSavedSkins);

	let selectedSkinId = $state("");
	let selectedSkinNick = $state(uiText("ui.0419454263a82604"));
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
    const canApplyAppearance = $derived(hydrated && Boolean(account.value && currentSkinUrl) && !saving);

    const capeTypes: readonly CapeType[] = ["none", "custom", "luxmc", "optifine", "migrator", "cherry", "vanilla", "minecon2011", "minecon2012", "minecon2013", "minecon2015", "minecon2016", "tiktok", "twitch"];
    const currentCapePreview = $derived(selectedCape === "none" ? "" : selectedCape === "custom" ? customCapePreview : getCapePreviewDataUrl(selectedCape));
    const accountCapeUrl = $derived(account.value?.capeUrl || "");

	function hydrateAppearance() {
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
				if (!account.value?.skinVariant && !selectedSkinId && previewSkinUrl === initialSkin) {
					skinType = inferred;
				}
			}).catch(() => {});
		}
		let persistedSkinId = "";
		try {
			persistedSkinId = localStorage.getItem(`luxmc_selected_skin_id:${account.value?.id || "local"}`) || localStorage.getItem("luxmc_selected_skin_id") || "";
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
				selectedSkinNick = uiText("ui.0419454263a82604");
			}
			}
		}
        appliedSelection = JSON.stringify([appliedAccountSkin, appliedAccountModel, appliedAccountCape, appliedAccountCapeUrl]);
        draftOwner = account.value?.id || "local";
        try {
            const savedApplication = JSON.parse(localStorage.getItem(`luxmc_applied_skin_selection:${draftOwner}`) || "null");
            if (savedApplication?.accountAppearance === accountAppearance && Array.isArray(savedApplication.selection)) {
                appliedSelection = JSON.stringify(savedApplication.selection);
                const [skin, model, cape, capeUrl] = savedApplication.selection;
                if ((savedApplication.skinId === selectedSkinId || skin === previewSkinUrl) && typeof skin === "string" &&
                    (model === "steve" || model === "alex") && capeTypes.includes(cape) && typeof capeUrl === "string") {
                    previewSkinUrl = skin; skinType = model; selectedCape = cape; customCapeDataUrl = capeUrl;
                }
            }
        } catch {}
        try {
            const draft = JSON.parse(sessionStorage.getItem(`luxmc_skin_draft:${draftOwner}`) || "null");
            if (draft && typeof draft.skinUrl === "string" && /^(data:image\/png;base64,|https:\/\/|\/)/.test(draft.skinUrl) &&
                (draft.model === "steve" || draft.model === "alex") && capeTypes.includes(draft.cape) &&
                typeof draft.customCapeUrl === "string" && typeof draft.id === "string" && typeof draft.name === "string" && savedSkins.some(saved => saved.id === draft.id)) {
                previewSkinUrl = draft.skinUrl;
                skinType = draft.model;
                selectedCape = draft.cape;
                customCapeDataUrl = draft.customCapeUrl;
                selectedSkinId = draft.id;
                selectedSkinNick = draft.name;
            }
        } catch {}
        const pairedSkin = savedSkins.find(skin => skin.id === selectedSkinId);
        if (pairedSkin?.capeType && capeTypes.includes(pairedSkin.capeType)) {
            selectedCape = pairedSkin.capeType;
            customCapeDataUrl = pairedSkin.capeUrl || "";
        }
        hydrated = true;
	}
    onMount(hydrateAppearance);
    $effect(() => {
        const id = account.value?.id || "local";
        if (hydrated && draftOwner !== id) untrack(hydrateAppearance);
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
        if (!hydrated || !selectedSkinId) return;
        const id = selectedSkinId;
        const cape = selectedCape;
        const url = cape === "custom" ? customCapeDataUrl : "";
        untrack(() => {
            const skin = savedSkins.find(item => item.id === id);
            if (!skin || (skin.capeType === cape && skin.capeUrl === url)) return;
            savedSkins = savedSkins.map(item => item.id === id ? { ...item, capeType: cape, capeUrl: url } : item);
            try { localStorage.setItem("luxmc_saved_skins", JSON.stringify(savedSkins)); } catch {}
        });
    });

    async function openNameMcCatalog() {
        showNameMc = true;
        await tick();
        await nameMcPicker?.browse();
    }

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
                toast(uiText("ui.14a0c4f82ef22bf5"), "success");
                return;
            }
            selectValidatedSkin(skin, (skin.model as string === "alex" || skin.model as string === "slim") ? "alex" : (skin.model as string === "steve" || skin.model as string === "classic") ? "steve" : inferSkinModelType(canvas));
        } catch (error) {
            if (!controller.signal.aborted) toast(uiText("ui.787f24e28eef8258") + String(error), "error");
        }
    }

    function selectValidatedSkin(skin: SavedSkinItem, model: "steve" | "alex") {
        selectedSkinId = skin.id;
        selectedSkinNick = skin.name;
        skinType = model;
        previewSkinUrl = skin.url;
        selectedCape = skin.capeType && capeTypes.includes(skin.capeType) ? skin.capeType : "none";
        customCapeDataUrl = selectedCape === "custom" ? skin.capeUrl || "" : "";
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
				selectedSkinNick = uiText("ui.0419454263a82604");
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

    async function importNameMcSkin(skin: NameMcSkin) {
        importController?.abort();
        const controller = new AbortController();
        importController = controller;
        const image = await loadTextureImage(skin.skinUrl, controller.signal);
        if (controller.signal.aborted) return;
        const canvas = textureCanvas(image);
        if (classifyTexture(canvas) !== "skin") throw new Error(uiText("ui.4b23da27cc1f6901"));
        const model = inferSkinModelType(canvas);
        const existing = savedSkins.find(item => item.url === skin.skinUrl);
        const saved: SavedSkinItem = existing || { id: crypto.randomUUID(), name: `NameMC ${skin.skinId}`, url: skin.skinUrl, model };
        if (!existing) {
            const next = [saved, ...savedSkins];
            localStorage.setItem("luxmc_saved_skins", JSON.stringify(next));
            savedSkins = next;
        }
        selectValidatedSkin(saved, model);
        toast(uiText("skinsStudio.nameMcImported"), "success");
    }

    let importController: AbortController | null = null;
    onMount(() => () => importController?.abort());
    async function handleAddSkinFile() {
        try {
            const selected = await open({ multiple: false, filters: [{ name: uiText("ui.74f0c431111d3350"), extensions: ["png", "webp", "jpeg", "jpg"] }] });
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
                toast(uiText("ui.14a0c4f82ef22bf5"), "success");
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
            toast(`Skin "${name}" adicionada (${model === "alex" ? "Fino / Alex" : uiText("ui.0c681eeca1e08ff9")})!`, "success");
        } catch (error) {
            if (!(error instanceof DOMException && error.name === "AbortError")) toast(uiText("ui.97bfa663aaafe580") + String(error), "error");
        }
    }

    let searchController: AbortController | null = null;
    onMount(() => () => searchController?.abort());
    async function handleSearchNick() {
        const nick = searchNick.trim();
        if (!/^[A-Za-z0-9_]{3,16}$/.test(nick)) { toast(uiText("ui.c5679f576d780243"), "info"); return; }
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
				filters: [{ name: uiText("ui.1ffd2b659842ed2b"), extensions: ["png", "webp", "jpeg", "jpg"] }]
			});
			if (!selected || typeof selected !== "string") return;

            importController?.abort();
            const controller = new AbortController();
            importController = controller;
            const image = await loadTextureImage(await authReadLocalTexture(selected), controller.signal);
            customCapeDataUrl = normalizeCape(image).toDataURL("image/png");
			selectedCape = "custom";
			toast(uiText("ui.97e744321263c71e"), "success");
		} catch (e) {
			toast(uiText("ui.b57990d4f3de61f4") + String(e), "error");
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
        if (!canApplyAppearance) return;
        const applied = selection;
        const owner = account.value?.id;
        const source = currentSkinUrl;
        const model = skinType;
        const cape = selectedCape;
        const capeSource = cape === "custom" ? customCapeDataUrl : cape === "none" ? "" : getFullCapeDataUrl(cape);
        const controller = new AbortController();
        saving = true;
        saveError = "";
        syncError = "";
        const normalize = async (url: string, isCape = false) => {
            if (url.startsWith("https://")) url = await authResolveTexture(url);
            const image = await loadTextureImage(url, controller.signal);
            if (isCape) return normalizeCape(image).toDataURL("image/png");
            const canvas = document.createElement("canvas");
            canvas.width = image.naturalWidth;
            canvas.height = image.naturalHeight;
            if (canvas.width > 2048 || canvas.height > 2048) throw new Error("Textura muito grande");
            const ctx = canvas.getContext("2d");
            if (!ctx) throw new Error(uiText("ui.facb67b092a4f99b"));
            ctx.drawImage(image, 0, 0);
            if (classifyTexture(canvas) !== "skin") throw new Error(uiText("ui.4b23da27cc1f6901"));
            return url.startsWith("data:image/png;base64,") ? url : canvas.toDataURL("image/png");
        };
        await Promise.all([normalize(source), capeSource ? normalize(capeSource, true) : Promise.resolve(null)]).then(async ([skin, capeUrl]) => {
            if (controller.signal.aborted) return;
            const avatarUrl = await createSkinAvatar(skin, controller.signal, model);
            if (controller.signal.aborted) return;
            await saveAppearance(skin, model === "alex" ? "slim" : "classic", capeUrl, avatarUrl);
            if (account.value?.id !== owner) throw new Error(uiText("ui.10552831b711a961"));
            if (controller.signal.aborted) return;
            appliedSelection = applied;
            try {
                localStorage.setItem(`luxmc_applied_skin_selection:${account.value?.id || "local"}`, JSON.stringify({
                    accountAppearance: JSON.stringify([account.value?.skinUrl || "", account.value?.skinVariant || "", account.value?.capeUrl || ""]),
                    skinId: selectedSkinId,
                    selection: JSON.parse(applied)
                }));
            } catch {}
            activeSkinStore.setSkin({ id: selectedSkinId, name: selectedSkinNick, avatarUrl, skinUrl: skin, type: model, capeType: cape, hasCape: cape !== "none", customCapeUrl: capeUrl || "" });
            try {
                localStorage.setItem("luxmc_selected_skin_id", selectedSkinId);
                localStorage.setItem(`luxmc_selected_skin_id:${account.value?.id || "local"}`, selectedSkinId);
                localStorage.setItem("luxmc_selected_skin_nick", selectedSkinNick);
            } catch {}
            if (isMicrosoft && account.value) {
                try {
                    await authChangeSkin(account.value.id, model === "alex" ? "slim" : "classic", skin);
                    toast(uiText("ui.5ee886e3e6e1e5b9"), "success");
                } catch (error) {
                    syncError = uiText("ui.ae2e64d560aac426", {arg0: (String(error))});
                    toast(syncError, "error");
                }
            } else {
                toast(uiText("ui.74dcc7a5ad4bea2f"), "success");
            }
        }).catch(error => {
            if (!controller.signal.aborted) { saveError = String(error); toast(saveError, "error"); }
        }).finally(() => { if (!controller.signal.aborted) saving = false; });
    }

	async function handleSyncWithWeb(): Promise<void> {
		const nick = selectedSkinNick || username;
		const webUrl = `https://luxmc-r92.pages.dev/skins.html?nick=${encodeURIComponent(nick)}&model=${skinType}&cape=${selectedCape}`;
		try {
			await openUrl(webUrl);
			toast(uiText("ui.9824b68a0ca569df"), "info");
		} catch (error) {
			toast(String(error), "error");
		}
	}
</script>

<div class="h-full flex flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-24">

	<header class="flex items-center justify-between gap-4 py-1">
		<div class="flex items-center gap-3">
			<div class="flex items-center gap-1 bg-bg-elevated border border-fg/10 rounded-xl p-1 shadow-sm">
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					title={uiText("common.back")}
					onclick={() => history.back()}
				>
					<ArrowLeft class="w-3.5 h-3.5" />
				</button>
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					title={uiText("ui.859e48c3c9333657")}
					onclick={() => history.forward()}
				>
					<ChevronRight class="w-3.5 h-3.5" />
				</button>
			</div>

			<div class="flex items-center gap-2 text-xs font-bold text-fg/80">
				<Shirt class="w-3.5 h-3.5 text-fg/60" />
				<span class="text-fg font-extrabold">{uiText("ui.786aa1f40fcdff95")}</span>
			</div>
		</div>

		<div class="flex items-center gap-2.5">
			<button
				type="button"
				onclick={handleSyncWithWeb}
				class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
				title={uiText("ui.596fb3bb01539cb0")}
			>
				<Share2 class="w-3.5 h-3.5 text-brand-400" />
				<span>{uiText("ui.d812f36fd857abac")}</span>
			</button>
		</div>
	</header>

    <div class="sticky top-0 z-20 shrink-0 flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-fg/10 bg-bg-elevated p-4 shadow-elevated backdrop-blur-xl">
        <p class="text-sm text-fg-muted" role="status">{saving ? isMicrosoft ? uiText("skinsStudio.syncing") : uiText("ui.c96a5ffa4c9a8e9c") : saveError ? uiText("ui.f3e4295a34844bee") + saveError : syncError || (hasPendingChange ? uiText("ui.69e9fbeba0d02d87") : uiText("ui.0d8e3ff973d2531e"))}</p>
        <div class="flex gap-2">
            <Button variant="primary" size="lg" loading={saving} disabled={!canApplyAppearance} onclick={applyAppearance}>{saving ? uiText("skinsStudio.applying") : isMicrosoft ? uiText("ui.96d74ee46d837a1f") : uiText("ui.24e056555b501d83")}</Button>
        </div>
    </div>
    <p class="text-sm text-fg-muted rounded-2xl border border-fg/10 bg-bg-elevated p-4"><Info class="inline h-4 w-4 mr-2 text-brand-400" />{isMicrosoft ? uiText("ui.bd7f21aa535401a7") : account.value?.id.startsWith("luxmc:") ? uiText("profileCard.luxmcSkins") : uiText("profileCard.offlineSkins")}</p>


	<div class="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">

		<div class="lg:col-span-4 flex flex-col items-center gap-4 lg:sticky lg:top-2">

			<div class="skin-preview-stage w-full h-[520px] rounded-3xl bg-bg-elevated border border-fg/10 relative overflow-hidden shadow-2xl flex flex-col items-center justify-center p-4">

				<div class="absolute top-3 left-3 right-3 flex items-center justify-end z-10 pointer-events-none">

					<button
						type="button"
						onclick={() => isRotating = !isRotating}
						class={launcherButton({ variant: "secondary", size: "sm", class: "pointer-events-auto" })}
						title={isRotating ? uiText("ui.6ed3c3a7bd4a365f") : uiText("ui.e7d44cc64970d022")}
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
					<span>{uiText("ui.7d6a6d773e0cd710")}</span>
				</div>

				<div class="flex items-center gap-1 bg-bg-elevated border border-fg/10 rounded-xl p-1 shadow-sm">
					{#each [
						{ id: "idle" as const, label: uiText("ui.31e8729fef873dd7") },
						{ id: "walk" as const, label: uiText("ui.e3691dd454b50eb6") },
						{ id: "run" as const, label: uiText("ui.f58a67a713c98dd0") },
						{ id: "fly" as const, label: uiText("ui.011029a4827dc053") }
					] as anim}
						<button
							type="button"
							onclick={() => activeAnimation = anim.id}
							class={launcherButton({ variant: activeAnimation === anim.id ? "primary" : "ghost", size: "sm" })} aria-pressed={activeAnimation === anim.id}
						>
							{anim.label}
						</button>
					{/each}
				</div>

				<div class="flex items-center gap-1 bg-bg-elevated border border-fg/10 rounded-xl p-1 shadow-sm">
					<button
						type="button"
						onclick={() => viewerRef?.setFrontView()}
						class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
						title={uiText("ui.ce1785248bf1aa9e")}
					>
						{uiText("ui.ba711a7a53056618")}
					</button>
					<button
						type="button"
						onclick={() => viewerRef?.setBackView()}
						class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
						title={uiText("ui.bddf50fe294ea9a8")}
					>
						{uiText("ui.357ac3567b470c10")}
					</button>
					<button
						type="button"
						onclick={() => viewerRef?.setIsometricView()}
						class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
						title={uiText("ui.3618d6380d8c5b40")}
					>
						3D
					</button>
					<button
						type="button"
						onclick={() => viewerRef?.resetCamera()}
						class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
						title={uiText("ui.3fdd339b0d171ddb")}
					>
						<RotateCcw class="w-3 h-3" />
					</button>
				</div>

				<button
					type="button"
					onclick={() => showEditModal = true}
					class={launcherButton({ variant: "secondary", size: "sm", class: "w-full max-w-[200px] flex items-center justify-center gap-2" })}
				>
					<Pencil class="w-3.5 h-3.5 text-fg/70" />
					<span>{uiText("skinsStudio.model")}</span>
				</button>
			</div>


		</div>

		<div class="lg:col-span-8 flex flex-col gap-6">

            <section class="rounded-2xl p-5 space-y-4 bg-bg-elevated border border-fg/10 shadow-sm" aria-label={uiText("ui.cd41a52072c213e4")}>
                <div class="flex flex-wrap items-center justify-between gap-4">
                    <div>
                        <h2 class="font-semibold text-fg">{uiText("ui.cd41a52072c213e4")}</h2>
                        <p class="mt-1 text-xs leading-relaxed text-fg-muted">{uiText("skinsStudio.capeDescription")}</p>
                    </div>
                    <Button variant="secondary" size="xl" onclick={handleCustomCapeUpload}><Upload class="h-5 w-5" />{uiText("skinsStudio.addCape")}</Button>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    {#if selectedCape !== "none"}
                        <div class="flex items-center gap-3 rounded-xl border border-fg/10 bg-fg/[0.03] px-4 py-3">
                            {#if currentCapePreview}<img class="h-14 w-9 object-contain [image-rendering:pixelated]" src={currentCapePreview} alt={uiText("skinsStudio.currentCape")} />{/if}
                            <span class="text-sm font-medium text-fg">{uiText("skinsStudio.currentCape")}</span>
                        </div>
                    {/if}
                    <Button variant={selectedCape === "none" ? "outline" : "ghost"} size="lg" aria-pressed={selectedCape === "none"} onclick={() => selectedCape = "none"}>{uiText("skinsStudio.removeCape")}</Button>
                    {#if customCapeDataUrl && selectedCape !== "custom"}
                        <Button variant="secondary" size="lg" onclick={() => selectedCape = "custom"}>{uiText("ui.b0dcf66f807f8b19")}</Button>
                    {/if}
                    {#if accountCapeUrl && accountCapeUrl !== customCapeDataUrl}
                        <Button variant="secondary" size="lg" onclick={() => { customCapeDataUrl = accountCapeUrl; selectedCape = "custom"; }}>{uiText("skinsStudio.accountCape")}</Button>
                    {/if}
                </div>
            </section>

            <div class="rounded-2xl border border-brand-400/20 bg-brand-500/[0.06] p-5 flex flex-col gap-4 sm:flex-row sm:items-center">
                <div class="min-w-0 flex-1">
                    <h2 class="font-semibold text-fg">{uiText("skinsStudio.collection")}</h2>
                    <p class="mt-1 text-sm leading-relaxed text-fg-muted">{uiText("skinsStudio.collectionDescription")}</p>
                </div>
                <Button variant="ghost" size="lg" onclick={async () => { showNameMc = true; await tick(); nameMcPicker?.showLinkImport(); }}>{uiText("skinsStudio.nameMcLink")}</Button>
                <Button variant="primary" size="xl" onclick={openNameMcCatalog}><Globe2 class="h-5 w-5" />{uiText("skinsStudio.nameMcBrowse")}<ArrowUpRight class="h-4 w-4" /></Button>
            </div>

			<section class="space-y-3">
				<button
					type="button"
					onclick={() => savedSkinsExpanded = !savedSkinsExpanded}
					class={launcherButton({ variant: "ghost", size: "sm", class: "flex items-center gap-2" })}
				>
					{#if savedSkinsExpanded}
						<ChevronUp class="w-4 h-4 text-fg/50" />
					{:else}
						<ChevronDown class="w-4 h-4 text-fg/50" />
					{/if}
					<span>{uiText("ui.a28ec08e13c3486e")}</span>
				</button>

				{#if savedSkinsExpanded}
					<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 gap-3.5" transition:slide={{ easing: quintOut, duration: 220 }}>

						<button
							type="button"
							onclick={handleAddSkinFile}
							class="rounded-2xl border-2 border-dashed border-fg/15 hover:border-brand-500 bg-fg/[0.01] hover:bg-fg/[0.03] p-6 flex flex-col items-center justify-center text-center gap-3 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer min-h-[200px] group shadow-sm"
						>
							<div class="w-10 h-10 rounded-full bg-fg/[0.04] border border-fg/10 group-hover:border-brand-500 flex items-center justify-center text-fg/60 group-hover:text-brand-400 transition-colors">
								<Plus class="w-5 h-5 stroke-[2.5]" />
							</div>
							<div>
								<span class="text-sm font-bold text-fg block">{uiText("ui.4c9f7bf72839140b")}</span>
								<span class="text-[11px] text-fg/40 block mt-0.5">{uiText("ui.bff1b507008c4ae1")}</span>
							</div>
						</button>

						{#each savedSkins as s (s.id)}
							{@const isSelected = selectedSkinId === s.id}
							<div
								role="button"
								tabindex="0"
								onclick={() => selectSavedSkin(s)}
								onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") selectSavedSkin(s); }}
								class="rounded-2xl bg-bg-elevated hover:bg-fg/10 border transition-[color,background-color,border-color,box-shadow,transform,opacity] p-3 flex flex-col items-center justify-between relative cursor-pointer group min-h-[170px] shadow-sm {isSelected ? 'border-brand-500 ring-1 ring-brand-500' : 'border-fg/10 hover:border-fg/20'}"
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
												class={launcherButton({ variant: "ghost", size: "icon", class: "" })}
												title={uiText("ui.a8a60a5f8ce44324")}
											>
												<Pencil class="w-3 h-3" />
											</button>
											<button
												type="button"
												onclick={(e) => {
													e.stopPropagation();
													removeSavedSkin(s.id);
												}}
												class={launcherButton({ variant: "danger", size: "icon", class: "" })}
												title={uiText("ui.02a395e8e81169aa")}
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
								<p class="text-sm font-bold text-fg">{uiText("ui.ed03cfca46dfa4bd")}</p>
								<p class="max-w-sm text-xs text-fg/50">{uiText("ui.1447b4b57ec95585")}</p>
								<button type="button" onclick={handleAddSkinFile} class={launcherButton({ variant: "ghost", size: "sm", class: "mt-1" })}>{uiText("ui.c59a739c055d2700")}</button>
							</div>
						{/if}

					</div>
				{/if}
			</section>

		</div>

	</div>


    <NameMcPicker bind:this={nameMcPicker} isOpen={showNameMc} onClose={() => showNameMc = false} onImport={importNameMcSkin} onImportFile={handleAddSkinFile} />

	{#if showEditModal}
		<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md" in:fade={{ easing: quintOut, duration: 220 }}>
			<div class="w-full max-w-xl rounded-3xl bg-bg-elevated border border-fg/10 p-6 shadow-2xl space-y-5" in:fly={{ easing: backOut, y: 20, duration: 260 }}>
				<div class="flex items-center justify-between border-b border-fg/[0.06] pb-3">
					<div class="flex items-center gap-2.5">
						<Pencil class="w-4 h-4 text-brand-400" />
						<h3 class="text-sm font-extrabold text-fg">{uiText("ui.71d4b9b16931a1e6")}</h3>
					</div>
					<button
						type="button"
						class={launcherButton({ variant: "ghost", size: "icon", class: "" })}
						onclick={() => showEditModal = false}
					>
						<X class="w-4 h-4" />
					</button>
				</div>

				<div class="space-y-2">
					<span class="text-xs font-bold text-fg/70 block">{uiText("ui.cba1dbb98b38725c")}</span>
					<div class="grid grid-cols-2 gap-2">
						<button
							type="button"
							onclick={() => setModelType("steve")}
							class="p-3 rounded-2xl border text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {skinType === 'steve' ? 'bg-brand-500/15 border-brand-500 text-brand-400' : 'bg-fg/[0.02] border-fg/10 text-fg/60 hover:text-fg'}"
						>
							<div class="text-xs font-extrabold">{uiText("ui.96a64cbea1a373fc")}</div>
							<div class="text-[10px] opacity-70">{uiText("ui.1ff20d38849b2834")}</div>
						</button>
						<button
							type="button"
							onclick={() => setModelType("alex")}
							class="p-3 rounded-2xl border text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {skinType === 'alex' ? 'bg-brand-500/15 border-brand-500 text-brand-400' : 'bg-fg/[0.02] border-fg/10 text-fg/60 hover:text-fg'}"
						>
							<div class="text-xs font-extrabold">{uiText("ui.d3371d4701fa915a")}</div>
							<div class="text-[10px] opacity-70">{uiText("ui.9d48e239645d29a3")}</div>
						</button>
					</div>
				</div>

				<div class="space-y-2">
					<label for="search-nickname-input" class="text-xs font-bold text-fg/70 block">{uiText("ui.2482dbad375ed2ad")}</label>
					<div class="flex items-center gap-2">
						<input
							id="search-nickname-input"
							type="text"
							placeholder={uiText("ui.745ee544a80d6267")}
							bind:value={searchNick}
							onkeydown={(e) => e.key === 'Enter' && handleSearchNick()}
							class="flex-1 bg-black/40 border border-fg/10 rounded-xl px-3.5 py-2 text-xs text-fg placeholder-fg/30 outline-none focus:border-brand-500"
						/>
						<button
							type="button"
							onclick={handleSearchNick}
							disabled={isSearchingNick}
							class={launcherButton({ variant: "primary", size: "sm", class: "disabled:opacity-50" })}
						>
							{#if isSearchingNick}
								<RefreshCw class="w-3.5 h-3.5 animate-spin" />
							{:else}
								<span>{uiText("mods.searchButton")}</span>
							{/if}
						</button>
					</div>
				</div>

				<div class="flex items-center justify-end gap-2 pt-2 border-t border-fg/[0.06]">
					<button
						type="button"
						onclick={() => showEditModal = false}
						class={launcherButton({ variant: "primary", size: "sm", class: "" })}
					>
						{uiText("ui.039eb59231cce07c")}
					</button>
				</div>
			</div>
		</div>
	{/if}

</div>
