<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { Cpu, MemoryStick } from "lucide-svelte";

	type Reading = {
		cpuPercent: number;
		memMb: number;
		ts: number;
	};

	let readings = $state<Reading[]>([]);
	let pollHandle: number | null = $state(null);

	async function tick(pid: number | null) {
		if (!pid) return;
		try {
			const sample = await sampleProcess(pid);
			if (sample) {
				readings = [...readings, sample].slice(-60);
			}
		} catch {}
	}

	async function sampleProcess(pid: number): Promise<Reading | null> {
		try {
			const stat = await fetch(`/proc/${pid}/stat`).then((r) => (r.ok ? r.text() : null));
			if (!stat) return null;
			const parts = stat.split(" ");
			const utime = Number(parts[13]);
			const stime = Number(parts[14]);
			const rssKb = Number(parts[23]);
			const totalTicks = utime + stime;
			const totalMemMb = rssKb / 1024;
			return {
				cpuPercent: Math.min(100, totalTicks / 100),
				memMb: Math.round(totalMemMb),
				ts: Date.now(),
			};
		} catch {
			return null;
		}
	}

	type Props = {
		pid: number | null;
		running: boolean;
	};

	let { pid, running }: Props = $props();
	let lastRunning = $state(false);

	$effect(() => {
		const shouldRun = running && pid !== null;
		if (shouldRun && !lastRunning) {
			pollHandle = window.setInterval(() => tick(pid), 1000);
			lastRunning = true;
		}
		if (!shouldRun && lastRunning) {
			clearInterval(pollHandle!);
			pollHandle = null;
			lastRunning = false;
		}
	});

	onDestroy(() => {
		if (pollHandle !== null) {
			clearInterval(pollHandle);
		}
	});

	const current = $derived(readings.at(-1));
	const max = $derived(Math.max(1024, ...readings.map((r) => r.memMb)));
</script>

<div
	class="flex items-stretch gap-3 rounded-lg p-2"
	style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg-subtle));"
>
	<div class="flex flex-1 items-center gap-2">
		<MemoryStick class="h-3.5 w-3.5" style="color: rgb(var(--brand-400));" />
		<div class="flex-1">
			<p class="text-[10px] uppercase tracking-wide" style="color: rgb(var(--fg-subtle));">
				Memory (RSS)
			</p>
			<p class="text-sm font-medium" style="color: rgb(var(--fg));">
				{current ? `${(current.memMb / 1024).toFixed(1)} GB` : "—"}
			</p>
			<div
				class="mt-1 h-1 w-full overflow-hidden rounded-full"
				style="background: rgba(255,255,255,0.04);"
			>
				<div
					class="h-full"
					style="width: {current ? Math.min(100, (current.memMb / max) * 100) : 0}%; background: rgb(var(--brand-500));"
				></div>
			</div>
		</div>
	</div>
	<div class="flex flex-1 items-center gap-2">
		<Cpu class="h-3.5 w-3.5" style="color: rgb(var(--brand-400));" />
		<div class="flex-1">
			<p class="text-[10px] uppercase tracking-wide" style="color: rgb(var(--fg-subtle));">
				CPU (rough)
			</p>
			<p class="text-sm font-medium" style="color: rgb(var(--fg));">
				{current ? `${current.cpuPercent.toFixed(1)} %` : "—"}
			</p>
			<div
				class="mt-1 h-1 w-full overflow-hidden rounded-full"
				style="background: rgba(255,255,255,0.04);"
			>
				<div
					class="h-full"
					style="width: {current ? Math.min(100, current.cpuPercent) : 0}%; background: rgb(var(--brand-500));"
				></div>
			</div>
		</div>
	</div>
</div>
