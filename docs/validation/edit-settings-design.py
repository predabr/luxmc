from pathlib import Path
p=Path('src/routes/instances/[id]/+page.svelte')
s=p.read_text(encoding='utf-8')
s=s.replace('\t\tLink\n', '\t\tLink,\n        Wrench\n')
s=s.replace('\tlet showKeybindEditorModal = $state(false);', '\tlet showKeybindEditorModal = $state(false);\n    let toolsExpanded = $state(false);')
start=s.index('<section class="instance-settings-group" aria-label={uiText("ui.fa18e9c316c39701")}>')
end=s.index('<section id="instance-settings-geral"', start)
tools=s[start:end]
s=s[:start]+s[end:]
a=s.index('                <button type="button" class={button({ variant: \'ghost\', size: \'sm\' })} onclick={() => showModpackExportModal')
b=s.index('            {/snippet}', a)
s=s[:a]+'''                <button type="button" class={button({ variant: toolsExpanded ? 'secondary' : 'ghost', size: 'sm' })} aria-expanded={toolsExpanded} aria-controls="instance-toolbox" onclick={() => toolsExpanded = !toolsExpanded}><Wrench class="h-4 w-4" />{uiText("settings.refinement.instanceTools")}</button>
'''+s[b:]
new='''        {#if toolsExpanded}
            <section id="instance-toolbox" class="surface-glass space-y-5 p-5 sm:p-6" aria-label={uiText("settings.refinement.instanceTools")} in:fade={{ duration: 150 }}>
                <header class="flex items-start justify-between gap-4">
                    <div class="space-y-1"><p class="page-eyebrow">{uiText("settings.refinement.instanceTools")}</p><h2 class="text-lg font-semibold text-fg">{uiText("ui.fa18e9c316c39701")}</h2><p class="text-sm text-fg-muted">{uiText("ui.5cccec99fd3f2ff4")}</p></div>
                    <button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={uiText("common.close")} onclick={() => toolsExpanded = false}><X class="h-4 w-4" /></button>
                </header>
'''+tools+'''            </section>
        {/if}
'''
s=s.replace('        </InstanceHero>\n', '        </InstanceHero>\n'+new, 1)
s=s.replace('class="flex items-center gap-2 text-xs font-bold text-fg/50 hover:text-fg transition-colors w-fit group"','class={button({ variant: "ghost", size: "sm", class: "w-fit group" })}',1)
s=s.replace('class="w-full bg-bg-elevated hover:bg-amber-500/10 border border-amber-500/30 rounded-2xl p-4 flex items-center gap-4 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer text-left group"','class={button({ variant: "secondary", class: "w-full h-auto justify-start gap-4 whitespace-normal p-4 text-left group border-amber-500/25" })}')
s=s.replace('class="w-full bg-bg-elevated hover:bg-rose-500/10 border border-rose-500/30 rounded-2xl p-4 flex items-center gap-4 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer text-left group"','class={button({ variant: "danger", class: "w-full h-auto justify-start gap-4 whitespace-normal p-4 text-left group" })}')
s=s.replace('class="p-3 rounded-2xl border text-xs font-bold flex items-center justify-between transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {instanceLoaderType === loader ? \'bg-brand-500/20 border-brand-500 text-brand-500\' : \'bg-bg-elevated border-fg/10 text-fg/60 hover:text-fg\'}"','aria-pressed={instanceLoaderType === loader}\n                                            class={button({ variant: instanceLoaderType === loader ? "primary" : "secondary", class: "justify-between" })}')
p.write_text(s,encoding='utf-8')
p=Path('src/routes/settings/+page.svelte')
s=p.read_text(encoding='utf-8')
s=s.replace('\t\tDatabase\n','\t\tDatabase,\n        SlidersHorizontal\n')
s=s.replace('\tlet gameResolution = $state(settings.value.startFullscreen ? "fullscreen" : "1920x1080");','''    let gameResolution = $state(settings.value.startFullscreen ? "fullscreen" : `${settings.value.defaultResWidth ?? 1920}x${settings.value.defaultResHeight ?? 1080}`);
    let customResWidth = $state(settings.value.defaultResWidth ?? 1920);
    let customResHeight = $state(settings.value.defaultResHeight ?? 1080);''')
