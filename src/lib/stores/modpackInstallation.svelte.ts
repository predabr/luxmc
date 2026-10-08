import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { modsVersions, modsDownloadToTemp, instanceImportMrpack, instanceImportModpack, instanceCancelImport, type ModSearchResultItem } from "$lib/api";
import { listen } from "$lib/api/client";
import { profiles } from "$lib/stores/profiles.svelte";
import { toast } from "$lib/stores/toasts.svelte";
import { fireModpackSuccessConfetti } from "$lib/utils/confetti";
import { installationQueueGet, installationQueueSave, verifyPackDownload, type InstallationJob as Job } from '$lib/api/experience';
import { z } from 'zod';
import { packReferenceVersion } from '$lib/api/studio';
let jobs = $state<Job[]>([]);
let current: Job | null = null;
let paused = false;
let initialization: Promise<void> | null = null;
let persistence = Promise.resolve();
function persist() {
    const snapshot = $state.snapshot(jobs);
    persistence = persistence.catch(() => {}).then(() => installationQueueSave(snapshot));
    void persistence.catch(error => toast(String(error), 'error'));
    return persistence;
}
async function initialize(): Promise<void> {
    if (!initialization) initialization=loadJobs();
    try { await initialization; } catch(error) { initialization=null; throw error; }
}
async function loadJobs() {
    try {
        const saved = await installationQueueGet();
        const schema = z.array(z.object({id:z.string(),name:z.string().min(1).max(200),ramMb:z.number().min(512).max(65536),selectedVersion:z.string(),modpackVersionId:z.string().optional(),status:z.enum(['queued','running','paused','failed']),error:z.string().optional(),item:z.object({sourceId:z.string(),source:z.string(),title:z.string(),slug:z.string(),description:z.string(),downloads:z.number(),author:z.string().nullable(),versions:z.array(z.string()),categories:z.array(z.string()),iconUrl:z.string().nullable(),bannerUrl:z.string().nullable()}).passthrough()})).max(50);
        const result = schema.safeParse(saved);
        if (!result.success) throw new Error(uiText('workshop.invalidQueue'));
        if (result.success) jobs = result.data.map(job => ({...job,status:job.status === 'failed' ? 'failed' : 'paused'})) as Job[];
    } catch (error) { toast(String(error), 'error'); throw error; }
}
async function pump() {
    if (operation.busy || current) return;
    const next = jobs.find(job => job.status === 'queued');
    if (!next) return;
    current = next; paused = false; next.status = 'running'; persist();
    await run(next.item,next.name,next.ramMb,next.selectedVersion,next.modpackVersionId);
    if (paused) next.status = 'paused';
    else if (operation.error) { next.status = 'failed'; next.error = operation.error; }
    else jobs = jobs.filter(job => job.id !== next.id);
    current = null; persist();
    if (!paused) void pump();
}
async function start(item: ModSearchResultItem, name: string, ramMb: number, selectedVersion: string, modpackVersionId?: string) {
    try { await initialize(); } catch { return; }
    if (jobs.length >= 50) { toast(uiText('workshop.queueFull'), 'error'); return; }
    const id=crypto.randomUUID();
    jobs.push({id,item,name,ramMb,selectedVersion,modpackVersionId,status:'queued'});
    try { await persist(); void pump(); } catch(error) { const job=jobs.find(job => job.id===id); if(job) {job.status='failed';job.error=String(error);} }
}
const operation = $state({ paused: false, busy: false, cancelling: false, item: null as ModSearchResultItem | null, name: '', text: '', percent: 0, error: '', profileId: '' });
	async function run(item: ModSearchResultItem, name: string, ramMb: number, selectedVersion: string, modpackVersionId?: string) {
		if (operation.busy) return;
        operation.paused = false; operation.item = item; operation.name = name; operation.error = ""; operation.profileId = "";

		operation.cancelling = false;
        operation.busy = true; operation.text = uiText("ui.5a4fa7bb57930284");
		operation.percent = 0;
		let unlisten: (() => void) | null = null;
		try {
			unlisten = await listen<{ phase: string; current: number; total: number; percent?: number; status: string }>("modpack-progress", (ev) => {
				operation.text = ev.payload.status;
				operation.percent = ev.payload.percent ?? 0;
			});
			const ver = selectedVersion === "Qualquer Versão" || modpackVersionId ? "" : selectedVersion;
			let versions = modpackVersionId ? [await packReferenceVersion({source:item.source as 'modrinth'|'curseforge',projectId:item.sourceId,versionId:modpackVersionId,name:item.title})] : await modsVersions(item.sourceId, ver, item.source);
			if (modpackVersionId) versions = versions.filter(version => version.id === modpackVersionId);
            if (versions.length === 0) throw new Error(uiText("ui.c38446c7df13fc52", {arg0:item.title,arg1:item.source === 'curseforge' ? 'CurseForge' : 'Modrinth'}));
			const extension = item.source === "curseforge" ? ".zip" : ".mrpack";
            const f = versions.flatMap(version => version.files).find(file => file.url && file.filename.toLowerCase().endsWith(extension))
				|| versions.flatMap(version => version.files).find(file => file.url && (file.filename.toLowerCase().endsWith(".zip") || file.filename.toLowerCase().endsWith(".mrpack")));
			if (!f?.url) throw new Error(uiText("ui.ebe5f9d75823b7c4"));
            if (current && !current.modpackVersionId) { current.modpackVersionId = versions.find(version => version.files.includes(f))?.id; persist(); }
			operation.text = uiText("ui.18333ba6ce43048e", {arg0: (f.filename)});
			operation.percent = -1;
			if (operation.cancelling) throw new Error(uiText("ui.bb9811b14ec4fb48"));
            const tempPath = await modsDownloadToTemp(f.url, current ? `${current.id}${f.filename.toLowerCase().endsWith(".mrpack") ? ".mrpack" : ".zip"}` : f.filename);
            await verifyPackDownload(tempPath,f.size || 0,f.sha1 || '');
            if (operation.cancelling) throw new Error(uiText("ui.bb9811b14ec4fb48"));
			operation.text = uiText("ui.cbc82784c96e08ef");
			operation.percent = 0;
			const iconUrl = item.iconUrl || "";
			const detectedLoader = item.categories?.find(c => ["forge", "fabric", "neoforge", "quilt"].includes(c.toLowerCase()))?.toLowerCase()
				|| (item.title.toLowerCase().includes("forge") && !item.title.toLowerCase().includes("neoforge") ? "forge" : "")
				|| (item.title.toLowerCase().includes("neoforge") ? "neoforge" : "")
				|| (item.title.toLowerCase().includes("fabric") ? "fabric" : "");
			const cp = !f.filename.toLowerCase().endsWith(".mrpack")
				? await instanceImportModpack(tempPath, name, item.versions[0] || "1.20.1", detectedLoader, iconUrl, ramMb)
				: await instanceImportMrpack(tempPath, name, iconUrl, ramMb);
			profiles.add({
				id: cp.id,
				name: cp.name,
				icon: iconUrl || "default",
				banner: item.bannerUrl || undefined,
				mcVersion: cp.mcVersion,
				loader: (cp.loader || detectedLoader || "fabric") as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				loaderVersion: cp.loaderVersion ?? undefined,
				gameDir: cp.gameDir,
				ramMb: ramMb,
				createdAt: Date.now(),
				updatedAt: Date.now(),
			});
			if (item.bannerUrl) {
				profiles.setBanner(cp.id, item.bannerUrl);
			}
			profiles.activeId = cp.id;

			operation.profileId = cp.id; operation.percent = 100; operation.text = uiText("downloadsDesign.complete");
            void profiles.refresh().catch(() => {});
			fireModpackSuccessConfetti();
			toast(uiText("ui.1b6566167e91c704", {arg0: (name)}), "success");


		} catch (e) {
            if (paused) { operation.paused=true; operation.text=uiText('workshop.paused'); return; }
			operation.error = String(e);
            const cancelled = operation.cancelling || String(e).toLowerCase().includes("cancelad");
			toast(cancelled ? uiText("ui.e6c50b7bfe042bcf") : uiText("ui.66f69086d7d99519", {arg0: (e instanceof Error ? e.message : String(e))}), cancelled ? "info" : "error");
		} finally {
			if (unlisten) unlisten();
			operation.busy = false; operation.cancelling = false;
		}
	}

async function cancel() {
    if (!operation.busy || operation.cancelling) return;
    operation.cancelling = true;
    try { await instanceCancelImport(); } catch (error) { operation.cancelling = false; toast(String(error), 'error'); }
}
export const modpackInstallation = {
    get state() { return operation; }, get jobs() { return jobs; }, start, initialize,
    async cancel() { if (current) { jobs = jobs.filter(job => job.id !== current!.id); persist(); } await cancel(); },
    async pause() { if (!current) return; paused = true; await cancel(); },
    resume(id: string) { const job = jobs.find(job => job.id === id); if (job && job.status !== 'running') { job.status='queued'; job.error=''; persist(); void pump(); } },
    prioritize(id: string) { const index = jobs.findIndex(job => job.id === id && job.status !== 'running'); if (index >= 0) { const [job] = jobs.splice(index,1); jobs.unshift(job); persist(); } },
    remove(id: string) { if (current?.id === id) { void this.cancel(); return; } jobs = jobs.filter(job => job.id !== id); persist(); },
    dismiss() { if (!operation.busy) operation.item = null; }
};
