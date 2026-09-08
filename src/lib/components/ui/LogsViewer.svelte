<script lang="ts">
	import { Search, Trash2, RefreshCw, X } from "lucide-svelte";
	import { onMount } from "svelte";
	import { launchLogsList, launchLogsGet, launchLogsSearch, launchLogsClear, type LaunchLogSummary, type LaunchLogLine } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	let summaries = $state<LaunchLogSummary[]>([]);
	let selected = $state<LaunchLogSummary | null>(null);
	let lines = $state<LaunchLogLine[]>([]);
	let query = $state("");
	let minLevel = $state<"INFO" | "WARN" | "ERROR">("INFO");
	let loading = $state(false);
	let searching = $state(false);
	let error = $state<string | null>(null);

	async function loadList() {
		loading = true;
		try {
			summaries = await launchLogsList(100);
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function loadLines(id: number) {
		try {
			lines = await launchLogsGet(id);
		} catch (e) {
			error = String(e);
		}
	}

	async function doSearch() {
		if (!query.trim()) {
			if (selected) await loadLines(selected.id);
			return;
		}
		searching = true;
		try {
			lines = await launchLogsSearch(query, minLevel, 500);
		} catch (e) {
			error = String(e);
		} finally {
			searching = false;
		}
	}

	async function clearAll() {
		try {
			await launchLogsClear();
			summaries = [];
			lines = [];
			selected = null;
		} catch (e) {
			error = String(e);
		}
	}

	function pickSummary(s: LaunchLogSummary) {
		selected = s;
		query = "";
		loadLines(s.id);
	}

	onMount(loadList);

	function levelColor(level: string): string {
		if (level === "ERROR") return "rgb(248, 113, 113)";
		if (level === "WARN") return "rgb(250, 204, 21)";
		return "rgb(145, 155, 180)";
	}
</script>

<div class="flex h-full flex-col gap-3">
	<div class="flex items-center gap-2">
		<div class="relative flex-1">
			<Search
				class="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2"
				style="color: rgb(var(--fg-subtle));"
			/>
			<input
				type="text"
				class="h-8 w-full rounded-md pl-7 pr-2 text-sm outline-none"
				style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg)); color: rgb(var(--fg));"
				placeholder={t("logs.searchSessions")}
				bind:value={query}
				onkeydown={(e) => e.key === "Enter" && doSearch()}
			/>
		</div>
		<select
			class="h-8 rounded-md px-2 text-xs outline-none"
			style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg)); color: rgb(var(--fg));"
			bind:value={minLevel}
			onchange={doSearch}
		>
			<option value="INFO">{t("logs.levelInfo")}</option>
			<option value="WARN">{t("logs.levelWarn")}</option>
			<option value="ERROR">{t("logs.levelError")}</option>
		</select>
		<button
			type="button"
			class="grid h-8 w-8 place-items-center rounded-md"
			style="border: 1px solid rgb(var(--border)); color: rgb(var(--fg-muted));"
			onclick={loadList}
			aria-label={t("logs.refresh")}
		>
			<RefreshCw class="h-3.5 w-3.5" />
		</button>
		<button
			type="button"
			class="grid h-8 w-8 place-items-center rounded-md"
			style="border: 1px solid rgb(var(--border)); color: rgb(248, 113, 113);"
			onclick={clearAll}
			aria-label={t("logs.clearAll")}
		>
			<Trash2 class="h-3.5 w-3.5" />
		</button>
	</div>

	{#if error}
		<p class="text-xs" style="color: rgb(248, 113, 113);">{error}</p>
	{/if}

	<div class="grid flex-1 grid-cols-1 gap-3 md:grid-cols-[20rem_1fr]">
		<div
			class="overflow-y-auto rounded-md"
			style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));"
		>
			{#if loading}
				<p class="p-3 text-xs" style="color: rgb(var(--fg-subtle));">{t("logs.loadingSessions")}</p>
			{:else if summaries.length === 0}
				<p class="p-3 text-xs" style="color: rgb(var(--fg-subtle));">{t("logs.noSessions")}</p>
			{:else}
				<ul>
					{#each summaries as s (s.id)}
						<li>
							<button
								type="button"
								class="flex w-full flex-col gap-1 border-b px-3 py-2 text-left text-xs transition-colors"
								style="border-color: rgb(var(--border)); background: {selected?.id === s.id
									? 'rgba(45, 212, 191, 0.08)'
									: 'transparent'};"
								onclick={() => pickSummary(s)}
							>
								<span class="flex items-center justify-between">
									<span class="font-medium" style="color: rgb(var(--fg));">{s.versionId}</span>
									<span
										style="color: {s.exitCode === 0
											? 'rgb(74, 222, 128)'
											: 'rgb(248, 113, 113)'};"
									>
										{t("logs.exit", { code: s.exitCode ?? "?" })}
									</span>
								</span>
								<span class="flex items-center justify-between text-[10px]" style="color: rgb(var(--fg-subtle));">
									<span>{new Date(s.startedAt).toLocaleString()}</span>
									<span>{t("logs.linesCountPlain", { count: s.lineCount })}</span>
								</span>
								{#if s.errorClassification}
									<span class="text-[10px] font-medium" style="color: rgb(250, 204, 21);">
										{s.errorClassification}
									</span>
								{/if}
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</div>

		<div
			class="flex min-h-0 flex-col overflow-hidden rounded-md"
			style="border: 1px solid rgb(var(--border)); background: rgb(8, 9, 14);"
		>
			<div
				class="flex items-center justify-between border-b px-3 py-1.5 text-[11px]"
				style="border-color: rgb(var(--border)); color: rgb(var(--fg-subtle));"
			>
				<span>
					{selected
						? `Log #${selected.id} — ${selected.versionId}`
						: searching
							? t("logs.searching")
							: t("logs.selectSessionOrSearch")}
				</span>
				{#if selected}
					<button
						type="button"
						class="grid h-6 w-6 place-items-center rounded transition-colors hover:bg-white/5"
						onclick={() => {
							selected = null;
							lines = [];
						}}
						aria-label={t("logs.closeSession")}
					>
						<X class="h-3 w-3" />
					</button>
				{/if}
			</div>
			<pre class="flex-1 overflow-y-auto p-3 font-mono text-[11px] leading-relaxed" style="color: rgb(var(--fg-muted));">
{#each lines as line (line.id)}<span style="color: {levelColor(line.level)};">{line.level.padEnd(5)}</span> <span style="color: rgb(var(--fg-subtle));">{line.ts.substring(11, 19)}</span> {line.message}
{/each}
			</pre>
		</div>
	</div>
</div>
