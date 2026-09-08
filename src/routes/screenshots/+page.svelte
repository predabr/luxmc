<script lang="ts">
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import Modal from "$lib/components/ui/Modal.svelte";
	import { Camera, X, FolderOpen, RefreshCw, Copy, Download, Trash2 } from "lucide-svelte";
import { fade } from "svelte/transition";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { instancesScreenshots, screenshotDelete, screenshotsOpenFolder } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

	let screenshots = $state<Array<{ name: string; path: string; modified: string }>>([]);
	let loading = $state(false);
	let selectedImage = $state<{ name: string; path: string; modified: string } | null>(null);
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

	function openImage(screenshot: { name: string; path: string; modified: string }) {
		selectedImage = screenshot;
	}

	function closeModal() {
		selectedImage = null;
	}

	async function copyImage(screenshot: { name: string; path: string }) {
		try {
			const res = await fetch(`asset://localhost/${screenshot.path}`);
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
		a.href = `asset://localhost/${screenshot.path}`;
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
		navigator.clipboard.writeText(path).then(() => {
			toast("Copied to clipboard", "success");
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
				<div class="group relative overflow-hidden rounded-[20px] luxmc-glass border border-white/5 cursor-zoom-in break-inside-avoid" onclick={() => selectedImage = s}>
					<img src={'http://localhost:1420/_tauri_asset/' + encodeURIComponent(s.path)} alt={s.name} class="w-full h-auto object-cover transition-transform duration-500 group-hover:scale-105" loading="lazy" />
					<div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100 flex flex-col justify-end p-4">
						<p class="text-xs font-bold text-white truncate drop-shadow-md">{s.name}</p>
						<p class="text-[10px] text-brand-300 font-medium">{new Date(s.modified).toLocaleString()}</p>
					</div>
					<button class="absolute top-3 right-3 p-2 bg-red-500/20 text-red-300 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-red-500 hover:text-white transition-all backdrop-blur-md" onclick={(e) => { e.stopPropagation(); confirmDelete(s); }}>
						<Trash2 class="h-4 w-4" />
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if selectedImage}
	<div class="fixed inset-0 z-[100] bg-black/90 backdrop-blur-2xl flex items-center justify-center p-8" onclick={() => selectedImage = null} transition:fade={{duration: 200}}>
		<button class="absolute top-6 right-6 p-3 bg-white/10 hover:bg-white/20 rounded-full text-white transition-colors">
			<X class="h-6 w-6" />
		</button>
		<img src={'http://localhost:1420/_tauri_asset/' + encodeURIComponent(selectedImage.path)} alt={selectedImage.name} class="max-w-full max-h-full object-contain rounded-[24px] shadow-2xl" onclick={(e) => e.stopPropagation()} />
		<div class="absolute bottom-8 flex gap-4" onclick={(e) => e.stopPropagation()}>
			<Button variant="secondary" onclick={() => copyToClipboard(selectedImage!.path)}>
				<Copy class="h-4 w-4"/> Copy Path
			</Button>
			<Button variant="solid" onclick={() => confirmDelete(selectedImage!)}>
				<Trash2 class="h-4 w-4"/> Delete
			</Button>
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
