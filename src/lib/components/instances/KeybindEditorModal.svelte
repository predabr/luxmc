<script lang="ts">
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
			toast("Falha ao ler opções de controles: " + String(e), "error");
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
		if (!listeningKeyId) return;
		e.preventDefault();
		e.stopPropagation();

		let rawKey = `key.keyboard.${e.code.toLowerCase().replace("key", "").replace("digit", "")}`;
		if (e.code === "Space") rawKey = "key.keyboard.space";
		if (e.code === "Escape") rawKey = "key.keyboard.escape";
		if (e.code === "ShiftLeft") rawKey = "key.keyboard.left.shift";
		if (e.code === "ShiftRight") rawKey = "key.keyboard.right.shift";
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
			toast("Controles salvos com sucesso!", "success");
			pendingUpdates = {};
			loadKeybinds();
			onClose();
		} catch (e) {
			toast("Erro ao salvar controles: " + String(e), "error");
		} finally {
			isSaving = false;
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-md select-none" in:fade={{ duration: 150 }}>
		<div class="w-full max-w-3xl rounded-3xl bg-bg-elevated border border-fg/15 p-6 shadow-2xl space-y-5 max-h-[90vh] flex flex-col" in:scale={{ start: 0.95, duration: 200 }}>
			<!-- Header -->
			<div class="flex items-center justify-between border-b border-fg/10 pb-4 shrink-0">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center text-brand-400">
						<Keyboard class="w-5 h-5" />
					</div>
					<div>
						<div class="flex items-center gap-2">
							<h3 class="text-base font-black text-fg">Gerenciador de Controles & Teclas</h3>
							{#if totalConflicts > 0}
								<span class="px-2.5 py-0.5 rounded-full text-[10px] font-black bg-rose-500/20 text-rose-300 border border-rose-500/40 flex items-center gap-1">
									<AlertTriangle class="w-3 h-3" />
									{totalConflicts} Conflito(s)
								</span>
							{/if}
						</div>
						<p class="text-xs text-fg/50 mt-0.5">Edite atalhos e resolva teclas conflitantes sem precisar entrar no jogo</p>
					</div>
				</div>

				<button
					type="button"
					class="p-2 rounded-xl text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors cursor-pointer"
					onclick={onClose}
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
							class="px-3.5 py-1.5 rounded-full text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeCategory === cat ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/40 hover:text-fg'}"
							onclick={() => activeCategory = cat}
						>
							{cat}
						</button>
					{/each}
				</div>

				<div class="relative w-full sm:w-60">
					<Search class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-fg/40" />
					<input
						type="text"
						bind:value={searchQuery}
						placeholder="Buscar controle..."
						class="w-full pl-9 pr-3 py-1.5 rounded-full bg-bg-elevated border border-fg/10 text-xs text-fg focus:outline-none focus:border-brand-500"
					/>
				</div>
			</div>

			<!-- Keybind List (Scrollable) -->
			<div class="flex-1 overflow-y-auto pr-1 space-y-2 custom-scrollbar">
				{#if isLoading}
					<div class="p-12 text-center text-fg/40 flex flex-col items-center justify-center gap-2">
						<RefreshCw class="w-6 h-6 animate-spin text-brand-400" />
						<span class="text-xs font-bold">Lendo options.txt...</span>
					</div>
				{:else if filteredKeybinds.length === 0}
					<div class="p-12 text-center text-fg/40 rounded-2xl bg-bg-elevated border border-fg/5">
						<p class="text-xs">Nenhum controle encontrado com este filtro.</p>
					</div>
				{:else}
					{#each filteredKeybinds as k}
						<div class="p-3 rounded-2xl border transition-all flex items-center justify-between {k.isConflict ? 'bg-rose-500/10 border-rose-500/30' : 'bg-bg-elevated border-fg/5 hover:border-fg/15'}">
							<div class="min-w-0 pr-4">
								<div class="flex items-center gap-2">
									<span class="text-xs font-bold text-fg truncate">{k.label}</span>
									<span class="text-[9px] font-mono text-fg/40 bg-fg/5 px-2 py-0.5 rounded">{k.category}</span>
									{#if k.isConflict}
										<span class="text-[9px] font-extrabold text-rose-400 bg-rose-500/20 px-1.5 py-0.5 rounded border border-rose-500/30">CONFLITO</span>
									{/if}
								</div>
								<span class="text-[10px] text-fg/40 font-mono block mt-0.5">{k.id}</span>
							</div>

							<button
								type="button"
								class="px-4 py-2 rounded-xl text-xs font-mono font-bold transition-all cursor-pointer min-w-[90px] text-center {listeningKeyId === k.id ? 'bg-brand-500 text-brand-foreground animate-pulse shadow-lg' : k.isConflict ? 'bg-rose-500/20 text-rose-300 border border-rose-500/40 hover:bg-rose-500/30' : 'bg-bg-subtle hover:bg-bg-subtle text-fg/90 border border-fg/10'}"
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
				<span class="text-xs text-fg/40">Clique em qualquer tecla para redefinir pelo teclado</span>

				<div class="flex items-center gap-2.5">
					<button
						type="button"
						class="px-5 py-2.5 rounded-2xl bg-fg/5 hover:bg-fg/10 text-fg/60 hover:text-fg font-bold text-xs transition-colors cursor-pointer"
						onclick={onClose}
					>
						Cancelar
					</button>

					<button
						type="button"
						class="px-7 py-2.5 rounded-2xl bg-brand-500 hover:bg-brand-400 text-brand-foreground font-black text-xs transition-all shadow-md cursor-pointer disabled:opacity-50"
						onclick={handleSave}
						disabled={isSaving}
					>
						{isSaving ? "Salvando..." : "Salvar Controles"}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
