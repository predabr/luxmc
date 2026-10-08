import { api } from './client';
import type { AppSettings } from '$lib/stores/settings.svelte';
import type { InstanceReadiness, CrashDiagnosis } from './doctor';
import type { ModSearchResultItem } from './types';
import { open, save } from '@tauri-apps/plugin-dialog';
export interface InstallationJob { id:string; item:ModSearchResultItem; name:string; ramMb:number; selectedVersion:string; modpackVersionId?:string; status:'queued'|'running'|'paused'|'failed'; error?:string }
export async function installationQueueGet():Promise<unknown> { const settings=await api.invoke<{installationQueue?:unknown}|null>('settings_get'); return settings?.installationQueue ?? []; }
export const installationQueueSave = (jobs:InstallationJob[]):Promise<void> => api.invoke('settings_set',{value:{installationQueue:jobs}});
export const verifyPackDownload = (path:string,size:number,sha1:string):Promise<void> => api.invoke('verify_pack_download',{path,size,sha1});
export async function themeFileChoose():Promise<string|null> { const path=await open({multiple:false,filters:[{name:'Luxmc theme',extensions:['luxtheme']}]}); return typeof path==='string' ? path : null; }
export const themeFileDestination = ():Promise<string|null> => save({defaultPath:'Luxmc.luxtheme',filters:[{name:'Luxmc theme',extensions:['luxtheme']}]});
export const supportFileDestination = ():Promise<string|null> => save({defaultPath:'Luxmc-support.json',filters:[{name:'JSON',extensions:['json']}]});
export interface PerformanceEntry { profileId: string; timestamp: string; metrics: { kind: 'preparation' | 'session' | 'installation'; seconds?: number; phases?:{name:string;seconds:number}[]; pid: number; durationSeconds?: number; peakRamMb?: number; exitCode?: number; cleanExit?: boolean } }
export interface SupportReport { launcherVersion: string; platform: string; readiness: InstanceReadiness; diagnosis: CrashDiagnosis }
export const performanceHistory = (): Promise<PerformanceEntry[]> => api.invoke('performance_history');
export const supportReport = (profileId: string): Promise<SupportReport> => api.invoke('support_report',{profileId});
export const themeExport = (path: string): Promise<void> => api.invoke('theme_export',{path});
export const themeImport = (path: string): Promise<Partial<AppSettings>> => api.invoke('theme_import',{path});
export const restoreWorld = (profileId: string,fileName: string): Promise<void> => api.invoke('world_restore',{profileId,fileName});

export interface Compatibility { mcVersion:string; loader:string; modFingerprint:string; modCount:number }
export const instanceCompatibility = (profileId:string):Promise<Compatibility> => api.invoke("instance_compatibility",{profileId});

export const validateModpack = (profileId:string):Promise<InstanceReadiness> => api.invoke("modpack_validate",{profileId});

export const exportSupportReport = (path:string,report:SupportReport):Promise<void> => api.invoke("support_report_export",{path,report});
