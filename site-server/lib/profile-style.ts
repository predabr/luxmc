export type ProfilePlan = 'free' | 'plus' | 'pro';
export const profileAccents = ['brand', 'ocean', 'mint', 'sunset', 'rose', 'lilac'] as const;
export const profileFrames = ['none', 'halo', 'double', 'orbit'] as const;
export const profileEffects = ['none', 'shimmer', 'pulse'] as const;
export const profileAmbientEffects = ['none', 'aurora', 'beacon'] as const;
export const profileLayouts = ['classic', 'centered'] as const;
export const profileFrameDetails = ['none', 'prism'] as const;
export const profileShowcaseLayouts = ['grid', 'spotlight', 'poster'] as const;
export const profileCardBackgrounds = ['none', 'cover'] as const;
export interface ProfileStyle {
    accent: typeof profileAccents[number];
    frame: typeof profileFrames[number];
    effect: typeof profileEffects[number];
    ambientEffect?: typeof profileAmbientEffects[number];
    layout: typeof profileLayouts[number];
    frameDetail?: typeof profileFrameDetails[number];
    showcaseLayout?: typeof profileShowcaseLayouts[number];
    cardBackground?: typeof profileCardBackgrounds[number];
}
export const defaultProfileStyle: Readonly<ProfileStyle> = { accent: 'brand', frame: 'none', effect: 'none', layout: 'classic' };
export const profileBenefits = {
    free: { favourites: 8, accents: ['brand'], frames: ['none'], effects: ['none'], ambientEffects: ['none'], layouts: ['classic'], frameDetails: ['none'], showcaseLayouts: ['grid'], cardBackgrounds: ['none'] },
    plus: { favourites: 16, accents: [...profileAccents], frames: ['none', 'halo', 'double'], effects: ['none'], ambientEffects: ['none'], layouts: ['classic'], frameDetails: ['none'], showcaseLayouts: ['grid'], cardBackgrounds: ['none'] },
    pro: { favourites: 24, accents: [...profileAccents], frames: [...profileFrames], effects: [...profileEffects], ambientEffects: [...profileAmbientEffects], layouts: [...profileLayouts], frameDetails: [...profileFrameDetails], showcaseLayouts: [...profileShowcaseLayouts], cardBackgrounds: [...profileCardBackgrounds] }
} satisfies Record<ProfilePlan, { favourites: number; accents: readonly string[]; frames: readonly string[]; effects: readonly string[]; ambientEffects: readonly string[]; layouts: readonly string[]; frameDetails: readonly string[]; showcaseLayouts: readonly string[]; cardBackgrounds: readonly string[] }>;
export function parseProfileStyle(value: unknown): ProfileStyle | null {
    if (value === undefined) return { ...defaultProfileStyle };
    if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
    const record = value as Record<string, unknown>;
    if (Object.keys(record).some(key => !['accent', 'frame', 'effect', 'layout', 'frameDetail', 'cardBackground', 'showcaseLayout', 'ambientEffect'].includes(key))) return null;
    if (record.ambientEffect !== undefined && !profileAmbientEffects.includes(record.ambientEffect as 'none')) return null;
    if (record.showcaseLayout !== undefined && !profileShowcaseLayouts.includes(record.showcaseLayout as 'grid')) return null;
    if (record.cardBackground !== undefined && !profileCardBackgrounds.includes(record.cardBackground as 'none' | 'cover')) return null;
    if (record.frameDetail !== undefined && !profileFrameDetails.includes(record.frameDetail as 'none' | 'prism')) return null;
    const style = { ...defaultProfileStyle, ...record };
    if (!profileAccents.includes(style.accent as ProfileStyle['accent']) || !profileFrames.includes(style.frame as ProfileStyle['frame']) || !profileEffects.includes(style.effect as ProfileStyle['effect']) || !profileLayouts.includes(style.layout as ProfileStyle['layout'])) return null;
    return style as ProfileStyle;
}
export function activeProfileStyle(value: unknown, plan: ProfilePlan): ProfileStyle {
    const style = parseProfileStyle(value) || { ...defaultProfileStyle };
    const benefits = profileBenefits[plan];
    return {
        accent: (benefits.accents as readonly string[]).includes(style.accent) ? style.accent : 'brand',
        frame: (benefits.frames as readonly string[]).includes(style.frame) ? style.frame : 'none',
        ...(style.ambientEffect !== undefined ? { ambientEffect: (benefits.ambientEffects as readonly string[]).includes(style.ambientEffect) ? style.ambientEffect : 'none' as const } : {}),
        effect: (benefits.effects as readonly string[]).includes(style.effect) ? style.effect : 'none',
        layout: (benefits.layouts as readonly string[]).includes(style.layout) ? style.layout : 'classic',
        ...(style.showcaseLayout !== undefined ? { showcaseLayout: (benefits.showcaseLayouts as readonly string[]).includes(style.showcaseLayout) ? style.showcaseLayout : 'grid' as const } : {}),
        ...(style.frameDetail !== undefined ? { frameDetail: (benefits.frameDetails as readonly string[]).includes(style.frameDetail) ? style.frameDetail : 'none' as const } : {}),
        ...(style.cardBackground !== undefined ? { cardBackground: (benefits.cardBackgrounds as readonly string[]).includes(style.cardBackground) ? style.cardBackground : 'none' as const } : {})
    };
}
export function profileStyleAllowed(value: ProfileStyle, plan: ProfilePlan): boolean {
    const active = activeProfileStyle(value, plan);
    return active.ambientEffect === value.ambientEffect && active.accent === value.accent && active.frame === value.frame && active.effect === value.effect && active.layout === value.layout && active.cardBackground === value.cardBackground && active.frameDetail === value.frameDetail && active.showcaseLayout === value.showcaseLayout;
}
export function retainedProfileStyle(value: ProfileStyle, stored: unknown, plan: ProfilePlan): ProfileStyle {
    const saved = parseProfileStyle(stored) || { ...defaultProfileStyle };
    const active = activeProfileStyle(saved, plan);
    const allowed = profileBenefits[plan];
    return {
        accent: !(allowed.accents as readonly string[]).includes(saved.accent) && value.accent === active.accent ? saved.accent : value.accent,
        frame: !(allowed.frames as readonly string[]).includes(saved.frame) && value.frame === active.frame ? saved.frame : value.frame,
        ...(value.ambientEffect !== undefined || saved.ambientEffect !== undefined ? { ambientEffect: value.ambientEffect === undefined || saved.ambientEffect !== 'none' && plan !== 'pro' && value.ambientEffect === 'none' ? saved.ambientEffect : value.ambientEffect } : {}),
        effect: !(allowed.effects as readonly string[]).includes(saved.effect) && value.effect === active.effect ? saved.effect : value.effect,
        layout: !(allowed.layouts as readonly string[]).includes(saved.layout) && value.layout === active.layout ? saved.layout : value.layout,
        ...(value.showcaseLayout !== undefined || saved.showcaseLayout !== undefined ? { showcaseLayout: value.showcaseLayout === undefined || saved.showcaseLayout !== 'grid' && plan !== 'pro' && value.showcaseLayout === 'grid' ? saved.showcaseLayout : value.showcaseLayout } : {}),
        ...(value.frameDetail !== undefined || saved.frameDetail !== undefined ? { frameDetail: value.frameDetail === undefined || saved.frameDetail === 'prism' && plan !== 'pro' && value.frameDetail === 'none' ? saved.frameDetail : value.frameDetail } : {}),
        ...(value.cardBackground !== undefined || saved.cardBackground !== undefined ? { cardBackground: value.cardBackground === undefined || saved.cardBackground === 'cover' && plan !== 'pro' && value.cardBackground === 'none' ? saved.cardBackground : value.cardBackground } : {})
    };
}

export function isProfileGif(value: unknown): value is string {
    if (typeof value !== 'string') return false;
    if (value.startsWith('data:image/gif;base64,')) return true;
    const prefix = 'data:image/webp;base64,';
    if (!value.startsWith(prefix)) return false;
    try {
        const header = atob(value.slice(prefix.length, prefix.length + 40));
        return header.startsWith('RIFF') && header.slice(8,12) === 'WEBP' && header.slice(12,16) === 'VP8X' && (header.charCodeAt(20) & 2) !== 0;
    } catch { return false; }
}
export function activeProfileImage(value: string, plan: ProfilePlan): string { return plan === 'free' && isProfileGif(value) ? '' : value; }
export function retainedProfileImage(value: string, stored: string, plan: ProfilePlan): string { return plan === 'free' && !value && isProfileGif(stored) ? stored : value; }
