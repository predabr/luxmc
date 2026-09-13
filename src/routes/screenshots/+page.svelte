<script lang="ts">
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import Modal from "$lib/components/ui/Modal.svelte";
	import { Camera, X, FolderOpen, RefreshCw, Copy, Download, Trash2, ZoomIn, ZoomOut, RotateCcw } from "lucide-svelte";
	import { fade } from "svelte/transition";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { instancesScreenshots, screenshotDelete, screenshotsOpenFolder } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

	let screenshots = $state<Array<{ name: string; path: string; modified: string; dataUrl?: string | null }>>([]);
	let loading = $state(false);
	let selectedImage = $state<{ name: string; path: string; modified: string; dataUrl?: string | null } | null>(null);
	let screenshotToDelete = $state<{ name: string; path: string } | null>(null);
	let showDeleteConfirm = $state(false);
	let deleting = $state(false);

	async function loadScreenshots() {
		if (!profiles.active) return;
		loading = true;
		try {
			screenshots = await instancesScreenshots(profiles.active.id);
		} catch (e) {
			screenshots = [];
			toast(t("screenshots.loadFailed", { error: String(e) }), "error");
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (profiles.active) {
			loadScreenshots();
		} else {
			screenshots = [];
		}
	});

	async function handleOpenFolder() {
		if (!profiles.active) return;
		try {
			await screenshotsOpenFolder(profiles.active.id);
		} catch (e) {
			toast(t("screenshots.openFolderFailed", { error: String(e) }), "error");
		}
	}

	let zoom = $state(1);

	function zoomIn() {
		zoom = Math.min(3, +(zoom + 0.25).toFixed(2));
	}

	function zoomOut() {
		zoom = Math.max(0.5, +(zoom - 0.25).toFixed(2));
	}

	function resetZoom() {
		zoom = 1;
	}

	function openImage(screenshot: { name: string; path: string; modified: string }) {
		selectedImage = screenshot;
		resetZoom();
	}

	$effect(() => {
		if (!selectedImage) return;
		function onKey(e: KeyboardEvent) {
			if (e.key === "Escape") {
				selectedImage = null;
			} else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "c") {
				e.preventDefault();
				copyImage(selectedImage!);
			} else if (e.key === "+" || e.key === "=") {
				zoomIn();
			} else if (e.key === "-") {
				zoomOut();
			}
		}
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	});

	function closeModal() {
		selectedImage = null;
	}

	async function copyImage(screenshot: { name: string; path: string }) {
		try {
			const res = await fetch(convertFileSrc(screenshot.path));
			const blob = await res.blob();
			const pngBlob = blob.type === "image/png" ? blob : new Blob([blob], { type: "image/png" });
			await navigator.clipboard.write([
				new ClipboardItem({ "image/png": pngBlob })
			]);
			toast(t("screenshots.copied"), "success");
		} catch (e) {
			toast(t("screenshots.copyFailed", { error: String(e) }), "error");
		}
	}

	function exportImage(screenshot: { name: string; path: string }) {
		const a = document.createElement("a");
		a.href = convertFileSrc(screenshot.path);
		a.download = screenshot.name;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		toast(t("screenshots.exported"), "success");
	}

	function promptDelete(screenshot: { name: string; path: string }, e?: Event) {
		e?.stopPropagation();
		screenshotToDelete = screenshot;
		showDeleteConfirm = true;
	}

	async function executeDelete() {
		if (!screenshotToDelete) return;
		deleting = true;
		try {
			await screenshotDelete(screenshotToDelete.path);
			screenshots = screenshots.filter((s) => s.path !== screenshotToDelete!.path);
			if (selectedImage?.path === screenshotToDelete.path) {
				selectedImage = null;
			}
			toast(t("screenshots.deleted"), "success");
			showDeleteConfirm = false;
			screenshotToDelete = null;
		} catch (e) {
			toast(t("screenshots.deleteFailed", { error: String(e) }), "error");
		} finally {
			deleting = false;
		}
	}

	function confirmDelete(s: { name: string; path: string; modified: string }) {
		screenshotToDelete = s;
		showDeleteConfirm = true;
	}
	async function deleteImage() {
		if (!screenshotToDelete || !profiles.active) return;
		deleting = true;
		try {
			await screenshotDelete(screenshotToDelete.path);
			showDeleteConfirm = false;
			selectedImage = null;
			await loadScreenshots();
			toast(t("screenshots.deleteSuccess"), "success");
		} catch (e) {
			toast(t("screenshots.deleteFailed", { error: String(e) }), "error");
		} finally {
			deleting = false;
		}
	}
	function copyToClipboard(path: string) {
		navigator.clipboard.writeText(path)
			.then(() => {
				toast("Caminho copiado para a área de transferência!", "success");
			})
			.catch((e) => {
				toast("Falha ao copiar caminho: " + String(e), "error");
			});
	}
	async function openFolder() {
		if (profiles.active) {
			try {
				await screenshotsOpenFolder(profiles.active.id);
			} catch (e) {
				toast("Could not open folder", "error");
			}
		}
	}
