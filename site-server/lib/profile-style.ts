export type ProfilePlan = 'free' | 'plus' | 'pro';
export const profileAccents = ['brand', 'ocean', 'mint', 'sunset', 'rose', 'lilac'] as const;
export const profileFrames = ['none', 'halo', 'double', 'orbit'] as const;
export const profileEffects = ['none', 'shimmer', 'pulse'] as const;
export const profileLayouts = ['classic', 'centered'] as const;
export interface ProfileStyle {
    accent: typeof profileAccents[number];
    frame: typeof profileFrames[number];
    effect: typeof profileEffects[number];
    layout: typeof profileLayouts[number];
}
export const defaultProfileStyle: Readonly<ProfileStyle> = { accent: 'brand', frame: 'none', effect: 'none', layout: 'classic' };
export const profileBenefits = {
    free: { favourites: 8, accents: ['brand'], frames: ['none'], effects: ['none'], layouts: ['classic'] },
    plus: { favourites: 16, accents: [...profileAccents], frames: ['none', 'halo', 'double'], effects: ['none'], layouts: ['classic'] },
    pro: { favourites: 24, accents: [...profileAccents], frames: [...profileFrames], effects: [...profileEffects], layouts: [...profileLayouts] }
} satisfies Record<ProfilePlan, { favourites: number; accents: readonly string[]; frames: readonly string[]; effects: readonly string[]; layouts: readonly string[] }>;
export function parseProfileStyle(value: unknown): ProfileStyle | null {
    if (value === undefined) return { ...defaultProfileStyle };
    if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
    const record = value as Record<string, unknown>;
    if (Object.keys(record).some(key => !['accent', 'frame', 'effect', 'layout'].includes(key))) return null;
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
        effect: (benefits.effects as readonly string[]).includes(style.effect) ? style.effect : 'none',
        layout: (benefits.layouts as readonly string[]).includes(style.layout) ? style.layout : 'classic'
    };
}
export function profileStyleAllowed(value: ProfileStyle, plan: ProfilePlan): boolean {
    const active = activeProfileStyle(value, plan);
    return active.accent === value.accent && active.frame === value.frame && active.effect === value.effect && active.layout === value.layout;
}
export function retainedProfileStyle(value: ProfileStyle, stored: unknown, plan: ProfilePlan): ProfileStyle {
    const saved = parseProfileStyle(stored) || { ...defaultProfileStyle };
    const active = activeProfileStyle(saved, plan);
    const allowed = profileBenefits[plan];
    return {
        accent: !(allowed.accents as readonly string[]).includes(saved.accent) && value.accent === active.accent ? saved.accent : value.accent,
        frame: !(allowed.frames as readonly string[]).includes(saved.frame) && value.frame === active.frame ? saved.frame : value.frame,
        effect: !(allowed.effects as readonly string[]).includes(saved.effect) && value.effect === active.effect ? saved.effect : value.effect,
        layout: !(allowed.layouts as readonly string[]).includes(saved.layout) && value.layout === active.layout ? saved.layout : value.layout
    };
}

export function isProfileGif(value: unknown): value is string { return typeof value === 'string' && value.startsWith('data:image/gif;base64,'); }
export function activeProfileImage(value: string, plan: ProfilePlan): string { return plan === 'free' && isProfileGif(value) ? '' : value; }
export function retainedProfileImage(value: string, stored: string, plan: ProfilePlan): string { return plan === 'free' && !value && isProfileGif(stored) ? stored : value; }
