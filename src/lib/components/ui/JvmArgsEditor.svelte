<script lang="ts">
	import { Cog, ShieldAlert, Sparkles, X } from "lucide-svelte";
	import { jvmArgsValidate, type JvmValidationResult } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

	type Props = {
		value: string;
		ramMb: number | null;
		onChange?: (jvmArgs: string, ramMb: number) => void;
	};

	let { value = $bindable(""), ramMb = $bindable<number | null>(2048), onChange }: Props = $props();

	let validation = $state<JvmValidationResult | null>(null);
	let validating = $state(false);
	let lastChecked = $state("");

	$effect(() => {
		if (value !== lastChecked) {
			validate();
		}
	});

	async function validate() {
		if (!value.trim()) {
			validation = { valid: true, rejected: [], normalized: "", suggestions: [] };
			return;
		}
		validating = true;
		try {
			validation = await jvmArgsValidate(value);
			lastChecked = value;
			if (validation.valid && validation.normalized !== value) {
				value = validation.normalized;
			}
		} catch (e) {
			validation = null;
			toast(t("jvmEditor.validationFailed", { error: String(e) }), "error");
		} finally {
			validating = false;
		}
	}

	function pickRam(preset: number) {
		ramMb = preset;
	}

	function clear() {
		value = "";
		validation = { valid: true, rejected: [], normalized: "", suggestions: [] };
		lastChecked = "";
	}

	function notify() {
		onChange?.(value, ramMb ?? 0);
	}
</script>

<div class="flex flex-col gap-3">
	<div class="flex items-center justify-between">
		<label for="ram-mb" class="text-xs font-medium" style="color: rgb(var(--fg-muted));">
			{t("jvmEditor.ram")}
		</label>
		<span class="text-xs" style="color: rgb(var(--fg-subtle));">
			{((ramMb ?? 0) / 1024).toFixed(1)} GB
		</span>
	</div>
	<input
		id="ram-mb"
		type="number"
		min="512"
		max="32768"
		step="256"
		class="h-9 w-full rounded-md px-2 text-sm outline-none"
		style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg)); color: rgb(var(--fg));"
		bind:value={ramMb}
		onchange={notify}
	/>
	<div class="flex flex-wrap gap-1.5">
		{#each [1024, 2048, 3072, 4096, 6144, 8192, 12288, 16384] as preset (preset)}
			<button
				type="button"
				class="rounded-md px-2 py-1 text-[11px] transition-colors"
				style="border: 1px solid rgb(var(--border)); color: {ramMb === preset
					? 'rgb(var(--brand-400))'
					: 'rgb(var(--fg-muted))'}; background: {ramMb === preset
					? 'rgba(45, 212, 191, 0.1)'
					: 'transparent'};"
				onclick={() => {
					pickRam(preset);
					notify();
				}}
			>
				{preset / 1024} GB
			</button>
		{/each}
	</div>

	<div class="mt-2 flex items-center justify-between">
		<label for="jvm-args" class="flex items-center gap-1.5 text-xs font-medium" style="color: rgb(var(--fg-muted));">
			<Cog class="h-3.5 w-3.5" />
			{t("jvmEditor.customArgs")}
		</label>
		{#if value}
			<button
				type="button"
				class="grid h-6 w-6 place-items-center rounded transition-colors hover:bg-white/5"
				style="color: rgb(var(--fg-subtle));"
				onclick={clear}
				aria-label={t("jvmEditor.clearArgs")}
			>
				<X class="h-3 w-3" />
			</button>
		{/if}
	</div>
	<textarea
		id="jvm-args"
		class="min-h-[5rem] w-full resize-y rounded-md p-2 font-mono text-xs outline-none"
		style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg)); color: rgb(var(--fg));"
		placeholder="-XX:+UseG1GC -XX:MaxGCPauseMillis=50 -Dfile.encoding=UTF-8"
		bind:value
		oninput={notify}
	></textarea>

	{#if validating}
		<p class="flex items-center gap-1.5 text-[11px]" style="color: rgb(var(--fg-subtle));">
			<Sparkles class="h-3 w-3 animate-pulse" />
			{t("jvmEditor.validating")}
		</p>
	{:else if validation && validation.rejected.length > 0}
		<div
			class="rounded-md px-3 py-2 text-[11px]"
			style="border: 1px solid rgba(239, 68, 68, 0.3); background: rgba(239, 68, 68, 0.08); color: rgb(252, 165, 165);"
		>
			<div class="mb-1 flex items-center gap-1.5 font-medium">
				<ShieldAlert class="h-3 w-3" />
				{t("jvmEditor.flagsRejected", { count: validation.rejected.length })}
			</div>
			{#each validation.suggestions as s, i (i)}
				<p class="text-[10px]">· {s}</p>
			{/each}
		</div>
	{:else if validation && validation.valid}
		<p class="flex items-center gap-1.5 text-[11px]" style="color: rgb(74, 222, 128);">
			<Sparkles class="h-3 w-3" />
			{t("jvmEditor.allValid")}
		</p>
	{/if}
</div>
