<script lang="ts">
    import { profileAccents, profileEffects, profileFrames, profileLayouts, profileBenefits, defaultProfileStyle, type ProfilePlan, type ProfileStyle } from '$lib/utils/profileStyle';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import ProfileSignature from '$lib/components/ui/ProfileSignature.svelte';
    import { Check, LockKeyhole, Palette, RotateCcw } from 'lucide-svelte';
    let { value = $bindable(), plan, disabled = false }: { value: ProfileStyle; plan: ProfilePlan; disabled?: boolean } = $props();
    const groups = [ { key: 'accent', values: profileAccents, benefit: 'accents' }, { key: 'frame', values: profileFrames, benefit: 'frames' }, { key: 'effect', values: profileEffects, benefit: 'effects' }, { key: 'layout', values: profileLayouts, benefit: 'layouts' } ] as const;
    function allowed(group: typeof groups[number], option: string) { return (profileBenefits[plan][group.benefit] as readonly string[]).includes(option); }
</script>

<section class="space-y-5 rounded-2xl border border-brand-400/25 bg-brand-500/5 p-4 sm:p-5">
    <div class="flex flex-wrap items-start justify-between gap-3"><div><h4 class="flex items-center gap-2 text-sm font-semibold"><Palette class="h-4 w-4 text-brand-400" />{t('planProfile.title')}</h4><p class="mt-2 text-xs leading-5 text-fg-muted">{t('planProfile.help')}</p></div><span class="rounded-full border border-brand-400/25 px-2.5 py-1 text-[10px] font-semibold">Luxmc {plan === 'free' ? 'Free' : plan === 'plus' ? 'Plus' : 'Pro'}</span></div>
    {#each groups as group}
        <fieldset><legend class="mb-2.5 text-xs font-semibold">{t(`planProfile.${group.key}`)}</legend><div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
            {#each group.values as option}
                {@const enabled = allowed(group, option)}
                <button type="button" aria-label={`${t(`planProfile.${group.key}`)}: ${t(`planProfile.${group.key}_${option}`)}`} aria-pressed={value[group.key] === option} disabled={disabled || !enabled} title={enabled ? t('planProfile.available') : t(option === 'orbit' || group.key === 'effect' || group.key === 'layout' ? 'planProfile.requiresPro' : 'planProfile.requiresPlus')} class="flex min-h-12 items-center gap-2 rounded-xl border px-3 py-2 text-left text-xs transition-colors disabled:opacity-55 {value[group.key] === option ? 'border-brand-400/60 bg-brand-500/10' : 'border-border bg-bg-elevated hover:border-brand-400/40'}" onclick={() => { if (enabled && !disabled) value = {...value, [group.key]:option}; }}>
                    {#if group.key === 'accent'}<ProfileSignature signature={{...defaultProfileStyle, accent:option as ProfileStyle['accent']}}><span data-profile-name class="inline-block h-4 w-4 rounded-full bg-current" aria-hidden="true"></span></ProfileSignature>{/if}
                    <span class="min-w-0 flex-1">{t(`planProfile.${group.key}_${option}`)}</span>{#if !enabled}<LockKeyhole class="h-3.5 w-3.5 shrink-0" />{:else if value[group.key] === option}<Check class="h-3.5 w-3.5 shrink-0 text-brand-400" />{/if}
                </button>
            {/each}
        </div></fieldset>
    {/each}
    <div class="flex flex-wrap items-center justify-between gap-3"><p class="max-w-md text-[11px] leading-5 text-fg-muted">{t('planProfile.expiry')}</p><button type="button" disabled={disabled} class={button({variant:'ghost',size:'sm'})} onclick={() => value = {...defaultProfileStyle}}><RotateCcw class="h-3.5 w-3.5" />{t('planProfile.reset')}</button></div>
</section>
