<script lang="ts">
	import { Cpu, MemoryStick } from "lucide-svelte";

	type Reading = {
		cpuPercent: number;
		memMb: number;
		ts: number;
	};

	const MAX_READINGS = 60;
	let readings = $state<Reading[]>(Array(MAX_READINGS).fill(null).map(() => ({ cpuPercent: 0, memMb: 0, ts: 0 })));
	let idx = $state(0);

	async function tick(pid: number | null) {
		if (!pid) return;
		try {
			const sample = await sampleProcess(pid);
			if (sample) {
				readings[idx] = sample;
				idx = (idx + 1) % MAX_READINGS;
			}
		} catch {}
	}

	async function sampleProcess(_pid: number): Promise<Reading | null> {
		return null;
	}

	type Props = {
		pid: number | null;
		running: boolean;
	};

	let { pid, running }: Props = $props();

	$effect(() => {
		if (!running || pid === null) return;
		const currentPid = pid;
		const handle = window.setInterval(() => tick(currentPid), 1000);
		return () => clearInterval(handle);
	});

	const current = $derived.by(() => {
		const i = idx === 0 ? MAX_READINGS - 1 : idx - 1;
		const r = readings[i];
		return (r && r.ts > 0) ? r : null;
	});

	const max = $derived(Math.max(1024, ...readings.filter(r => r.ts > 0).map((r) => r.memMb)));
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
				style="background: rgb(var(--fg) / 0.04);"
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
				style="background: rgb(var(--fg) / 0.04);"
			>
				<div
					class="h-full"
					style="width: {current ? Math.min(100, current.cpuPercent) : 0}%; background: rgb(var(--brand-500));"
				></div>
			</div>
		</div>
	</div>
</div>
