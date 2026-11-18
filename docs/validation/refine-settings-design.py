from pathlib import Path
import json
p=Path('src/routes/settings/+page.svelte');s=p.read_text(encoding='utf-8')
s=s.replace('let gameResolution = $state(settings.value.startFullscreen ? "fullscreen" : `${settings.value.defaultResWidth ?? 1920}x${settings.value.defaultResHeight ?? 1080}`);','''function resolveDefaultResolution(value: AppSettings) {
        if (value.startFullscreen) return "fullscreen";
        const dimensions = `${value.defaultResWidth ?? 1920}x${value.defaultResHeight ?? 1080}`;
        return dimensions === "854x480" ? "default" : ["1920x1080", "1280x720"].includes(dimensions) ? dimensions : "custom";
    }
    let gameResolution = $state(resolveDefaultResolution(settings.value));''')
s=s.replace('startFullscreen: settings.value.startFullscreen,','startFullscreen: settings.value.startFullscreen,\n        defaultResWidth: settings.value.defaultResWidth,\n        defaultResHeight: settings.value.defaultResHeight,',1)
s=s.replace('if (current.startFullscreen !== syncedSettings.startFullscreen) {\n\t\t\tsyncedSettings.startFullscreen = current.startFullscreen;\n\t\t\tgameResolution = current.startFullscreen ? "fullscreen" : "1920x1080";\n\t\t}', '''if (current.startFullscreen !== syncedSettings.startFullscreen || current.defaultResWidth !== syncedSettings.defaultResWidth || current.defaultResHeight !== syncedSettings.defaultResHeight) {
            syncedSettings.startFullscreen = current.startFullscreen;
            syncedSettings.defaultResWidth = current.defaultResWidth;
            syncedSettings.defaultResHeight = current.defaultResHeight;
            gameResolution = resolveDefaultResolution(current);
            customResWidth = current.defaultResWidth ?? 1920;
            customResHeight = current.defaultResHeight ?? 1080;
        }''')
p.write_text(s,encoding='utf-8')
p=Path('src/routes/instances/[id]/+page.svelte');s=p.read_text(encoding='utf-8')
s=s.replace('                            <div><h3 class="text-base font-semibold text-fg">{uiText("ui.fa18e9c316c39701")}</h3><p class="mt-1 text-xs text-fg-muted">{uiText("ui.5cccec99fd3f2ff4")}</p></div>\n','',1)
pos=s.index('                                <button type="button" class={button({ variant: "secondary", class: "h-auto justify-start whitespace-normal p-4 text-left" })} onclick={openInstanceFolder}')
s=s[:pos]+'''                                <button type="button" class={button({ variant: "secondary", class: "h-auto justify-start whitespace-normal p-4 text-left" })} onclick={() => showDeathDetectorModal = true}><Skull class="h-5 w-5 shrink-0 text-brand-400" /><span class="text-sm font-semibold">{uiText("ui.d88bf4aee9b1d26f")}</span></button>
'''+s[pos:]
p.write_text(s,encoding='utf-8')
p=Path('src/lib/components/settings/ThemeSection.svelte');s=p.read_text(encoding='utf-8')
s=s.replace('\tlet mysticAuraGlow = $state(true);\n','')
s=s.replace('\t\t\t{ title: t("settings.appearanceOptions.mysticAura"), desc: t("settings.appearanceOptions.mysticAuraDesc"), val: mysticAuraGlow, toggle: () => mysticAuraGlow = !mysticAuraGlow },\n','')
p.write_text(s,encoding='utf-8')
entries={
'instanceTools':['Ferramentas','Tools','Herramientas'],
'settingsTitle':['Seu launcher, do seu jeito','Your launcher, your way','Tu launcher, a tu manera'],
'customResolution':['Personalizada','Custom','Personalizada'],
'resolutionWidth':['Largura da janela (px)','Window width (px)','Ancho de la ventana (px)'],
'resolutionHeight':['Altura da janela (px)','Window height (px)','Altura de la ventana (px)'],
'feedbackTitle':['Atualizações, áudio e diagnóstico','Updates, audio and diagnostics','Actualizaciones, audio y diagnóstico'],
'soundEnabled':['Sons do launcher','Launcher sounds','Sonidos del launcher'],
'soundEnabledDesc':['Feedback ao iniciar, confirmar e receber avisos.','Feedback when launching, confirming and receiving alerts.','Avisos al iniciar, confirmar y recibir notificaciones.'],
'soundVolume':['Volume dos efeitos','Sound effect volume','Volumen de los efectos'],
'previewSound':['Testar som','Preview sound','Probar sonido'],
'advancedDiscord':['Discord avançado','Advanced Discord settings','Discord avanzado'],
'storageCopied':['Resumo copiado sem caminhos pessoais.','Summary copied without personal paths.','Resumen copiado sin rutas personales.']
}
catalog={language:{'settings':{'refinement':{k:v[i] for k,v in entries.items()}}} for i,language in enumerate(['pt-BR','en','es'])}
Path('docs/validation/settings-design-translations.json').write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
