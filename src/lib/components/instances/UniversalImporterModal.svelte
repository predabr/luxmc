<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { instanceExportShareCode } from "$lib/api/instances";
	import { 
		Download, 
		Package, 
		Check, 
		RefreshCw, 
		HardDrive, 
		ExternalLink, 
		Layers, 
		Boxes, 
		FolderOpen,
		Sparkles
	} from "lucide-svelte";
	import { api } from "$lib/api/client";
    import { chooseImportDirectory } from '$lib/api/importer';
	import { profiles } from "$lib/stores/profiles.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import Modal from "$lib/components/ui/Modal.svelte";
	import Button from "$lib/components/ui/Button.svelte";

	type ExternalInstance = {
		launcher: string;
		name: string;
		path: string;
		mcVersion: string;
		loader: string;
		modCount: number;
		hasSaves: boolean;
	};

	type ImportResult = {
		success: boolean;
		profileId: string;
		name: string;
		message: string;
	};

	type Props = {
		open: boolean;
		onClose: () => void;
	};

	let { open, onClose }: Props = $props();

	let detected = $state<ExternalInstance[]>([]);
	let loading = $state(false);
	let importingPath = $state<string | null>(null);
    let manual = $state<ExternalInstance | null>(null);
    let choosing = $state(false);
    async function chooseFolder() {
        if (choosing) return;
        choosing = true;
        try { manual = await chooseImportDirectory(); }
        catch (cause) { toast(String(cause), 'error'); }
        finally { choosing = false; }
    }
    let exportProfileId = $state("");
    let exportCode = $state("");
    let exporting = $state(false);
    async function generateCode() {
        if (!exportProfileId || exporting) return;
        exporting = true; exportCode = "";
        try { exportCode = await instanceExportShareCode(exportProfileId); await navigator.clipboard.writeText(exportCode); toast(uiText("ui.d77fa15de787fb41"), "success"); }
        catch (error) { toast(String(error), "error"); }
        finally { exporting = false; }
    }

	async function scanLaunchers() {
		loading = true;
		try {
			detected = await api.invoke<ExternalInstance[]>("importer_detect_launchers");
		} catch (e) {
			toast(uiText("ui.88f8071333396051", {arg0: (e)}), "error");
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (open) {
			scanLaunchers();
		}
	});

	async function handleImport(inst: ExternalInstance) {
        if (!inst.mcVersion.trim()) { manual = {...inst}; return; }
		importingPath = inst.path;
		toast(uiText("ui.c17c0c7d49498ad3", {arg0: (inst.name), arg1: (inst.launcher)}), "info");

		try {
			const res = await api.invoke<ImportResult>("importer_execute_import", {
				sourcePath: inst.path,
				targetName: inst.name,
				mcVersion: inst.mcVersion,
				loader: inst.loader
			});

			if (res.success) {
				toast(res.message, "success");
				await profiles.refresh();
				onClose();
			}
		} catch (e) {
			toast(uiText("ui.08ecb8adbe4e3173", {arg0: (e)}), "error");
		} finally {
			importingPath = null;
		}
	}
</script>

