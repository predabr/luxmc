<script lang="ts">
    import { profileAccents, profileCardBackgrounds, profileEffects, profileFrameDetails, profileFrames, profileLayouts, profileBenefits, defaultProfileStyle, type ProfilePlan, type ProfileStyle } from '$lib/utils/profileStyle';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import ProfileSignature from '$lib/components/ui/ProfileSignature.svelte';
    import { planPrompt } from '$lib/stores/planPrompt.svelte';
    import { Check, LockKeyhole, RotateCcw, Sparkles } from 'lucide-svelte';
    let { value = $bindable(), plan, disabled = false, previewName = 'Luxmc', previewImage = '', previewBanner = '' }: { value: ProfileStyle; plan: ProfilePlan; disabled?: boolean; previewName?: string; previewImage?: string; previewBanner?: string } = $props();
    const groups = [ { key: 'accent', values: profileAccents, benefit: 'accents' }, { key: 'frame', values: profileFrames, benefit: 'frames' }, { key: 'effect', values: profileEffects, benefit: 'effects' }, { key: 'layout', values: profileLayouts, benefit: 'layouts' }, { key: 'frameDetail', values: profileFrameDetails, benefit: 'frameDetails' }, { key: 'cardBackground', values: profileCardBackgrounds, benefit: 'cardBackgrounds' } ] as const;
    const initials = $derived(Array.from(previewName || 'Luxmc').slice(0,2).join('').toUpperCase());
    function allowed(group: typeof groups[number], option: string) { return (profileBenefits[plan][group.benefit] as readonly string[]).includes(option); }
    function fallback(event: Event) { const image = event.currentTarget as HTMLImageElement; if (!image.src.endsWith('/grass_head.png')) image.src = '/grass_head.png'; }
</script>

