<script lang="ts">
	import { onMount, tick } from "svelte";
	import { Keyboard } from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	function focusOnMount(node: HTMLInputElement) {
		tick().then(() => node.focus());
	}

	type Shortcut = {
		id: string;
		label: string;
		combo: string;
	};

	const STORAGE_KEY = "luxmc.shortcuts";

	const defaults: Shortcut[] = [
		{ id: "open-instances", label: "Go to Instances", combo: "g i" },
		{ id: "open-news", label: "Go to News", combo: "g n" },
		{ id: "open-servers", label: "Go to Servers", combo: "g s" },
		{ id: "open-logs", label: "Go to Logs", combo: "g l" },
		{ id: "open-settings", label: "Go to Settings", combo: "g ," },
		{ id: "open-mods", label: "Go to Mods", combo: "g m" },
		{ id: "open-screenshots", label: "Go to Screenshots", combo: "g p" },
		{ id: "show-shortcuts", label: "Show shortcut help", combo: "?" },
		{ id: "focus-search", label: "Focus search", combo: "/" },
		{ id: "close-modal", label: "Close any modal", combo: "Escape" },
	];

	let shortcuts = $state<Shortcut[]>(defaults);
	let recordingId = $state<string | null>(null);

	onMount(() => {
		try {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (stored) {
				const parsed: Shortcut[] = JSON.parse(stored);
				if (Array.isArray(parsed) && parsed.length === defaults.length) {
					shortcuts = parsed;
				}
			}
		} catch {}
	});

	function persist() {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(shortcuts));
	}

	function describe(e: KeyboardEvent): string {
		const parts: string[] = [];
		if (e.ctrlKey) parts.push("Ctrl");
		if (e.metaKey) parts.push("Meta");
		if (e.altKey) parts.push("Alt");
		if (e.shiftKey) parts.push("Shift");
		const key = e.key === " " ? "Space" : e.key.length === 1 ? e.key.toLowerCase() : e.key;
		if (!["Control", "Shift", "Alt", "Meta"].includes(e.key)) {
			parts.push(key);
		}
		return parts.join("+");
	}

	function handleKey(e: KeyboardEvent, s: Shortcut) {
		e.preventDefault();
		const combo = describe(e);
		shortcuts = shortcuts.map((x) => (x.id === s.id ? { ...x, combo } : x));
		recordingId = null;
		persist();
	}

	function reset(s: Shortcut) {
		const original = defaults.find((d) => d.id === s.id);
		if (original) {
			shortcuts = shortcuts.map((x) => (x.id === s.id ? { ...x, combo: original.combo } : x));
			persist();
		}
	}

	function resetAll() {
		shortcuts = [...defaults];
		persist();
	}

	function getShortcutLabel(id: string, fallback: string): string {
		switch (id) {
			case "open-instances": return t("shortcutsModal.goToInstances");
			case "open-news": return t("shortcutsModal.goToNews");
			case "open-servers": return t("shortcutsModal.goToServers");
			case "open-logs": return t("shortcutsModal.goToLogs");
			case "open-settings": return t("shortcutsModal.goToSettings");
			case "open-mods": return t("shortcutsModal.goToMods");
			case "open-screenshots": return t("shortcutsModal.goToScreenshots");
			case "show-shortcuts": return t("shortcutsModal.showHelp");
			case "focus-search": return t("shortcutsModal.focusSearch");
			case "close-modal": return t("shortcutsModal.closeModal");
			default: return fallback;
		}
	}
</script>

<div class="flex flex-col gap-3">
	<div class="flex items-center justify-between">
		<h3 class="flex items-center gap-2 text-sm font-medium" style="color: rgb(var(--fg));">
			<Keyboard class="h-4 w-4" />
			{t("shortcutsModal.title")}
		</h3>
		<button
			type="button"
			class="rounded-md px-2 py-1 text-[11px] transition-colors"
			style="border: 1px solid rgb(var(--border)); color: rgb(var(--fg-muted));"
			onclick={resetAll}
		>
			{t("shortcutsModal.resetAll")}
		</button>
	</div>

	<ul class="flex flex-col gap-1.5">
		{#each shortcuts as s (s.id)}
			<li
				class="flex items-center gap-2 rounded-md px-2 py-1.5"
				style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));"
			>
				<span class="flex-1 text-xs" style="color: rgb(var(--fg));">{getShortcutLabel(s.id, s.label)}</span>
				{#if recordingId === s.id}
					<input
						class="h-7 w-40 rounded-md px-2 text-[11px] font-mono outline-none"
						style="border: 1px solid rgb(45, 212, 191); background: rgb(var(--bg-subtle)); color: rgb(var(--fg));"
						placeholder={t("shortcutsModal.pressCombo")}
						aria-label={t("shortcutsModal.pressCombo")}
						use:focusOnMount
						onkeydown={(e) => handleKey(e, s)}
						onblur={() => (recordingId = null)}
					/>
				{:else}
					<button
						type="button"
						class="rounded-md px-2 py-0.5 font-mono text-[11px] transition-colors"
						style="border: 1px solid rgb(var(--border)); color: rgb(var(--fg-muted)); background: rgb(var(--bg-subtle));"
						onclick={() => (recordingId = s.id)}
						aria-label={`${getShortcutLabel(s.id, s.label)}: ${s.combo}`}
					>
						{s.combo}
					</button>
					<button
						type="button"
						class="rounded-md px-1.5 py-0.5 text-[10px] transition-colors"
						style="color: rgb(var(--fg-subtle));"
						onclick={() => reset(s)}
					>
						{t("shortcutsModal.reset")}
					</button>
				{/if}
			</li>
		{/each}
	</ul>
</div>
