<script lang="ts">
    import { profileAccents, profileEffects, profileFrames, profileLayouts, profileBenefits, defaultProfileStyle, type ProfilePlan, type ProfileStyle } from '$lib/utils/profileStyle';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import ProfileSignature from '$lib/components/ui/ProfileSignature.svelte';
    import { Check, LockKeyhole, Palette, RotateCcw, UserRound } from 'lucide-svelte';
    let { value = $bindable(), plan, disabled = false, previewName = 'Luxmc', previewImage = '' }: { value: ProfileStyle; plan: ProfilePlan; disabled?: boolean; previewName?: string; previewImage?: string } = $props();
    const groups = [ { key: 'accent', values: profileAccents, benefit: 'accents' }, { key: 'frame', values: profileFrames, benefit: 'frames' }, { key: 'effect', values: profileEffects, benefit: 'effects' }, { key: 'layout', values: profileLayouts, benefit: 'layouts' } ] as const;
    function allowed(group: typeof groups[number], option: string) { return (profileBenefits[plan][group.benefit] as readonly string[]).includes(option); }
</script>

<section class="space-y-5 rounded-2xl border border-brand-400/25 bg-brand-500/5 p-4 sm:p-5">
    <div class="flex flex-wrap items-start justify-between gap-3"><div><h4 class="flex items-center gap-2 text-sm font-semibold"><Palette class="h-4 w-4 text-brand-400" />{t('planProfile.title')}</h4><p class="mt-2 text-xs leading-5 text-fg-muted">{t('planProfile.help')}</p></div><span class="rounded-full border border-brand-400/25 px-2.5 py-1 text-[10px] font-semibold">Luxmc {plan === 'free' ? 'Free' : plan === 'plus' ? 'Plus' : 'Pro'}</span></div>
    <ProfileSignature signature={value} class="signature-preview">
        <div data-profile-heading class="flex items-center gap-4 rounded-2xl border border-border bg-bg-elevated p-5">
            <div data-profile-avatar class="relative flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-bg-subtle">{#if previewImage}<img src={previewImage} alt="" class="h-full w-full rounded-[inherit] object-cover" />{:else}<UserRound class="h-7 w-7 text-fg-muted" />{/if}</div>
            <div class="min-w-0"><strong data-profile-name class="block truncate text-base">{previewName || 'Luxmc'}</strong><span data-profile-status class="mt-1 block text-xs text-fg-muted">{t('planProfile.preview')}</span></div>
        </div>
    </ProfileSignature>
    {#each groups as group}
        <fieldset><legend class="mb-2.5 text-xs font-semibold">{t(`planProfile.${group.key}`)}</legend><div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
            {#each group.values as option}
                {@const enabled = allowed(group, option)}
                <button type="button" aria-label={`${t(`planProfile.${group.key}`)}: ${t(`planProfile.${group.key}_${option}`)}`} aria-pressed={value[group.key] === option} disabled={disabled || !enabled} title={enabled ? t('planProfile.available') : t(option === 'orbit' || group.key === 'effect' || group.key === 'layout' ? 'planProfile.requiresPro' : 'planProfile.requiresPlus')} class="signature-option flex min-h-24 flex-col gap-3 rounded-xl border px-3 py-4 text-left text-xs transition-colors disabled:opacity-55 {value[group.key] === option ? 'border-brand-400/60 bg-brand-500/10' : 'border-border bg-bg-elevated hover:border-brand-400/40'}" onclick={() => { if (enabled && !disabled) value = {...value, [group.key]:option}; }}>
                    <ProfileSignature signature={{...defaultProfileStyle,[group.key]:option}} class="w-full"><div data-profile-heading class="flex items-center gap-2"><span data-profile-avatar class="relative flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-bg-subtle"><UserRound class="h-4 w-4" /></span><span data-profile-name class="text-[10px] font-semibold">Luxmc</span></div></ProfileSignature>
                    <span class="flex w-full items-center gap-2"><span class="min-w-0 flex-1">{t(`planProfile.${group.key}_${option}`)}</span>{#if !enabled}<LockKeyhole class="h-3.5 w-3.5 shrink-0" />{:else if value[group.key] === option}<Check class="h-3.5 w-3.5 shrink-0 text-brand-400" />{/if}</span>
                </button>
            {/each}
        </div></fieldset>
    {/each}
    <div class="flex flex-wrap items-center justify-between gap-3"><p class="max-w-md text-[11px] leading-5 text-fg-muted">{t('planProfile.expiry')}</p><button type="button" disabled={disabled} class={button({variant:'ghost',size:'sm'})} onclick={() => value = {...defaultProfileStyle}}><RotateCcw class="h-3.5 w-3.5" />{t('planProfile.reset')}</button></div>
</section>
