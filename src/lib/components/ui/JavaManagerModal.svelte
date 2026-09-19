<script lang="ts">
	import { onMount } from "svelte";
	import { X, Coffee, Download, Trash2, CheckCircle, AlertCircle, RefreshCw, ExternalLink } from "lucide-svelte";
	import { javaScan, javaInstall, javaUninstall } from "$lib/api/java";
	import type { JavaInstallStatus } from "$lib/api/types";
	import { toast } from "$lib/stores/toasts.svelte";
	import { listen } from "$lib/api/client";

	type Props = { open?: boolean; onClose?: () => void };
	let { open = false, onClose }: Props = $props();

	let runtimes = $state<JavaInstallStatus[]>([]);
	let loading = $state(true);
	let installing = $state<number | null>(null);
	let installProgress = $state(0);

	const javaInfo: Record<number, { label: string; use: string; color: string }> = {
		8:  { label: "Java 8",  use: "Minecraft 1.7 – 1.16.x",  color: "text-orange-400" },
		17: { label: "Java 17", use: "Minecraft 1.17 – 1.20.4",  color: "text-blue-400" },
		21: { label: "Java 21", use: "Minecraft 1.20.5+",        color: "text-emerald-400" },
	};

	onMount(() => {
		scan();
	});

	async function scan() {
		loading = true;
		try {
			const result = await javaScan();
			runtimes = result.runtimes;
		} catch {
			toast("Erro ao verificar runtimes Java", "error");
		} finally {
			loading = false;
		}
	}

	async function install(major: number) {
		installing = major;
		installProgress = 0;

		const unlisten = await listen<{ phase: string; completed: number; total: number }>(
			"download-progress",
			(ev) => {
				if (ev.payload.total > 0) {
					installProgress = Math.round((ev.payload.completed / ev.payload.total) * 100);
				}
			}
		);

		try {
			const result = await javaInstall(major);
			runtimes = runtimes.map((r) => (r.major === major ? result : r));
			toast(`Java ${major} instalado com sucesso!`, "success");
		} catch (e) {
			toast(`Erro ao instalar Java ${major}: ${e}`, "error");
		} finally {
			installing = null;
			installProgress = 0;
			unlisten();
		}
	}

	async function uninstall(major: number) {
		try {
			await javaUninstall(major);
			runtimes = runtimes.map((r) => r.major === major ? { ...r, installed: false, path: null, versionString: null, isSystem: false } : r);
			toast(`Java ${major} removido`, "info");
		} catch (e) {
			toast(`Erro ao remover Java ${major}: ${e}`, "error");
		}
	}
</script>

{#if open}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/70 backdrop-blur-md">
	<div class="w-full max-w-lg bg-bg-elevated border border-fg/10 rounded-3xl shadow-2xl flex flex-col gap-0 overflow-hidden animate-slide-up">
		<div class="flex items-center justify-between px-6 py-4 border-b border-fg/5">
			<div class="flex items-center gap-3">
				<div class="w-9 h-9 rounded-2xl bg-orange-500/15 border border-orange-500/25 flex items-center justify-center text-orange-400">
					<Coffee class="w-4 h-4" />
				</div>
				<div>
					<h2 class="text-sm font-black text-fg">Gerenciador de Java</h2>
					<p class="text-[11px] text-fg/40">Detectar e instalar runtimes automaticamente</p>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<button type="button" onclick={scan} class="p-2 rounded-xl text-fg/40 hover:text-fg hover:bg-fg/5 transition-colors cursor-pointer" title="Verificar novamente">
					<RefreshCw class="w-4 h-4 {loading ? 'animate-spin' : ''}" />
				</button>
				<button type="button" onclick={onClose} class="p-2 rounded-xl text-fg/40 hover:text-fg hover:bg-fg/5 transition-colors cursor-pointer">
					<X class="w-4 h-4" />
				</button>
			</div>
		</div>

		<div class="p-6 flex flex-col gap-3">
			{#if loading}
				{#each [8, 17, 21] as _}
					<div class="h-20 rounded-2xl bg-fg/5 animate-pulse"></div>
				{/each}
			{:else}
				{#each runtimes as runtime}
					{@const info = javaInfo[runtime.major]}
					<div class="flex items-center justify-between gap-4 p-4 rounded-2xl border {runtime.installed ? 'border-emerald-500/20 bg-emerald-500/5' : 'border-fg/5 bg-bg-subtle'}">
						<div class="flex items-center gap-3 min-w-0">
							<div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0 {runtime.installed ? 'bg-emerald-500/15 border border-emerald-500/25' : 'bg-fg/5 border border-fg/10'}">
								{#if runtime.installed}
									<CheckCircle class="w-4 h-4 text-emerald-400" />
								{:else}
									<AlertCircle class="w-4 h-4 text-fg/30" />
								{/if}
							</div>
							<div class="min-w-0">
								<div class="flex items-center gap-2">
									<span class="text-sm font-bold {info?.color ?? 'text-fg'}">{info?.label}</span>
									{#if runtime.isSystem}
										<span class="text-[9px] px-1.5 py-0.5 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20 font-bold">SISTEMA</span>
									{:else if runtime.installed}
										<span class="text-[9px] px-1.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-bold">GERENCIADO</span>
									{/if}
								</div>
								<p class="text-[11px] text-fg/40 truncate">{info?.use}</p>
								{#if runtime.versionString}
									<p class="text-[10px] text-fg/30 font-mono truncate mt-0.5">{runtime.versionString}</p>
								{/if}
							</div>
						</div>

						<div class="flex items-center gap-2 shrink-0">
							{#if installing === runtime.major}
								<div class="flex flex-col items-end gap-1.5 w-24">
									<span class="text-[10px] text-fg/50">Instalando… {installProgress}%</span>
									<div class="w-full h-1.5 rounded-full bg-fg/10 overflow-hidden">
										<div class="h-full bg-brand-500 rounded-full transition-all duration-300" style="width:{installProgress}%"></div>
									</div>
								</div>
							{:else if !runtime.installed}
								<button
									type="button"
									onclick={() => install(runtime.major)}
									class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-brand-500/15 hover:bg-brand-500/25 border border-brand-500/30 text-brand-400 text-[11px] font-bold transition-all cursor-pointer"
								>
									<Download class="w-3.5 h-3.5" />
									Instalar
								</button>
							{:else if !runtime.isSystem}
								<button
									type="button"
									onclick={() => uninstall(runtime.major)}
									class="p-2 rounded-xl text-fg/30 hover:text-danger hover:bg-danger/10 border border-fg/5 hover:border-danger/20 transition-all cursor-pointer"
									title="Remover runtime gerenciado"
								>
									<Trash2 class="w-3.5 h-3.5" />
								</button>
							{/if}
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<div class="px-6 pb-5 flex items-center justify-between">
			<a
				href="https://adoptium.net/temurin/releases/"
				target="_blank"
				rel="noopener noreferrer"
				class="flex items-center gap-1.5 text-[11px] text-fg/30 hover:text-fg/70 transition-colors"
			>
				<ExternalLink class="w-3 h-3" />
				Adoptium Temurin
			</a>
			<p class="text-[11px] text-fg/30">Instalação automática via Mojang Launcher Meta</p>
		</div>
	</div>
</div>
{/if}
