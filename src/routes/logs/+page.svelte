<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import Input from "$lib/components/ui/Input.svelte";
	import { Trash2, Terminal, Search, Copy, Download, ArrowDown, Filter, Share2 } from "lucide-svelte";
	import { gameLogs, type LogEntry } from "$lib/stores/app.svelte";
	import { listenGameLog, listenGameExit, listenLauncherLog, shareLogMclogs, type GameExitEvent } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

	let unlistenGameLog: (() => void) | null = null;
	let unlistenGameExit: (() => void) | null = null;
	let unlistenLauncherLog: (() => void) | null = null;
	let autoScroll = $state(true);
	let exitInfo = $state<GameExitEvent | null>(null);
	let searchQuery = $state("");
	let isSharing = $state(false);
	let lastUpdate = $state<Date | null>(null);
	let logContainer = $state<HTMLDivElement | null>(null);

	const filteredEntries = $derived(() => {
		if (!searchQuery.trim()) return gameLogs.entries;
		const q = searchQuery.toLowerCase();
		return gameLogs.entries.filter((e) => e.message.toLowerCase().includes(q));
	});

	onMount(async () => {
		gameLogs.add("system", "Log viewer initialized. Game logs will appear here.");
		lastUpdate = new Date();

		unlistenGameLog = await listenGameLog((entry) => {
			gameLogs.add(entry.stream as "stdout" | "stderr", entry.message);
			lastUpdate = new Date();
		});

		unlistenGameExit = await listenGameExit((event) => {
			exitInfo = event;
			const code = event.code;
			const msg = event.success
				? `Game exited normally (code ${code})`
				: `Game exited with error (code ${code})`;
			gameLogs.add("system", msg);
			lastUpdate = new Date();
		});

		unlistenLauncherLog = await listenLauncherLog((message) => {
			gameLogs.add("system", message);
			lastUpdate = new Date();
		});
	});

	onDestroy(() => {
		unlistenGameLog?.();
		unlistenGameExit?.();
		unlistenLauncherLog?.();
	});

	function clearLogs() {
		gameLogs.clear();
		exitInfo = null;
		lastUpdate = null;
		gameLogs.add("system", "Logs cleared.");
	}

	function copyAll() {
		const text = gameLogs.entries
			.map((e) => `[${e.timestamp.toLocaleTimeString()}] [${e.stream}] ${e.message}`)
			.join("\n");
		navigator.clipboard.writeText(text);
	}

	function exportLogs() {
		const text = gameLogs.entries
			.map((e) => `[${e.timestamp.toLocaleTimeString()}] [${e.stream}] ${e.message}`)
			.join("\n");
		const blob = new Blob([text], { type: "text/plain" });
		const url = URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = url;
		a.download = `luxmc-logs-${new Date().toISOString().slice(0, 19).replace(/:/g, "-")}.txt`;
		a.click();
		URL.revokeObjectURL(url);
	}

	async function handleShareMclogs() {
		const text = gameLogs.entries
			.map((e) => `[${e.timestamp.toLocaleTimeString()}] [${e.stream}] ${e.message}`)
			.join("\n");
		if (!text.trim()) {
			toast("Não há logs para compartilhar.", "error");
			return;
		}
		isSharing = true;
		try {
			const url = await shareLogMclogs(text);
			await navigator.clipboard.writeText(url);
			toast(`Log publicado no mclo.gs! Link copiado: ${url}`, "success");
		} catch (e) {
			toast("Falha ao enviar log para mclo.gs: " + String(e), "error");
		} finally {
			isSharing = false;
		}
	}

	function scrollToBottom() {
		if (logContainer) {
			logContainer.scrollTop = logContainer.scrollHeight;
		}
	}

	function getLineClass(entry: LogEntry): string {
		const msg = entry.message;
		if (entry.stream === "stderr") return "log-error";
		if (entry.stream === "system") return "log-system";
		if (msg.includes("[ERROR]") || msg.includes("error")) return "log-error";
		if (msg.includes("[WARN]") || msg.includes("warn")) return "log-warn";
		if (msg.includes("[DEBUG]") || msg.includes("debug")) return "log-debug";
		return "log-info";
	}

	function getStreamTagClass(entry: LogEntry): string {
		if (entry.stream === "stderr") return "log-error";
		if (entry.stream === "system") return "log-system";
		return "log-info";
	}

	function getRelativeTime(date: Date): string {
		const now = Date.now();
		const diff = now - date.getTime();
		const seconds = Math.floor(diff / 1000);
		if (seconds < 60) return `${seconds}s ago`;
		const minutes = Math.floor(seconds / 60);
		if (minutes < 60) return `${minutes}m ago`;
		const hours = Math.floor(minutes / 60);
		return `${hours}h ago`;
	}
