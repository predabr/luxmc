<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import {
		Keyboard,
		AlertTriangle,
		Check,
		X,
		RefreshCw,
		Sliders,
		Search,
		ShieldAlert
	} from "lucide-svelte";
	import { keybindsList, keybindsUpdate, type KeybindEntry } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let {
		isOpen = $bindable(false),
		profileId,
		onClose
	}: {
		isOpen: boolean;
		profileId: string;
		onClose: () => void;
	} = $props();

	let keybinds = $state<KeybindEntry[]>([]);
	let totalConflicts = $state(0);
	let isLoading = $state(true);
	let isSaving = $state(false);
	let searchQuery = $state("");
	let activeCategory = $state<string>("Todos");

	let listeningKeyId = $state<string | null>(null);
	let pendingUpdates = $state<Record<string, string>>({});

	$effect(() => {
		if (isOpen && profileId) {
			loadKeybinds();
		}
	});

	async function loadKeybinds() {
		isLoading = true;
		try {
			const res = await keybindsList(profileId);
			keybinds = res.keybinds;
			totalConflicts = res.totalConflicts;
		} catch (e) {
			toast(uiText("ui.8cb70fab848dec17") + String(e), "error");
		} finally {
			isLoading = false;
		}
	}

	const categories = $derived.by(() => {
		const cats = new Set<string>();
		cats.add("Todos");
		for (const k of keybinds) {
			cats.add(k.category);
		}
		return Array.from(cats);
	});

	const filteredKeybinds = $derived.by(() => {
		return keybinds.filter(k => {
			const matchCat = activeCategory === "Todos" || k.category === activeCategory;
			const matchSearch = !searchQuery || k.label.toLowerCase().includes(searchQuery.toLowerCase()) || k.id.toLowerCase().includes(searchQuery.toLowerCase());
			return matchCat && matchSearch;
		});
	});

	function startListening(k: KeybindEntry) {
		listeningKeyId = k.id;
		playSound("click");
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (!listeningKeyId || e.key === "Shift") return;
		e.preventDefault();
		e.stopPropagation();

		let rawKey = `key.keyboard.${e.code.toLowerCase().replace("key", "").replace("digit", "")}`;
		if (e.code === "Space") rawKey = "key.keyboard.space";
		if (e.code === "Escape") rawKey = "key.keyboard.escape";
		if (e.code === "ControlLeft") rawKey = "key.keyboard.left.control";
		if (e.code === "ControlRight") rawKey = "key.keyboard.right.control";

		pendingUpdates[listeningKeyId] = rawKey;

		const target = keybinds.find(k => k.id === listeningKeyId);
		if (target) {
			target.rawKey = rawKey;
			target.displayKey = e.key.toUpperCase();
		}

		listeningKeyId = null;
		playSound("click");
	}

	async function handleSave() {
		if (Object.keys(pendingUpdates).length === 0) {
			onClose();
			return;
		}
		isSaving = true;
		try {
			await keybindsUpdate(profileId, pendingUpdates);
			toast(uiText("ui.3e9b650bc548eabe"), "success");
			pendingUpdates = {};
			loadKeybinds();
			onClose();
		} catch (e) {
			toast(uiText("ui.bc6e26732b5528a9") + String(e), "error");
		} finally {
			isSaving = false;
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-md select-none" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="w-full max-w-3xl rounded-3xl bg-bg-elevated border border-fg/15 p-6 shadow-2xl space-y-5 max-h-[90vh] flex flex-col" in:scale={{ easing: backOut, start: 0.95, duration: 260 }}>
			<!-- Header -->
			<div class="flex items-center justify-between border-b border-fg/10 pb-4 shrink-0">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center text-brand-400">
						<Keyboard class="w-5 h-5" />
					</div>
					<div>
						<div class="flex items-center gap-2">
							<h3 class="text-base font-black text-fg">{uiText("ui.de0b867e7c3eb8d5")}</h3>
							{#if totalConflicts > 0}
								<span class="px-2.5 py-0.5 rounded-full text-[10px] font-black bg-rose-500/20 text-rose-300 border border-rose-500/40 flex items-center gap-1">
									<AlertTriangle class="w-3 h-3" />
									{totalConflicts} {uiText("ui.6da93a563e93f964")}
								</span>
							{/if}
						</div>
						<p class="text-xs text-fg/50 mt-0.5">{uiText("ui.7b71c746a9487eae")}</p>
					</div>
				</div>

				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					onclick={onClose}
                    aria-label={uiText("ui.f5058ad040397ade")}
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			<!-- Filters -->
			<div class="flex flex-col sm:flex-row items-center justify-between gap-3 shrink-0">
				<div class="flex bg-bg-elevated border border-fg/10 rounded-full p-1 gap-1 overflow-x-auto w-full sm:w-auto">
					{#each categories as cat}
						<button
							type="button"
							class="px-3.5 py-1.5 rounded-full text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer whitespace-nowrap {activeCategory === cat ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/40 hover:text-fg'}"
							onclick={() => activeCategory = cat}
						>
							{cat === "Todos" ? uiText("mods.categoryAll") : cat}
						</button>
					{/each}
				</div>

				<div class="relative w-full sm:w-60">
					<Search class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-fg/40" />
					<input
						type="text"
						bind:value={searchQuery}
						placeholder={uiText("ui.6e1d3413fb4f3434")}
						class="w-full pl-9 pr-3 py-1.5 rounded-full bg-bg-elevated border border-fg/10 text-xs text-fg focus:outline-none focus:border-brand-500"
					/>
				</div>
			</div>

			<!-- Keybind List (Scrollable) -->
			<div class="flex-1 overflow-y-auto pr-1 space-y-2 custom-scrollbar">
				{#if isLoading}
					<div class="p-12 text-center text-fg/40 flex flex-col items-center justify-center gap-2">
						<RefreshCw class="w-6 h-6 animate-spin text-brand-400" />
						<span class="text-xs font-bold">{uiText("ui.4b4bfde86cd679f4")}</span>
					</div>
				{:else if filteredKeybinds.length === 0}
					<div class="p-12 text-center text-fg/40 rounded-2xl bg-bg-elevated border border-fg/5">
						<p class="text-xs">{uiText("ui.cba7124b7f9f3d95")}</p>
					</div>
				{:else}
					{#each filteredKeybinds as k (k.id)}
						<div class="p-3 rounded-2xl border transition-[color,background-color,border-color,box-shadow,transform,opacity] flex items-center justify-between {k.isConflict ? 'bg-rose-500/10 border-rose-500/30' : 'bg-bg-elevated border-fg/5 hover:border-fg/15'}">
							<div class="min-w-0 pr-4">
								<div class="flex items-center gap-2">
									<span class="text-xs font-bold text-fg truncate">{k.label}</span>
									<span class="text-[9px] font-mono text-fg/40 bg-fg/5 px-2 py-0.5 rounded">{k.category}</span>
									{#if k.isConflict}
										<span class="text-[9px] font-extrabold text-rose-400 bg-rose-500/20 px-1.5 py-0.5 rounded border border-rose-500/30">{uiText("ui.3f490145e2e4ea68")}</span>
									{/if}
								</div>
								<span class="text-[10px] text-fg/40 font-mono block mt-0.5">{k.id}</span>
							</div>

							<button
								type="button"
								class="px-4 py-2 rounded-xl text-xs font-mono font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer min-w-[90px] text-center {listeningKeyId === k.id ? 'bg-brand-500 text-brand-foreground animate-pulse shadow-lg' : k.isConflict ? 'bg-rose-500/20 text-rose-300 border border-rose-500/40 hover:bg-rose-500/30' : 'bg-bg-subtle hover:bg-bg-subtle text-fg/90 border border-fg/10'}"
								onclick={() => startListening(k)}
							>
								{listeningKeyId === k.id ? "Pressione..." : k.displayKey}
							</button>
						</div>
					{/each}
				{/if}
			</div>

			<!-- Footer -->
			<div class="flex items-center justify-between pt-3 border-t border-fg/10 shrink-0">
				<span class="text-xs text-fg/40">{uiText("ui.bc99604d78dda252")}</span>

				<div class="flex items-center gap-2.5">
					<button
						type="button"
						class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
						onclick={onClose}
					>
						{uiText("common.cancel")}
					</button>

					<button
						type="button"
						class={launcherButton({ variant: "primary", size: "sm", class: "disabled:opacity-50" })}
						onclick={handleSave}
						disabled={isSaving}
					>
						{isSaving ? "Salvando..." : uiText("ui.1fd800c616a0b723")}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