{#snippet avatar(size: string)}
    <span data-profile-avatar class="flex shrink-0 items-center justify-center rounded-xl bg-bg-elevated font-semibold text-fg {size}">
        {#if previewImage}<img src={previewImage} alt="" decoding="async" onerror={fallback} class="h-full w-full rounded-[inherit] object-cover" />{:else}{initials}{/if}
    </span>
{/snippet}

<section class="signature-studio space-y-6 rounded-3xl border border-border bg-bg-elevated/65 p-4 sm:p-5">
    <header class="flex flex-wrap items-start justify-between gap-3"><div><p class="mb-2 flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[.2em] text-brand-400"><Sparkles class="h-3.5 w-3.5" />{t('planProfile.previewTitle')}</p><h4 class="text-base font-semibold">{t('planProfile.title')}</h4><p class="mt-2 max-w-lg text-xs leading-5 text-fg-muted">{t('planProfile.help')}</p></div><span class="rounded-full border border-brand-400/25 bg-brand-500/10 px-3 py-1.5 text-[10px] font-semibold text-brand-300">Luxmc {plan === 'free' ? 'Free' : plan === 'plus' ? 'Plus' : 'Pro'}</span></header>
    <ProfileSignature signature={value} class="signature-preview overflow-hidden rounded-2xl border border-border bg-bg-elevated">
        <div data-profile-banner class="signature-cover h-24 overflow-hidden">{#if previewBanner}<img src={previewBanner} alt="" class="h-full w-full object-cover" />{:else}<div class="signature-cover-lines" aria-hidden="true"></div>{/if}<span class="absolute right-4 top-3 rounded-full border border-fg/10 bg-bg-overlay/70 px-2.5 py-1 text-[9px] font-medium text-fg">{t('planProfile.preview')}</span></div>
        <div data-profile-heading class="relative isolate flex gap-4 overflow-hidden px-5 pb-5 pt-4">{#if value.cardBackground === 'cover' && previewBanner}<img src={previewBanner} alt="" class="pointer-events-none absolute inset-0 -z-10 h-full w-full object-cover opacity-20" />{/if}
            {@render avatar('h-16 w-16 text-lg')}
            <div class="min-w-0"><strong data-profile-name class="block truncate text-base">{previewName || 'Luxmc'}</strong><p class="mt-1 text-[11px] leading-5 text-fg-muted">{t('planProfile.previewCaption')}</p><div class="mt-2 flex flex-wrap gap-1.5"><span class="rounded-full border border-border px-2 py-0.5 text-[9px] text-fg-muted">{t(`planProfile.accent_${value.accent}`)}</span><span class="rounded-full border border-border px-2 py-0.5 text-[9px] text-fg-muted">{t(`planProfile.frame_${value.frame}`)}</span></div></div>
        </div>
    </ProfileSignature>
    {#each groups as group}
        <fieldset><legend class="mb-1 text-xs font-semibold">{t(`planProfile.${group.key}`)}</legend><p class="mb-3 text-[11px] leading-5 text-fg-muted">{t(`planProfile.${group.key}Hint`)}</p><div class="grid gap-2 {group.key === 'accent' ? 'grid-cols-2 lg:grid-cols-3' : 'grid-cols-2 lg:grid-cols-3'}">
            {#each group.values as option}
                {@const enabled = allowed(group, option)}
                {@const proOnly = !(profileBenefits.plus[group.benefit] as readonly string[]).includes(option)}
                {@const selected = (value[group.key] ?? (['cardBackground','frameDetail'].includes(group.key) ? 'none' : undefined)) === option}
                <button type="button" aria-label={`${t(`planProfile.${group.key}`)}: ${t(`planProfile.${group.key}_${option}`)}`} aria-pressed={selected} disabled={disabled} title={enabled ? t('planProfile.available') : t(proOnly ? 'planProfile.requiresPro' : 'planProfile.requiresPlus')} class="signature-option relative overflow-hidden rounded-2xl border p-3 text-left text-xs transition-[border-color,background-color,transform] duration-200 disabled:opacity-55 {selected ? 'border-brand-400/60 bg-brand-500/10' : 'border-border bg-bg-subtle/45 hover:border-brand-400/40'}" onclick={() => { if (disabled) return; if (enabled) value = {...value, [group.key]:option}; else planPrompt.open(t(proOnly ? 'planProfile.requiresPro' : 'planPrompt.style'),proOnly ? 'pro' : 'plus'); }}>
                    <ProfileSignature signature={{...defaultProfileStyle,accent:value.accent,[group.key]:option}} class="signature-mini block">
                        <div class="signature-sample mb-3 overflow-hidden rounded-xl border border-border bg-bg-elevated">
                            <div data-profile-banner class="h-7 overflow-hidden">{#if previewBanner}<img src={previewBanner} alt="" loading="lazy" class="h-full w-full object-cover" />{:else}<div class="signature-cover-lines" aria-hidden="true"></div>{/if}</div>
                            <div data-profile-heading class="relative isolate flex min-h-24 gap-3 overflow-hidden px-3 py-4">{#if group.key === 'cardBackground' && option === 'cover'}<div class="pointer-events-none absolute inset-0 -z-10 bg-gradient-to-br from-brand-500/25 via-transparent to-info/20">{#if previewBanner}<img src={previewBanner} alt="" loading="lazy" class="h-full w-full object-cover opacity-25" />{/if}</div>{/if}
                                {@render avatar(group.key === 'frame' ? 'h-10 w-10 text-xs' : 'h-9 w-9 text-xs')}
                                <div class="min-w-0 flex-1"><span data-profile-name class="block truncate text-[10px] font-semibold">{previewName || 'Luxmc'}</span><span data-profile-status class="mt-2 inline-flex max-w-full items-center gap-1 rounded-full border px-1.5 py-1 text-[8px]"><span class="h-1 w-1 shrink-0 rounded-full bg-success"></span><span class="truncate">{t('planPrompt.previewStatus')}</span></span></div>
                            </div>
                            {#if group.key === 'accent'}<span class="signature-swatch block h-1.5" aria-hidden="true"></span>{/if}
                        </div>
                    </ProfileSignature>
                    <span class="flex w-full items-center gap-1.5"><span class="min-w-0 flex-1 text-[11px] font-medium">{t(`planProfile.${group.key}_${option}`)}</span>{#if !enabled}<LockKeyhole class="h-3 w-3 shrink-0 text-fg-subtle" />{:else if selected}<Check class="h-3 w-3 shrink-0 text-brand-400" />{/if}</span>
                </button>
            {/each}
        </div></fieldset>
    {/each}
    <footer class="flex flex-wrap items-center justify-between gap-3 border-t border-border pt-4"><p class="max-w-md text-[11px] leading-5 text-fg-muted">{t('planProfile.expiry')}</p><button type="button" disabled={disabled} class={button({variant:'ghost',size:'sm'})} onclick={() => value = {...defaultProfileStyle,frameDetail:'none',cardBackground:'none'}}><RotateCcw class="h-3.5 w-3.5" />{t('planProfile.reset')}</button></footer>
</section>

<style>
    .signature-cover-lines { position: absolute; pointer-events: none; inset: -50%; background: repeating-linear-gradient(135deg, transparent 0 24px, color-mix(in srgb, var(--profile-accent), transparent 84%) 24px 25px); transform: rotate(-12deg); }
    .signature-swatch { background: linear-gradient(135deg, var(--profile-accent), color-mix(in oklab, var(--profile-accent), rgb(var(--bg-elevated)) 60%)); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--profile-accent), rgb(var(--fg)) 20%); }
    .signature-option:not(:disabled):hover { transform: translateY(-2px); }
    .signature-option:not(:disabled):active { transform: translateY(0) scale(.98); }
    :global(.signature-mini [data-profile-avatar])::after, :global(.signature-mini [data-profile-avatar])::before, :global(.signature-mini [data-profile-banner])::after { animation: none; }
    :global(.signature-mini[data-profile-effect="shimmer"] [data-profile-banner])::after { transform: translateX(0); opacity: .7; }
    :global(.signature-mini[data-profile-effect="pulse"] [data-profile-avatar])::before { opacity: .65; transform: scale(1.04); }
    .signature-option:hover :global(.signature-mini [data-profile-avatar])::after, .signature-option:focus-visible :global(.signature-mini [data-profile-avatar])::after { animation: var(--profile-frame-animation); }
    .signature-option:hover :global(.signature-mini[data-profile-effect="pulse"] [data-profile-avatar])::before, .signature-option:focus-visible :global(.signature-mini[data-profile-effect="pulse"] [data-profile-avatar])::before { animation: lux-profile-pulse 4s ease-in-out infinite; }
    .signature-option:hover :global(.signature-mini[data-profile-effect="shimmer"] [data-profile-banner])::after, .signature-option:focus-visible :global(.signature-mini[data-profile-effect="shimmer"] [data-profile-banner])::after { animation: lux-profile-shimmer 3s ease-in-out infinite; }
    :global(html.no-anim) .signature-option, :global(html.no-ui-motion) .signature-option { transition: none; transform: none !important; }
    @media (prefers-reduced-motion: reduce) { .signature-option { transition: none; transform: none !important; } }
</style>
