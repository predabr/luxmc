import { parseProfileStyle, type ProfileStyle } from './profile-style.js';

export interface PlayerProfile {
    profileStyle: ProfileStyle;
    collections: PackCollection[];
    displayName: string;
    status: string;
    description: string;
    banner: string;
    portrait: string;
    packs: string[];
    packArtwork?: PackArtwork[];
}
export interface PackArtwork { name: string; title: string; icon: string; kind?: 'mod' | 'modpack' }
export function packArtwork(value: unknown, names: string[]): PackArtwork[] | null {
    if (!Array.isArray(value) || value.length > 24) return null;
    const used = new Set<string>();
    const result: PackArtwork[] = [];
    for (const item of value) {
        if (!item || typeof item !== 'object' || typeof item.name !== 'string' || !names.includes(item.name) || used.has(item.name) || typeof item.title !== 'string' || !item.title.trim() || item.title.length > 80 || typeof item.icon !== 'string' || item.icon.length > 2048) return null;
        try {
            const url = new URL(item.icon);
            if (url.protocol !== 'https:' || url.username || url.password || url.port || !['cdn.modrinth.com','media.forgecdn.net','mediafilez.forgecdn.net','edge.forgecdn.net'].includes(url.hostname)) return null;
        } catch { return null; }
        if (item.kind !== undefined && !['mod','modpack'].includes(item.kind)) return null;
        used.add(item.name);
        result.push({name:item.name,title:item.title.trim(),icon:item.icon,...(item.kind ? {kind:item.kind} : {})});
    }
    return result;
}
export interface PackEntry { source:'modrinth'|'curseforge'; projectId:string; versionId:string; name:string }
export interface PackCollection { id:string; title:string; description:string; entries:PackEntry[] }
export function collections(value:unknown, maximum = 6): PackCollection[] | null {
    if (value === undefined) return [];
    if (!Array.isArray(value) || value.length>maximum) return null;
    const ids = new Set<string>();
    for (const item of value) {
        if (!item || typeof item!=='object' || typeof item.id!=='string' || !/^[a-zA-Z0-9-]{1,64}$/.test(item.id) || ids.has(item.id) || typeof item.title!=='string' || !item.title.trim() || item.title.length>80 || typeof item.description!=='string' || item.description.length>240 || !Array.isArray(item.entries) || item.entries.length>8) return null;
        ids.add(item.id);
        for (const entry of item.entries) {
            if (!entry || !['modrinth','curseforge'].includes(entry.source) || typeof entry.name!=='string' || !entry.name.trim() || entry.name.length>200 || ![entry.projectId,entry.versionId].every(id=>typeof id==='string' && /^[a-zA-Z0-9-]{1,80}$/.test(id))) return null;
            if (entry.source==='curseforge' && ![entry.projectId,entry.versionId].every(id=>/^[1-9][0-9]*$/.test(id))) return null;
        }
    }
    return value as PackCollection[];
}
export function profileImage(value: unknown): value is string {
    if (value === '') return true;
    if (typeof value !== 'string' || value.length > 934000) return false;
    const match = /^data:image\/(png|jpeg|webp|gif);base64,([A-Za-z0-9+/]+={0,2})$/.exec(value);
    if (!match) return false;
    try {
        const bytes = atob(match[2]);
        if (bytes.length > 700000 || bytes.length < 12) return false;
        return match[1] === 'png' ? bytes.startsWith('\x89PNG\r\n\x1a\n') : match[1] === 'jpeg' ? bytes.startsWith('\xff\xd8\xff') : match[1] === 'gif' ? /^GIF8[79]a/.test(bytes) : bytes.startsWith('RIFF') && bytes.slice(8,12) === 'WEBP';
    } catch { return false; }
}
export function validatePublicProfile(value: unknown, maximum = 6, favourites = 8): PlayerProfile | null {
    if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
    const body = value as Record<string,unknown>;
    const profileStyle = parseProfileStyle(body.profileStyle);
    if (!profileStyle) return null;
    const shared = collections(body.collections, maximum);
    if (!shared) return null;
    if (body.displayName !== undefined && (typeof body.displayName !== 'string' || body.displayName.length > 32)) return null;
    if (body.status !== undefined && (typeof body.status !== 'string' || body.status.length > 80)) return null;
    if (typeof body.description !== 'string' || body.description.length > 400 || !profileImage(body.banner) || !profileImage(body.portrait)) return null;
    if (!Array.isArray(body.packs) || body.packs.length > favourites || body.packs.some(pack => typeof pack !== 'string' || !pack.trim() || pack.length > 80)) return null;
    const packs = body.packs as string[];
    const artwork = body.packArtwork === undefined ? undefined : packArtwork(body.packArtwork, packs);
    if (artwork === null) return null;
    return {...(artwork !== undefined ? {packArtwork:artwork} : {}),profileStyle,collections:shared,displayName:typeof body.displayName === 'string' ? body.displayName.trim():'',status:typeof body.status === 'string' ? body.status.trim():'',description:body.description.trim(),banner:body.banner,portrait:body.portrait,packs:[...new Set(packs.map(pack => pack.trim()))]};
}