s=s.replace('\tfunction saveGeneral() {\n', '''    const settingsTabs = $derived([
        { id: "general", label: t("settings.tabs.general"), icon: Home },
        { id: "accounts", label: t("settings.tabs.accounts"), icon: Users },
        { id: "language", label: t("settings.tabs.language"), icon: Globe },
        { id: "appearance", label: t("settings.tabs.appearance"), icon: Palette },
        { id: "java", label: t("settings.tabs.java"), icon: Coffee },
        { id: "commands", label: t("settings.tabs.commands"), icon: Terminal },
        { id: "privacy", label: t("settings.tabs.privacy"), icon: ShieldCheck },
        { id: "runtime", label: t("settings.tabs.runtime"), icon: FolderOpen }
    ]);
    function handleSettingsTabKeys(event: KeyboardEvent) {
        if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
        const tabs = Array.from((event.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>("[role='tab']"));
        const index = tabs.indexOf(event.target as HTMLButtonElement);
        if (index < 0) return;
        event.preventDefault();
        const next = event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 : (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
        tabs[next].focus();
        tabs[next].click();
    }
    function updateDefaultResolution() {
        customResWidth = Math.max(640, Math.min(7680, Math.round(Number(customResWidth) || 1920)));
        customResHeight = Math.max(360, Math.min(4320, Math.round(Number(customResHeight) || 1080)));
        settings.patch({ defaultResWidth: customResWidth, defaultResHeight: customResHeight, startFullscreen: false });
        gameResolution = "custom";
        schedulePersist();
    }
\tfunction saveGeneral() {
        const dimensions = /^([0-9]+)x([0-9]+)$/.exec(gameResolution);
        if (dimensions) { customResWidth = Number(dimensions[1]); customResHeight = Number(dimensions[2]); }
        if (gameResolution === "default") { customResWidth = 854; customResHeight = 480; }
''')
s=s.replace('\t\t\tstartFullscreen: gameResolution === "fullscreen",','\t\t\tstartFullscreen: gameResolution === "fullscreen",\n            defaultResWidth: customResWidth,\n            defaultResHeight: customResHeight,',1)
s=s.replace('<div class="settings-page flex flex-col pr-2 pb-16 max-w-5xl mx-auto w-full">','<div class="settings-page flex flex-col gap-6 pb-16 max-w-6xl mx-auto w-full">')
a=s.index('\t<div class="flex items-center gap-1.5 p-1.5 rounded-2xl')
b=s.index('\n\t{#if activeTab === "general"}',a)
s=s[:a]+'''    <header class="flex items-center gap-4">
        <span class="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl border border-brand-400/20 bg-brand-500/10 text-brand-400"><SlidersHorizontal class="h-6 w-6" /></span>
        <div><p class="page-eyebrow">Luxmc</p><h1 class="mt-1 text-2xl font-semibold tracking-tight text-fg">{uiText("settings.refinement.settingsTitle")}</h1></div>
    </header>
    <div class="surface-glass grid grid-cols-2 gap-2 p-2 sm:grid-cols-4 xl:grid-cols-8" role="tablist" aria-label={uiText("settings.refinement.settingsTitle")} tabindex="-1" onkeydown={handleSettingsTabKeys}>
        {#each settingsTabs as tab}
            <button type="button" role="tab" id={`settings-tab-${tab.id}`} aria-controls="settings-panel" aria-selected={activeTab === tab.id} tabindex={activeTab === tab.id ? 0 : -1} onclick={() => activeTab = tab.id as SettingsTab} class={launcherButton({ variant: activeTab === tab.id ? "primary" : "ghost", class: "h-auto min-h-16 w-full flex-col gap-2 px-3 py-3" })}><tab.icon class="h-5 w-5" /><span>{tab.label}</span></button>
        {/each}
    </div>
    <div id="settings-panel" role="tabpanel" aria-labelledby={`settings-tab-${activeTab}`} class="space-y-6">
'''+s[b:]
s=s.replace('    <SettingsExtras section={activeTab} />','    <SettingsExtras section={activeTab} />\n    </div>')
s=s.replace('<option value="fullscreen">{t("settings.resolutionFullscreen")}</option>','<option value="fullscreen">{t("settings.resolutionFullscreen")}</option>\n                        <option value="custom">{uiText("settings.refinement.customResolution")}</option>',1)
marker='''				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<div class="flex items-center gap-2">
							<h3 class="text-sm font-bold text-fg">{t("settings.discordRpcTitle")}</h3>'''
s=s.replace(marker,'''                {#if gameResolution === "custom"}
                    <div class="grid grid-cols-1 gap-4 py-5 sm:grid-cols-2">
                        <label class="space-y-2 text-sm text-fg"><span>{uiText("settings.refinement.resolutionWidth")}</span><input type="number" min="640" max="7680" step="1" bind:value={customResWidth} onchange={updateDefaultResolution} class="w-full rounded-xl border border-border bg-bg-elevated p-3 text-fg" /></label>
                        <label class="space-y-2 text-sm text-fg"><span>{uiText("settings.refinement.resolutionHeight")}</span><input type="number" min="360" max="4320" step="1" bind:value={customResHeight} onchange={updateDefaultResolution} class="w-full rounded-xl border border-border bg-bg-elevated p-3 text-fg" /></label>
                    </div>
                {/if}
'''+marker,1)
s=s.replace('class="px-4 py-2 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg text-xs font-bold border border-fg/10 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer shadow-sm active:scale-[0.98]"','class={launcherButton({ variant: "secondary" })}',1)
p.write_text(s,encoding='utf-8')

