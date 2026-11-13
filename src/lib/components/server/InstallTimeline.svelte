<script lang="ts">
	import { fly, fade } from 'svelte/transition';
	import { quintOut } from 'svelte/easing';
	import {
		LoaderCircle,
		CheckCircle2,
		XCircle,
		CircleDashed,
		AlertTriangle,
		Terminal
	} from 'lucide-svelte';
	import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { localServerList, type ServerPhase } from '$lib/api/localServers';

	export interface TimelineStep {
		id: string;
		label: string;
		detail: string;
	}

	let { steps }: { steps: TimelineStep[] } = $props();

	let visibleCount = $state(0);
	let doneIds = $state<string[]>([]);
	let details = $state<Record<string, string>>({});
	let failedId = $state<string | null>(null);
	let failedDetail = $state('');
	let phaseSeen = $state<string>('');
	let consoleTail = $state<string[]>([]);

	const PHASE_STEP: Partial<Record<ServerPhase, string>> = {
		preparing: 'prepare',
		downloading: 'download',
		java: 'java',
		starting: 'start',
		running: 'verify',
		failed: 'verify'
	};

	export function begin(): void {
		visibleCount = Math.min(3, steps.length);
		doneIds = [];
		details = {};
		failedId = null;
		failedDetail = '';
		phaseSeen = '';
	}

	export async function completeStep(id: string, detail?: string): Promise<void> {
		const index = steps.findIndex((step) => step.id === id);
		if (index < 0) return;
		if (detail) steps[index].detail = detail;
		if (!doneIds.includes(id)) doneIds = [...doneIds, id];
		visibleCount = Math.max(visibleCount, Math.min(index + 3, steps.length));
	}

	export async function failStep(id: string, detail: string): Promise<void> {
		failedId = id;
		failedDetail = detail;
		visibleCount = steps.length;
	}

	export async function trackPhase(phase: string, runningConsoleTail: string[]): Promise<void> {
		if (phase && phase === phaseSeen) return;
		phaseSeen = phase;
		consoleTail = runningConsoleTail.slice(-4);
		if (phase === 'failed') {
			await failStep('verify', 'A inicialização falhou. Confira as mensagens do servidor abaixo.');
			return;
		}
		const target = PHASE_STEP[phase as ServerPhase];
		if (target) await completeStep(target);
		if (phase === 'running') {
			// Todas as etapas do pipeline real foram executadas pelo backend até chegar em running.
			for (const id of ['java', 'download', 'start']) await completeStep(id);
		}
	}

	export function isFailed(): boolean {
		return failedId !== null;
	}
</script>

<ol class="timeline" aria-label="Progresso da instalação">
	{#each steps as step, index (step.id)}
		{#if index < visibleCount}
			{@const detail = details[step.id] ?? step.detail}
		{@const done = doneIds.includes(step.id)}
			{@const failed = failedId === step.id}
			{@const pending = !done && !failed && index > 0 && !doneIds.includes(steps[index - 1].id) && failedId === null}
			<li
				class="step"
				class:done
				class:failed
				class:pending
				in:fly={{ y: 14, duration: 240, easing: quintOut, delay: Math.min(index * 40, 200) }}
			>
				<span class="marker" aria-hidden="true">
					{#if done}
						<CheckCircle2 class="h-4.5 w-4.5 text-success" />
					{:else if failed}
						<XCircle class="h-4.5 w-4.5 text-danger" />
					{:else if pending}
						<CircleDashed class="h-4.5 w-4.5 text-fg-subtle" />
					{:else}
						<LoaderCircle class="h-4.5 w-4.5 animate-spin text-amethyst-400" />
					{/if}
				</span>
				<span class="body">
					<span class="label">{step.label}</span>
					<span class="detail">{detail}</span>
				</span>
				{#if failed}
					<span class="fail-note" in:fade={{ duration: 160 }}>
						<AlertTriangle class="h-3.5 w-3.5 shrink-0 text-warning" />
						{failedDetail}
					</span>
				{/if}
			</li>
		{/if}
	{/each}
</ol>

{#if consoleTail.length > 0 && (phaseSeen === 'starting' || phaseSeen === 'running' || phaseSeen === 'failed')}
	<div class="log" in:fade={{ duration: 200 }} aria-label="Últimas mensagens do servidor">
		<p class="log-title"><Terminal class="h-3.5 w-3.5" /> Servidor</p>
		{#each consoleTail as line (line)}
			<p class="log-line">{line}</p>
		{/each}
	</div>
{/if}

<style>
	.timeline { display: flex; flex-direction: column; gap: 4px; }
	.step {
		display: flex; align-items: flex-start; gap: 12px;
		padding: 10px 12px; border-radius: 16px;
		border: 1px solid transparent;
	}
	.step.done { border-color: rgb(var(--success) / 0.16); background: rgb(var(--success) / 0.05); }
	.step.failed { border-color: rgb(var(--danger) / 0.35); background: rgb(var(--danger) / 0.07); }
	.step.pending { opacity: 0.55; }
	.marker { display: grid; place-items: center; width: 22px; height: 22px; flex: none; margin-top: 1px; }
	.marker :global(svg) { width: 18px; height: 18px; }
	.body { display: flex; flex-direction: column; gap: 2px; min-width: 0; flex: 1; }
	.label { font-size: 13.5px; font-weight: 600; color: rgb(var(--fg)); }
	.pending .label { color: rgb(var(--fg-muted)); }
	.detail { font-size: 12px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.fail-note { display: flex; align-items: center; gap: 8px; margin-top: 4px; font-size: 12px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.log { margin-top: 12px; padding: 12px 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: rgb(var(--bg-overlay) / 0.6); }
	.log-title { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; font-size: 10.5px; font-weight: 700; letter-spacing: 0.14em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.log-line { font-family: 'JetBrains Mono', ui-monospace, monospace; font-size: 11px; line-height: 1.6; color: rgb(var(--fg-muted)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
