<script lang="ts">
    import { settings, type AppSettings } from '$lib/stores/settings.svelte';
    import { schedulePersist } from '$lib/stores/persistence.svelte';
    import { appState } from '$lib/stores/app.svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { Sparkles, Image, Zap } from 'lucide-svelte';
    function update(value: Partial<AppSettings>) { settings.patch(value); schedulePersist(); }
</script>

<section class="space-y-5 rounded-2xl border border-border bg-bg-subtle p-5" aria-label={t('extendedSettings.interface')}>
    <div class="flex items-center gap-3"><Sparkles class="h-5 w-5 text-brand-400" /><div><h3 class="font-bold text-fg">{t('extendedSettings.interface')}</h3><p class="mt-1 text-xs text-fg-muted">{t('extendedSettings.interfaceHelp')}</p></div></div>
    <div class="grid gap-3 sm:grid-cols-2">
        <label class="flex items-center justify-between gap-3 rounded-xl border border-border p-4 text-sm"><span>{t('settings.appearanceOptions.animations')}</span><input type="checkbox" class="h-5 w-5 accent-brand-500" checked={settings.value.animations !== false} onchange={event => update({animations:event.currentTarget.checked})} /></label>
        <label class="flex items-center justify-between gap-3 rounded-xl border border-border p-4 text-sm"><span>{t('extendedSettings.reducedMotion')}</span><input type="checkbox" class="h-5 w-5 accent-brand-500" checked={settings.value.respectReducedMotion !== false} onchange={event => update({respectReducedMotion:event.currentTarget.checked})} /></label>
        <label class="flex items-center justify-between gap-3 rounded-xl border border-border p-4 text-sm"><span>{t('settings.perfModeTitle')}</span><input type="checkbox" class="h-5 w-5 accent-brand-500" checked={settings.value.performanceMode === true} onchange={event => {appState.performanceMode=event.currentTarget.checked;update({performanceMode:event.currentTarget.checked});}} /></label>
        <p class="self-center text-xs leading-5 text-fg-muted">{t('extendedSettings.motionHelp')}</p>
    </div>
    <div class="grid gap-4 sm:grid-cols-2">
        <label class="space-y-2 text-sm text-fg"><span>{t('extendedSettings.motionStyle')}</span><select class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.motionStyle ?? 'expressive'} onchange={event => update({ motionStyle: event.currentTarget.value as AppSettings['motionStyle'] })}><option value="expressive">{t('extendedSettings.expressive')}</option><option value="subtle">{t('extendedSettings.subtle')}</option></select></label>
        <label class="space-y-2 text-sm text-fg"><span>{t('extendedSettings.motionSpeed')}</span><select class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.motionSpeed ?? 'balanced'} onchange={event => update({ motionSpeed: event.currentTarget.value as AppSettings['motionSpeed'] })}><option value="balanced">{t('extendedSettings.balanced')}</option><option value="fast">{t('extendedSettings.fast')}</option></select></label>
        <label class="space-y-2 text-sm text-fg"><span class="flex items-center gap-2"><Image class="h-4 w-4" />{t('extendedSettings.gifQuality')}</span><select class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.gifQuality ?? 'high'} onchange={event => update({ gifQuality: event.currentTarget.value as AppSettings['gifQuality'] })}><option value="high">{t('extendedSettings.gifHigh')}</option><option value="balanced">{t('extendedSettings.gifBalanced')}</option></select><small class="block text-fg-muted">{t('extendedSettings.gifHelp')}</small></label>
        <label class="flex items-center justify-between gap-4 rounded-xl border border-border p-4 text-sm text-fg"><span><span class="flex items-center gap-2"><Zap class="h-4 w-4" />{t('extendedSettings.preload')}</span><small class="mt-2 block leading-relaxed text-fg-muted">{t('extendedSettings.preloadHelp')}</small></span><input type="checkbox" class="h-5 w-5 shrink-0 accent-brand-500" checked={settings.value.preloadContentIcons !== false} onchange={event => update({ preloadContentIcons: event.currentTarget.checked })} /></label>
    </div>
</section>