</script>

<div class="flex h-full flex-col gap-6">
	<div class="flex items-center justify-between">
		<div class="flex flex-col"><Heading>{t("screenshots.title")}</Heading><p class="text-sm text-fg-subtle">{t("screenshots.subtitle")}</p></div>
		<div class="flex items-center gap-3">
			<Button variant="secondary" onclick={loadScreenshots} disabled={loading}>
				<RefreshCw class="h-4 w-4 {loading ? 'animate-spin' : ''}" />
				<span class="hidden sm:inline">{t("screenshots.refreshBtn")}</span>
			</Button>
			<Button variant="solid" onclick={openFolder} disabled={!profiles.active}>
				<FolderOpen class="h-4 w-4" />
				<span class="hidden sm:inline">{t("screenshots.openFolderBtn")}</span>
			</Button>
		</div>
	</div>

	{#if !profiles.active}
		<Card class="flex flex-col items-center justify-center py-20 text-center luxmc-glass">
			<div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-brand-500/10">
				<Camera class="h-8 w-8 text-brand-400" />
			</div>
			<h3 class="mb-2 text-lg font-bold text-fg">{t("screenshots.noInstanceSelected")}</h3>
			<p class="max-w-md text-sm text-fg-subtle">{t("screenshots.selectInstanceHint")}</p>
		</Card>
	{:else if loading && screenshots.length === 0}
		<div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
			{#each Array(8) as _}
				<div class="aspect-video w-full animate-pulse rounded-2xl bg-white/5 luxmc-glass border border-white/5"></div>
			{/each}
		</div>
	{:else if screenshots.length === 0}
		<Card class="flex flex-col items-center justify-center py-20 text-center luxmc-glass">
			<div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-white/5">
				<Camera class="h-8 w-8 text-fg-subtle" />
			</div>
			<h3 class="mb-2 text-lg font-bold text-fg">{t("screenshots.noScreenshotsFound")}</h3>
			<p class="max-w-md text-sm text-fg-subtle">{t("screenshots.takeSomeHint")}</p>
		</Card>
	{:else}
		<div class="columns-2 md:columns-3 lg:columns-4 gap-4 space-y-4">
			{#each screenshots as s}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="group relative overflow-hidden rounded-2xl bg-[#141518] border border-white/5 cursor-zoom-in break-inside-avoid shadow-lg transition-all hover:border-brand-500/30 hover:shadow-brand-500/5" onclick={() => selectedImage = s}>
					<img src={s.dataUrl || convertFileSrc(s.path)} alt={s.name} class="w-full h-auto object-cover transition-transform duration-500 group-hover:scale-105" loading="lazy" />
					<div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100 flex flex-col justify-end p-4">
						<p class="text-xs font-bold text-white truncate drop-shadow-md">{s.name}</p>
						<p class="text-[10px] text-brand-500 font-medium">{new Date(s.modified).toLocaleString()}</p>
					</div>
					<button class="absolute top-3 right-3 p-2 bg-red-500/20 text-red-300 rounded-xl opacity-0 group-hover:opacity-100 hover:bg-red-500 hover:text-white transition-all backdrop-blur-md cursor-pointer" onclick={(e) => { e.stopPropagation(); confirmDelete(s); }}>
						<Trash2 class="h-4 w-4" />
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if selectedImage}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-[100] bg-black/95 backdrop-blur-2xl flex flex-col items-center justify-center p-4 select-none" onclick={() => selectedImage = null} transition:fade={{duration: 200}}>
		<div class="absolute top-6 left-6 right-6 flex items-center justify-between z-10" onclick={(e) => e.stopPropagation()}>
			<span class="text-xs font-bold text-white/80 truncate max-w-sm drop-shadow">{selectedImage.name}</span>
			<div class="flex items-center gap-2">
				<div class="flex items-center gap-1 bg-white/10 backdrop-blur-md rounded-full px-2 py-1 border border-white/10">
					<button type="button" class="p-1.5 text-white/70 hover:text-white rounded-full hover:bg-white/10 cursor-pointer" onclick={zoomOut} title="Diminuir Zoom (-)">
						<ZoomOut class="w-3.5 h-3.5" />
					</button>
					<button type="button" class="px-2 text-[10px] font-mono text-white/90 hover:text-white cursor-pointer" onclick={resetZoom} title="Resetar Zoom">
						{Math.round(zoom * 100)}%
					</button>
					<button type="button" class="p-1.5 text-white/70 hover:text-white rounded-full hover:bg-white/10 cursor-pointer" onclick={zoomIn} title="Aumentar Zoom (+)">
						<ZoomIn class="w-3.5 h-3.5" />
					</button>
				</div>
				<button type="button" aria-label="Fechar visualização" class="p-2.5 bg-white/10 hover:bg-white/20 rounded-full text-white transition-colors cursor-pointer" onclick={() => selectedImage = null}>
					<X class="h-4 w-4" />
				</button>
			</div>
		</div>

		<div class="flex-1 w-full flex items-center justify-center overflow-hidden p-6" onclick={(e) => e.stopPropagation()}>
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<img 
				src={selectedImage.dataUrl || convertFileSrc(selectedImage.path)} 
				alt={selectedImage.name} 
				class="max-w-full max-h-[75vh] object-contain rounded-2xl shadow-2xl transition-transform duration-150 ease-out" 
				style="transform: scale({zoom});"
				onclick={(e) => e.stopPropagation()} 
			/>
		</div>

		<div class="absolute bottom-6 flex flex-wrap items-center gap-3 z-10" onclick={(e) => e.stopPropagation()}>
			<button 
				type="button"
				class="px-4 py-2 rounded-full bg-brand-500 hover:bg-brand-400 text-black text-xs font-black flex items-center gap-2 shadow-lg shadow-brand-500/20 cursor-pointer transition-all"
				onclick={() => copyImage(selectedImage!)}
			>
				<Copy class="h-3.5 w-3.5" /> Copiar Imagem (Ctrl+C)
			</button>
			<button 
				type="button"
				class="px-4 py-2 rounded-full bg-white/10 hover:bg-white/20 text-white text-xs font-bold flex items-center gap-2 transition-all cursor-pointer border border-white/10"
				onclick={() => copyToClipboard(selectedImage!.path)}
			>
				<Copy class="h-3.5 w-3.5" /> Copiar Caminho
			</button>
			<button 
				type="button"
				class="px-4 py-2 rounded-full bg-red-500/20 hover:bg-red-500/30 text-red-300 text-xs font-bold flex items-center gap-2 transition-all cursor-pointer border border-red-500/20"
				onclick={() => confirmDelete(selectedImage!)}
			>
				<Trash2 class="h-3.5 w-3.5" /> {t("screenshots.deleteBtn")}
			</button>
		</div>
	</div>
{/if}

<Modal isOpen={showDeleteConfirm} onClose={() => showDeleteConfirm = false} title={t("screenshots.deleteTitle")}>
	<div class="py-4">
		<p class="text-fg">{t("screenshots.deleteConfirmText")} <strong class="text-brand-300">{screenshotToDelete?.name}</strong>?</p>
		<p class="mt-2 text-sm text-fg-subtle">{t("screenshots.deleteIrreversible")}</p>
	</div>
	<div class="mt-6 flex justify-end gap-3">
		<Button variant="ghost" onclick={() => showDeleteConfirm = false}>{t("app.cancel")}</Button>
		<Button variant="danger" onclick={deleteImage} loading={deleting}>
			{t("screenshots.deleteBtn")}
		</Button>
	</div>
</Modal>
