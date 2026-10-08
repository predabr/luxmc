import { api } from './client';
import type { PackEntry } from './social';
import type { ModVersion } from './types';
export type PreparationPhase = 'idle'|'planning'|'downloading'|'validating'|'ready'|'failed';
export interface Preparation { phase:PreparationPhase; percent:number; ownerId:string; fingerprint:string }
export interface RoomRecipe { name:string; mcVersion:string; loader:string; loaderVersion:string|null; fingerprint:string; mods:PackEntry[]; pack:PackEntry|null; unavailable:string[] }
export interface DependencyNode { id:string; name:string; file:string; version:string; requires:string[]; provides:string[]; requiredBy:string[]; missing:string[]; metadataKnown:boolean }
export interface Recommendation { profileId:string; name:string; icon:string; samples:number; peakRamMb:number; preparationSeconds:number|null; fitsNow:boolean; suggestedRamMb:number }
export interface PcRecommendations { totalRamMb:number; availableRamMb:number; budgetMb:number; items:Recommendation[] }
export const dependencyGraph = (profileId:string):Promise<DependencyNode[]> => api.invoke('dependency_graph',{profileId});
export const pcRecommendations = ():Promise<PcRecommendations> => api.invoke('pc_recommendations');
export const launcherResourceSample = ():Promise<{ramMb:number;cpuPercent:number;processes:number;sampleMilliseconds:number}> => api.invoke('launcher_resource_sample');
export const instancePackReference = (profileId:string):Promise<PackEntry|null> => api.invoke('instance_pack_reference',{profileId});
export const packReferenceVersion = (reference:PackEntry):Promise<ModVersion> => api.invoke('pack_reference_version',{reference});
export const roomRecipe = (ownerId:string):Promise<RoomRecipe> => api.invoke('tunnel_room_recipe',{ownerId});
export const roomPrepare = (ownerId:string):Promise<{id:string;name:string}> => api.invoke('room_prepare_instance',{ownerId});
export const roomPreparationPublish = (value:Preparation):Promise<void> => api.invoke('tunnel_set_preparation',{value});
