<script lang="ts" generics="T extends string">
	import { tick } from 'svelte';
	import { fly } from 'svelte/transition';
	import { ChevronDown, Check } from 'lucide-svelte';

	interface Option {
		value: T;
		label: string;
	}

	let {
		value = $bindable(undefined),
		options,
		disabled = false,
		ariaLabel = '',
		accent = ''
	}: { value?: T; options: Option[]; disabled?: boolean; ariaLabel?: string; accent?: string } = $props();

	let open = $state(false);
	let trigger: HTMLButtonElement | null = null;
	const current = $derived(options.find((option) => option.value === value) ?? options[0]);

	function toggle() {
		if (disabled) return;
		open = !open;
		if (open) void focusPicked();
	}

	async function focusPicked() {
		await tick();
		const picked = document.querySelector('.glass-select.open [aria-selected="true"]') as HTMLButtonElement | null;
		(picked ?? document.querySelector('.glass-select.open .option'))?.focus();
	}

	function pick(next: T) {
		value = next;
		open = false;
		trigger?.focus();
	}

	function onWindowDown(event: PointerEvent) {
		if (!open) return;
		const target = event.target as HTMLElement;
		if (!target.closest('.glass-select')) open = false;
	}

	function onTriggerKey(event: KeyboardEvent) {
		if (disabled) return;
		if (!open && (event.key === 'ArrowDown' || event.key === 'Enter' || event.key === ' ')) {
			event.preventDefault();
			toggle();
		}
	}

	function onListKey(event: KeyboardEvent) {
		if (!open) return;
		const items = Array.from(document.querySelectorAll('.glass-select.open .option')) as HTMLButtonElement[];
		const index = items.indexOf(document.activeElement as HTMLButtonElement);
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			items[Math.min(items.length - 1, index + 1)]?.focus();
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			items[Math.max(0, index - 1)]?.focus();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			open = false;
			trigger?.focus();
		}
	}
</script>

<svelte:window onpointerdown={onWindowDown} />

<div class="glass-select" class:open class:disabled style={accent ? `--node-theme: ${accent}` : undefined}>
	<button
		type="button"
		bind:this={trigger}
		class="trigger"
		disabled={disabled}
		onclick={toggle}
		onkeydown={onTriggerKey}
		aria-haspopup="listbox"
		aria-controls="glass-list"
		role="combobox"
		aria-expanded={open}
		aria-label={ariaLabel}
	>
		<span class="value">{current?.label ?? '—'}</span>
		<ChevronDown class="chev h-3.5 w-3.5" />
	</button>
	{#if open}
		<ul
			class="list custom-scrollbar"
			role="listbox"
			id="glass-list"
			onkeydown={onListKey}
			transition:fly={{ y: 4, duration: 160 }}
		>
			{#each options as option (option.value)}
				<li>
					<button
						type="button"
						class="option"
						class:picked={option.value === value}
						role="option"
						aria-selected={option.value === value}
						tabindex="-1"
						onclick={() => pick(option.value)}
					>
						<span>{option.label}</span>
						{#if option.value === value}<Check class="h-3.5 w-3.5" />{/if}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.glass-select { position: relative; display: flex; flex-direction: column; }
	.glass-select .trigger {
		display: flex; align-items: center; justify-content: space-between; gap: 8px;
		min-height: 34px; padding: 6px 11px;
		border: 1px solid transparent;
		border-radius: var(--radius-sm);
		background: rgb(var(--fg) / 0.07);
		color: rgb(var(--fg));
		font-size: 12.5px; font-weight: 600;
		transition: background-color 140ms ease, border-color 140ms ease;
	}
	.glass-select .trigger:hover:not(:disabled) {
		background: rgb(var(--fg) / 0.11);
	}
	.glass-select.open .trigger {
		background: rgb(var(--fg) / 0.11);
		border-color: rgb(var(--fg) / 0.12);
	}
	.glass-select .trigger:disabled { opacity: 0.5; cursor: not-allowed; }
	.glass-select .trigger:focus-visible { outline: 2px solid var(--node-theme); outline-offset: 2px; }
	.value { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-align: left; }
	.glass-select .trigger :global(.chev) { flex: none; color: rgb(var(--fg-subtle)); transition: transform 180ms cubic-bezier(0.34, 1.4, 0.64, 1); }
	.glass-select.open .trigger :global(.chev) { transform: rotate(180deg); color: var(--node-theme); }
	.list {
		position: absolute;
		top: calc(100% + 6px); left: 0; right: 0;
		z-index: 30;
		max-height: 190px;
		display: flex; flex-direction: column; gap: 2px;
		padding: 4px;
		border: 1px solid rgb(var(--fg) / 0.08);
		border-radius: var(--radius-sm);
		background: rgb(var(--bg-elevated));
		box-shadow: var(--shadow-elevated);
		overflow-y: auto;
		overscroll-behavior: contain;
	}
	.option {
		display: flex; align-items: center; justify-content: space-between; gap: 8px;
		width: 100%; padding: 7px 10px;
		border: 1px solid transparent;
		border-radius: var(--radius-xs);
		color: rgb(var(--fg-muted));
		font-size: 12.5px; font-weight: 600;
		text-align: left;
		transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;
	}
	.option:hover, .option:focus-visible {
		color: rgb(var(--fg));
		background: rgb(var(--fg) / 0.06);
		border-color: transparent;
		outline: none;
	}
	.option.picked {
		color: var(--node-theme);
		background: color-mix(in srgb, var(--node-theme) 12%, transparent);
		border-color: transparent;
	}
	.list::-webkit-scrollbar { width: 8px; }
	.list::-webkit-scrollbar-thumb { background: color-mix(in srgb, var(--node-theme) 40%, transparent); border-radius: 999px; }
	.list::-webkit-scrollbar-track { background: transparent; }
</style>
