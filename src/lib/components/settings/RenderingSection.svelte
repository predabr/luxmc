<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { settings, type AppSettings } from "$lib/stores/settings.svelte";
    import { schedulePersist } from "$lib/stores/persistence.svelte";
    import { useTranslation } from "$lib/i18n/useTranslation.svelte";
    const { t } = useTranslation();
    function update(value: Partial<AppSettings>) { settings.patch(value); schedulePersist(); }
</script>

<section class="rounded-2xl border border-border bg-bg-subtle p-5 space-y-5" aria-labelledby="rendering-title">
    <div>
        <h3 id="rendering-title" class="font-bold text-fg">{t("settings.renderingOptions.title")}</h3>
        <p class="mt-1 text-xs leading-relaxed text-fg-muted">{t("settings.renderingOptions.desc")}</p>
    </div>
    <div class="grid gap-4 sm:grid-cols-2">
        <label class="space-y-2 text-sm text-fg-muted">
            <span>{t("settings.renderingOptions.resolution")}</span>
            <select class="w-full rounded-xl border border-border bg-bg-elevated p-3 text-fg" value={String(settings.value.wallpaperWidth ?? 1280)} onchange={e => update({ wallpaperWidth: Number(e.currentTarget.value) as 960 | 1280 | 1920 })}>
                <option value="960">{uiText("ui.4a5fcbf1566396b6")}</option><option value="1280">{uiText("ui.3618a68de4effc07")}</option><option value="1920">{uiText("ui.1b13606c49b549e7")}</option>
            </select>
        </label>
        <label class="space-y-2 text-sm text-fg-muted">
            <span>{t("settings.renderingOptions.fps")}</span>
            <select class="w-full rounded-xl border border-border bg-bg-elevated p-3 text-fg" value={String(settings.value.wallpaperFps ?? 60)} onchange={e => update({ wallpaperFps: Number(e.currentTarget.value) as 15 | 24 | 30 | 60 })}>
                <option value="15">{uiText("ui.97ca8cc7edf289d5")}</option><option value="24">{uiText("ui.ad73871f299ad4a9")}</option><option value="30">{uiText("ui.39f81d21015af161")}</option><option value="60">{uiText("ui.8de37d25b5895bd4")}</option>
            </select>
        </label>
    </div>
    <label class="flex items-center justify-between gap-4 text-sm text-fg">
        <span>{t("settings.renderingOptions.pauseOnBlur")}</span>
        <input type="checkbox" checked={settings.value.pauseWallpaperOnBlur === true} onchange={e => update({ pauseWallpaperOnBlur: e.currentTarget.checked })} class="h-5 w-5 rounded border-border accent-brand-500" />
    </label>
    <label class="flex items-center justify-between gap-4 text-sm text-fg">
        <span>{t("settings.renderingOptions.animatedBlur")} <small class="block mt-1 text-fg-muted">{t("settings.renderingOptions.animatedBlurDesc")}</small></span>
        <input type="checkbox" checked={settings.value.animatedWallpaperBlur === true} onchange={e => update({ animatedWallpaperBlur: e.currentTarget.checked })} class="h-5 w-5 rounded border-border accent-brand-500" />
    </label>
    <p class="text-xs leading-relaxed text-fg-muted">{t("settings.renderingOptions.footer")}</p>
</section>