</script>

<div class="mx-auto flex h-full max-w-6xl flex-col gap-4">
	<div class="flex items-center justify-between">
		<Heading>{t("logs.title")}</Heading>
		<div class="flex items-center gap-2">
			{#if exitInfo}
				<span class="text-xs font-mono font-bold px-3 py-1 rounded-full border {exitInfo.success ? 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20' : 'text-rose-400 bg-rose-500/10 border-rose-500/20'}">
					{t("logs.lastExit", { code: exitInfo.code })}
				</span>
			{/if}
		</div>
	</div>

	<div class="flex flex-1 flex-col overflow-hidden bg-[#141518] border border-white/5 rounded-3xl p-5 shadow-xl">
		<div class="mb-3 flex flex-col gap-3">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<div class="w-7 h-7 rounded-xl bg-brand-500/10 border border-brand-500/20 flex items-center justify-center text-brand-500">
						<Terminal class="h-4 w-4" />
					</div>
					<span class="text-sm font-bold text-white/90">{t("logs.gameOutput")}</span>
					<span class="text-xs font-mono text-white/40 bg-white/5 px-2 py-0.5 rounded-full">{t("logs.linesCount", { count: filteredEntries().length })}</span>
				</div>
			</div>

			<div class="flex items-center gap-2">
				<div class="flex-1">
					<Input
						bind:value={searchQuery}
						placeholder={t("logs.filterPlaceholder")}
						class="flex-1"
					>
						{#snippet leadingIcon()}
							<Search class="h-3.5 w-3.5 text-white/40" />
						{/snippet}
					</Input>
				</div>
				<Button variant="secondary" size="sm" onclick={copyAll}>
					<Copy class="h-3.5 w-3.5" />
					{t("logs.copyAll")}
				</Button>
				<Button variant="secondary" size="sm" onclick={exportLogs}>
					<Download class="h-3.5 w-3.5" />
					{t("logs.export")}
				</Button>
				<Button variant="solid" size="sm" onclick={handleShareMclogs} loading={isSharing} class="bg-brand-500 hover:bg-[#ebd095] text-black font-bold">
					<Share2 class="h-3.5 w-3.5" />
					Compartilhar (mclo.gs)
				</Button>
				<Button variant="secondary" size="sm" onclick={clearLogs}>
					<Trash2 class="h-3.5 w-3.5" />
					{t("logs.clear")}
				</Button>
			</div>
		</div>

		<div
			bind:this={logContainer}
			class="flex-1 overflow-y-auto rounded-2xl p-4 font-mono text-xs leading-relaxed bg-[#0c0d0f] border border-white/5 custom-scrollbar select-text shadow-inner"
			onscroll={(e) => {
				const el = e.currentTarget;
				const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
				if (!atBottom) autoScroll = false;
			}}
		>
		{#each filteredEntries() as entry (entry.id)}
			<div class="flex gap-2 py-0.5 hover:bg-white/[0.02] rounded px-1 {getLineClass(entry)}">
				<span class="w-16 shrink-0 text-right text-white/30 text-[10px]">
					{getRelativeTime(entry.timestamp)}
				</span>
				<span class="w-14 shrink-0 text-right font-bold text-[11px] {getStreamTagClass(entry)}">
					[{entry.stream}]
				</span>
				<span class="break-all text-white/80">{entry.message}</span>
			</div>
			{:else}
				<p class="p-8 text-center text-white/30">{t("logs.noLogs")}</p>
			{/each}
		</div>

		<div class="mt-3 flex items-center justify-between border-t border-white/5 pt-3 text-[11px] text-white/40">
			<div class="flex items-center gap-3">
				<span>{t("logs.linesCountPlain", { count: filteredEntries().length })}</span>
				{#if searchQuery.trim()}
					<span class="flex items-center gap-1 text-brand-500 font-bold">
						<Filter class="h-3 w-3" />
						{t("logs.filterActive")}
					</span>
				{/if}
			</div>
			<div class="flex items-center gap-3">
				{#if lastUpdate}
					<span>{t("logs.lastUpdate", { time: lastUpdate.toLocaleTimeString() })}</span>
				{/if}
				<button
					class="flex items-center gap-1 rounded-lg px-2 py-1 transition-all cursor-pointer {autoScroll ? 'text-brand-500 bg-brand-500/10 font-bold' : 'text-white/40 hover:text-white hover:bg-white/5'}"
					onclick={() => {
						autoScroll = !autoScroll;
						if (autoScroll) scrollToBottom();
					}}
				>
					<ArrowDown class="h-3 w-3" />
					{t("logs.autoScroll", { status: autoScroll ? t("logs.on") : t("logs.off") })}
				</button>
			</div>
		</div>
	</div>
</div>
