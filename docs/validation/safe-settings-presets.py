from pathlib import Path
import json
p=Path('src/routes/settings/+page.svelte');s=p.read_text(encoding='utf-8')
s=s.replace('\tlet useVulkan = $state(false);\n','')
s=s.replace('\t\tif (typeof window !== "undefined") {\n\t\t\tuseVulkan = localStorage.getItem("luxmc_enable_vulkan") === "true";\n\t\t}\n','')
s=s.replace('\t\tif (typeof window !== "undefined") {\n\t\t\tlocalStorage.setItem("luxmc_enable_vulkan", String(useVulkan));\n\t\t}\n','')
a=s.index('\tfunction setJvmPreset(');b=s.index('\n\tlet preLaunchCmd',a)
s=s[:a]+'''    function setJvmPreset(preset: "automatic" | "g1gc") {
        jvmArgsInput = preset === "g1gc" ? "-XX:+UseG1GC -XX:MaxGCPauseMillis=200" : "";
        saveJava();
    }
'''+s[b:]
a=s.rfind('<div class="flex items-center justify-between py-5 gap-6">',0,s.index('{t("settings.vulkanTitle")}'))
b=s.index('</button>',a)+len('</button>');b=s.index('</div>',b)+len('</div>')
s=s[:a]+s[b:]
a=s.index('''						<span class="text-[11px] font-semibold text-fg/40 mr-1">{uiText("ui.1d096828d28c5cc3")}</span>''')
b=s.index('\n\t\t\t\t\t</div>',a)
s=s[:a]+'''                        <span class="text-[11px] font-semibold text-fg-muted mr-1">{uiText("ui.1d096828d28c5cc3")}</span>
                        <button type="button" onclick={() => setJvmPreset('automatic')} class={launcherButton({ variant: jvmArgsInput ? "secondary" : "primary", size: "sm" })}>{uiText("settings.refinement.automaticJvm")}</button>
                        <button type="button" onclick={() => setJvmPreset('g1gc')} class={launcherButton({ variant: jvmArgsInput === '-XX:+UseG1GC -XX:MaxGCPauseMillis=200' ? "primary" : "secondary", size: "sm" })}>G1GC</button>
                        <p class="w-full pt-2 text-xs leading-relaxed text-fg-muted">{uiText("settings.refinement.jvmPresetsDesc")}</p>
'''+s[b:]
s=s.replace('toast(`RAM recomendada aplicada: ${maxRamGb} GB.`, "success");','toast(uiText("settings.refinement.ramApplied", { amount: maxRamGb }), "success");')
s=s.replace('class="px-3 py-1 rounded-lg text-xs font-mono font-bold border transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {maxRamGb === preset ? \'bg-blue-600 border-blue-500 text-fg shadow-md\' : \'bg-fg/5 border-fg/10 text-fg/70 hover:bg-fg/10 hover:text-fg\'}"','class={launcherButton({ variant: maxRamGb === preset ? "primary" : "secondary", size: "sm" })}')
s=s.replace('class="px-3 py-1 rounded-lg text-xs font-sans font-bold border transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {maxRamGb === 6 ? \'bg-emerald-600/30 border-emerald-500/50 text-emerald-300\' : \'bg-fg/5 border-fg/10 text-fg/50 hover:bg-fg/10 hover:text-fg\'}"','class={launcherButton({ variant: "secondary", size: "sm" })}')
s=s.replace('{recommendingRam ? "Verificando…" : "Recomendar RAM"}','{uiText(recommendingRam ? "settings.checking" : "settings.refinement.recommendRam")}')
p.write_text(s,encoding='utf-8')
p=Path('src/routes/instances/[id]/+page.svelte');s=p.read_text(encoding='utf-8')
s=s.replace('''function applyJvmPreset(preset: "aikar" | "zgc" | "shenandoah") {
		instanceAutoOptimize = preset === "aikar";
		instanceJvmArgs = preset === "aikar" ? "" : preset === "zgc" ? "-XX:+UseZGC" : "-XX:+UseShenandoahGC";
	}''','''function applyJvmPreset(preset: "automatic" | "g1gc") {
        instanceAutoOptimize = preset === "automatic";
        instanceJvmArgs = preset === "g1gc" ? "-XX:+UseG1GC -XX:MaxGCPauseMillis=200" : "";
    }''')
s=s.replace('''{#each ['aikar', 'zgc', 'shenandoah'] as preset}<button type="button" class={button({ variant: "outline", size: "sm" })} onclick={() => applyJvmPreset(preset as 'aikar' | 'zgc' | 'shenandoah')}>{preset.toUpperCase()}</button>{/each}''','''{#each ['automatic', 'g1gc'] as preset}<button type="button" class={button({ variant: (preset === 'automatic' ? instanceAutoOptimize : instanceJvmArgs.includes('-XX:+UseG1GC')) ? "primary" : "secondary", size: "sm" })} onclick={() => applyJvmPreset(preset as 'automatic' | 'g1gc')}>{preset === 'automatic' ? uiText("settings.refinement.automaticJvm") : 'G1GC'}</button>{/each}''')
p.write_text(s,encoding='utf-8')
p=Path('tests/instance-content.browser.cjs');s=p.read_text(encoding='utf-8').replace("{ name: 'ZGC', exact: true }","{ name: 'G1GC', exact: true }").replace("savedSettings.jvmArgs.includes('-XX:+UseZGC')","savedSettings.jvmArgs.includes('-XX:+UseG1GC')");p.write_text(s,encoding='utf-8')
p=Path('tests/languages.browser.cjs');s=p.read_text(encoding='utf-8')
s=s.replace("{ en: 'More options', es: 'Más opciones', 'pt-BR': 'Mais opções' }", "{ en: 'Updates, audio and diagnostics', es: 'Actualizaciones, audio y diagnóstico', 'pt-BR': 'Atualizações, áudio e diagnóstico' }")
s=s.replace("getByRole('button', { name: 'Language', exact: true })", "getByRole('tab', { name: 'Language', exact: true })")
p.write_text(s,encoding='utf-8')
p=Path('tests/windows-launcher.browser.cjs');s=p.read_text(encoding='utf-8').replace("getByRole('button',{name:'Java',exact:true})", "getByRole('tab',{name:'Java',exact:true})");p.write_text(s,encoding='utf-8')
p=Path('docs/validation/settings-design-translations.json');catalog=json.loads(p.read_text(encoding='utf-8'));entries={
'automaticJvm':['Automático (recomendado)','Automatic (recommended)','Automático (recomendado)'],
'jvmPresetsDesc':['O modo automático usa parâmetros compatíveis com a versão do Java do jogo. G1GC é uma alternativa conservadora; configurações personalizadas continuam disponíveis acima.','Automatic mode selects settings compatible with the game\'s Java version. G1GC is a conservative alternative; custom settings remain available above.','El modo automático usa parámetros compatibles con la versión de Java del juego. G1GC es una alternativa conservadora; la configuración personalizada sigue disponible arriba.'],
'ramApplied':['RAM recomendada aplicada: {{amount}} GB.','Recommended RAM applied: {{amount}} GB.','RAM recomendada aplicada: {{amount}} GB.'],
'recommendRam':['Recomendar RAM','Recommend RAM','Recomendar RAM']}
for i,language in enumerate(['pt-BR','en','es']):catalog[language]['settings']['refinement'].update({k:v[i] for k,v in entries.items()})
p.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
