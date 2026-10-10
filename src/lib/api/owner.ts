import { api } from './client';
import { z } from 'zod';
export const ownerTools = (accountId: string, profileId: string, operation: 'diagnose' | 'repair'): Promise<Record<string, unknown>> => api.invoke('owner_tools', { accountId, profileId, operation });

const ownerUserSchema = z.object({id:z.string(),username:z.string(),displayName:z.string(),avatarUrl:z.string().nullable(),status:z.enum(['online','in_game','offline']),lastSeen:z.string().nullable(),tags:z.array(z.string().max(24)).max(5)});
const ownerOverviewSchema = z.object({stats:z.object({total:z.number(),online:z.number(),playing:z.number()}),users:z.array(ownerUserSchema).max(24),total:z.number(),nextOffset:z.number().nullable()});
export type OwnerUser = z.infer<typeof ownerUserSchema>;
export type OwnerOverview = z.infer<typeof ownerOverviewSchema>;
export async function ownerUsers(accountId:string,query='',offset=0):Promise<OwnerOverview> {
    return ownerOverviewSchema.parse(await api.invoke('social_request',{accountId,request:{action:'owner_users',query,offset}}));
}
export async function ownerSetTags(accountId:string,targetId:string,tags:string[]):Promise<string[]> {
    return z.object({ok:z.literal(true),tags:z.array(z.string()).max(5)}).parse(await api.invoke('social_request',{accountId,request:{action:'owner_tags',targetId,tags}})).tags;
}
export async function ownerDeleteProfile(accountId:string,targetId:string,confirmation:string):Promise<void> {
    z.object({ok:z.literal(true)}).parse(await api.invoke('social_request',{accountId,request:{action:'owner_delete',targetId,confirmation}}));
}
