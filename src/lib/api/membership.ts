import { z } from 'zod';
import { api } from './client';

export const membershipSchema = z.object({ plan: z.enum(['free', 'plus', 'pro']), expiresAt: z.number().int().nullable(), revision: z.number().int(), collections: z.number().int().min(6).max(24), presets: z.number().int().min(0).max(20) });
const appearanceSchema = z.object({ density: z.enum(['compact','comfortable','spacious']).optional(), motionStyle: z.enum(['subtle','expressive']).optional(), motionSpeed: z.enum(['fast','balanced']).optional(), wallpaperFps: z.union([z.literal(15),z.literal(24),z.literal(30),z.literal(60)]).optional(), pauseSkinWhileGaming: z.boolean().optional(), preloadContentIcons: z.boolean().optional(), respectReducedMotion: z.boolean().optional(), gifQuality: z.enum(['balanced','high']).optional(), interfaceOpacity: z.number().min(.35).max(1).optional() });
const presetSchema = z.object({ id: z.string().uuid(), name: z.string().min(1).max(40), preferences: appearanceSchema });
const settingsSchema = z.object({ preferences: appearanceSchema, presets: z.array(presetSchema).max(20), revision: z.number().int() });
export type Membership = z.infer<typeof membershipSchema>;
export type AppearancePreferences = z.infer<typeof appearanceSchema>;
export type AppearancePreset = z.infer<typeof presetSchema>;
export async function membershipStatus(accountId: string) {
    return z.object({ identity: z.object({id:z.string().uuid(),username:z.string(),owner:z.boolean()}), membership: membershipSchema, settings: settingsSchema }).parse(await api.invoke('membership_request', { accountId, request: { action: 'status' } }));
}
export async function membershipSync(accountId: string, preferences: AppearancePreferences, presets: AppearancePreset[], revision: number) {
    return z.object({ok:z.literal(true),revision:z.number().int()}).parse(await api.invoke('membership_request', { accountId, request: { action: 'preferences', preferences: appearanceSchema.parse(preferences), presets: z.array(presetSchema).max(20).parse(presets), revision } }));
}
export async function openMembershipPortal(accountId: string, owner = false) {
    const ticket = z.object({path:z.enum(['/planos','/painel-luxmc']),ticket:z.string().regex(/^[a-f0-9]{64}$/)}).parse(await api.invoke('membership_request', {accountId,request:{action:'portal_ticket',owner}}));
    const url = `https://luxmc-r92.pages.dev${ticket.path}#ticket=${ticket.ticket}`;
    if (typeof window !== 'undefined' && window.electronAPI) await api.invoke('open_url', {url});
    else { const { openUrl } = await import('@tauri-apps/plugin-opener'); await openUrl(url); }
}