<Modal isOpen={open} {onClose} title={uiText("ui.3818d977aa31f1ca")} maxWidth="max-w-4xl">
	<div class="flex flex-col gap-5 text-sm">
        <section class="space-y-3 rounded-2xl border border-border bg-bg-subtle p-5">
            <h3 class="font-semibold text-fg">{uiText('importFolder.title')}</h3>
            <p class="text-xs leading-relaxed text-fg-muted">{uiText('importFolder.description')}</p>
            <Button variant="secondary" disabled={choosing || importingPath !== null} onclick={chooseFolder}><FolderOpen class="h-4 w-4" />{uiText('importFolder.choose')}</Button>
            {#if manual}
                <p class="break-all text-xs text-fg-muted">{manual.path}</p>
                <div class="grid gap-3 sm:grid-cols-3">
                    <label class="text-xs">{uiText('importFolder.name')}<input bind:value={manual.name} class="mt-2 w-full rounded-xl border border-border bg-bg-elevated p-3" /></label>
                    <label class="text-xs">{uiText('importFolder.version')}<input bind:value={manual.mcVersion} placeholder="1.21.1" class="mt-2 w-full rounded-xl border border-border bg-bg-elevated p-3" /></label>
                    <label class="text-xs">Loader<select bind:value={manual.loader} class="mt-2 w-full rounded-xl border border-border bg-bg-elevated p-3">{#each ['vanilla','fabric','quilt','forge','neoforge'] as loader}<option value={loader}>{loader}</option>{/each}</select></label>
                </div>
                <Button variant="primary" disabled={!manual.mcVersion.trim() || importingPath !== null} onclick={() => manual && handleImport(manual)}><Download class="h-4 w-4" />{uiText('importFolder.import')}</Button>
            {/if}
        </section>
        <section class="rounded-2xl border border-border p-5 space-y-3" aria-label={uiText("ui.2b3376fa2c123da2")}>
            <h3 class="font-semibold text-fg">{uiText("ui.27be9ad2e803a73e")}</h3><p class="text-xs text-fg-muted">{uiText("ui.e349f5e5f06cf04d")}</p>
            <div class="flex flex-wrap gap-3"><select class="min-w-52 flex-1 rounded-xl border border-border bg-bg-elevated p-3" aria-label={uiText("ui.58b7180bb0814fda")} bind:value={exportProfileId}><option value="">{uiText("ui.2febc7ba48f6c044")}</option>{#each profiles.list as profile}<option value={profile.id}>{profile.name}</option>{/each}</select><Button variant="primary" size="lg" disabled={!exportProfileId || exporting} onclick={generateCode}>{exporting ? uiText("ui.7bf220a3c328954e") : uiText("ui.dde91d7f432a3e27")}</Button></div>
            {#if exportCode}<p role="status" class="select-all font-mono text-lg text-brand-400">{exportCode}</p>{/if}
        </section>
		<div class="flex flex-wrap items-center justify-between">
			<p class="text-fg/70 leading-relaxed max-w-md">
				{uiText("ui.062d042634c7d773")} <strong>{uiText("ui.fbbb5ae720226427")}</strong>, <strong>CurseForge</strong>, <strong>{uiText("ui.8723cacec4d2edc1")}</strong> {uiText("ui.3f79bb7b435b0532")} <strong>{uiText("ui.e63df56675e900cb")}</strong> {uiText("ui.74d58ece5aa3081a")}
			</p>
			<Button variant="secondary" size="lg" disabled={loading} onclick={scanLaunchers}>
				<RefreshCw class="w-3.5 h-3.5 {loading ? 'animate-spin' : ''}" />
				{uiText("ui.e7581d76fe678c99")}
			</Button>
		</div>

		{#if loading}
			<div class="py-12 flex flex-col items-center justify-center gap-2 text-fg/50">
				<RefreshCw class="w-6 h-6 animate-spin text-brand-400" />
				<span>{uiText("ui.32c151f48cb7fc78")}</span>
			</div>
		{:else if detected.length === 0}
			<div class="py-10 flex flex-col items-center justify-center gap-2 text-center bg-bg-subtle rounded-2xl border border-fg/5 p-6">
				<HardDrive class="w-8 h-8 text-fg/30" />
				<h4 class="text-fg font-bold text-sm">{uiText("ui.205501bfb972dc53")}</h4>
				<p class="text-fg/50 text-[11px] max-w-xs">
					{uiText("ui.78d13dba66dde833")}
				</p>
			</div>
		{:else}
			<div class="flex flex-col gap-2.5 max-h-[50vh] overflow-y-auto custom-scrollbar pr-1">
				{#each detected as inst}
					<div class="bg-bg-subtle border border-fg/10 hover:border-brand-400/40 rounded-2xl p-3.5 flex flex-wrap items-center justify-between gap-3 transition">
						<div class="flex items-center gap-3 min-w-0">
							<div class="w-10 h-10 rounded-xl bg-fg/5 border border-fg/10 flex items-center justify-center shrink-0">
								<Boxes class="w-5 h-5 text-brand-400" />
							</div>
							<div class="min-w-0">
								<div class="flex items-center gap-2">
									<h4 class="text-fg font-bold text-xs truncate">{inst.name}</h4>
									<span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-brand-400/15 text-brand-400 border border-brand-400/30 shrink-0">
										{inst.launcher}
									</span>
								</div>
								<div class="flex items-center gap-3 text-[11px] text-fg/50 mt-1 font-mono">
									<span>{uiText("ui.ca34ab5c748c6607")} {inst.mcVersion}</span>
									<span>·</span>
									<span class="capitalize">{inst.loader}</span>
									{#if inst.modCount > 0}
										<span>·</span>
										<span>{inst.modCount} {uiText("ui.695073cb6649c0a4")}</span>
									{/if}
									{#if inst.hasSaves}
										<span>·</span>
										<span class="text-emerald-400 font-sans font-semibold">{uiText("ui.65d3f884ebda33f7")}</span>
									{/if}
								</div>
							</div>
						</div>

						<Button 
							variant="primary" 
							size="lg" 
							disabled={importingPath !== null} 
							onclick={() => handleImport(inst)}
						>
							{#if importingPath === inst.path}
								<RefreshCw class="w-3.5 h-3.5 animate-spin" />
								<span>{uiText("ui.eeb8c8bbcaab66e1")}</span>
							{:else}
								<Download class="w-3.5 h-3.5" />
								<span>{uiText("settings.import")}</span>
							{/if}
						</Button>
					</div>
				{/each}
			</div>
		{/if}

		<div class="flex items-center justify-end pt-3 border-t border-fg/5">
			<Button variant="secondary" size="lg" onclick={onClose}>
				{uiText("statusBanner.dismiss")}
			</Button>
		</div>
	</div>
</Modal>
