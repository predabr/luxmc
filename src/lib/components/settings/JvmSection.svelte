<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { FolderOpen, Sparkles } from "lucide-svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { open } from "@tauri-apps/plugin-dialog";

	type Props = {
		onSave: () => void;
	};

	let { onSave }: Props = $props();

	let javaPath = $state(settings.value.javaPath || "/usr/bin/java");
	let minRam = $state(settings.value.minRamMb || 2048);
	let maxRam = $state(settings.value.maxRamMb || 6144);
	let selectedGc = $state("aikar");
	let jvmArgs = $state(settings.value.jvmArgs || "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200");
	let disableExplicitGc = $state(true);
	let parallelRefProc = $state(true);
	let tieredCompilation = $state(true);

	function autoOptimizeRam() {
		minRam = 2048;
		maxRam = 6144;
		selectedGc = "aikar";
		jvmArgs = "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC";
		disableExplicitGc = true;
		parallelRefProc = true;
		tieredCompilation = true;
		toast(uiText("ui.9f0b5c2cf48f2755"), "success");
	}

	async function browseJava() {
		try {
			const selected = await open({ directory: false, multiple: false });
			if (selected && typeof selected === "string") {
				javaPath = selected;
				toast(`Java selecionado: ${selected}`, "success");
			}
		} catch (e) {
			toast(String(e), "error");
		}
	}
</script>

<div class="space-y-4">
	<div class="bg-gradient-to-r from-bg-subtle to-bg-subtle border border-brand-500/30 rounded-2xl p-4 flex items-center justify-between shadow-md">
		<div>
			<div class="text-xs font-black text-fg flex items-center gap-2">
				<Sparkles class="w-4 h-4 text-brand-500" /> {uiText("ui.63768ca5a5cdf09e")}
			</div>
			<div class="text-[10px] text-fg/60 mt-0.5">{uiText("ui.fdea519e22989cb5")}</div>
		</div>
		<button
			class={launcherButton({ variant: "primary", size: "sm", class: "shrink-0 flex items-center gap-1.5" })}
			onclick={autoOptimizeRam}
		>
			<Sparkles class="w-3.5 h-3.5" /> {uiText("ui.79fef7de8bdedcaa")}
		</button>
	</div>

	<div>
		<span class="text-xs font-bold text-fg block mb-1.5">{uiText("ui.c1c15589e45adf21")}</span>
		<div class="flex gap-2">
			<input type="text" bind:value={javaPath} class="flex-1 bg-bg-subtle border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg focus:border-brand-500 focus:outline-none font-mono" />
			<button class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })} onclick={browseJava}>
				<FolderOpen class="w-3.5 h-3.5" /> {uiText("ui.6e7780ca6921c098")}
			</button>
		</div>
	</div>

	<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-4 space-y-3">
		<div class="flex items-center justify-between">
			<span class="text-xs font-bold text-fg">{uiText("ui.3057d26911787222")}</span>
			<span class="text-xs font-mono font-bold text-brand-500">{maxRam} {uiText("ui.5b61e00074c57ecd")}{Math.round(maxRam / 1024)} {uiText("ui.11b5fb43f524845d")}</span>
		</div>

		<input
			type="range"
			min="1024"
			max="16384"
			step="512"
			bind:value={maxRam}
			class="w-full accent-brand-500 cursor-pointer"
		/>

		<div class="flex justify-between text-[10px] text-fg/30 font-mono font-bold">
			<span>{uiText("ui.6c31b63a02708634")}</span>
			<span>{uiText("ui.64a214401e793659")}</span>
			<span>{uiText("ui.64b29b62d14dd300")}</span>
			<span>{uiText("ui.f2d32ef1e01403f8")}</span>
			<span>{uiText("ui.c854bb56c3dc2210")}</span>
		</div>
	</div>

	<div>
		<span class="text-xs font-bold text-fg block mb-1.5">{uiText("ui.41e384be92621e22")}</span>
		<input type="text" bind:value={jvmArgs} class="w-full bg-bg-subtle border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg font-mono focus:border-brand-500 focus:outline-none" />
	</div>

	<div class="space-y-2">
		{#each [
			{ title: uiText("ui.814e6ff13834c4b0"), desc: uiText("ui.a0ed3440aaf90e26"), val: disableExplicitGc, toggle: () => disableExplicitGc = !disableExplicitGc },
			{ title: uiText("ui.6ccbaa16b97ee165"), desc: uiText("ui.ec29fbd6ef1d4fa7"), val: parallelRefProc, toggle: () => parallelRefProc = !parallelRefProc },
			{ title: uiText("ui.b2b82f72b7294992"), desc: uiText("ui.b7f766a83524cb08"), val: tieredCompilation, toggle: () => tieredCompilation = !tieredCompilation }
		] as opt}
			<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3 flex items-center justify-between hover:border-fg/10 transition-[color,background-color,border-color,box-shadow,transform,opacity]">
				<div>
					<div class="text-xs font-bold text-fg">{opt.title}</div>
					<div class="text-[10px] text-fg/40">{opt.desc}</div>
				</div>
				<button
					type="button"
					role="switch"
					aria-label={opt.title}
					aria-checked={opt.val}
					class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-brand-400' : 'bg-bg-subtle'}"
					onclick={opt.toggle}
				>
					<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-bg-subtle' : 'translate-x-0 bg-fg'}"></span>
				</button>
			</div>
		{/each}
	</div>
</div>
