from pathlib import Path
import json
path=Path('src/lib/i18n/en.json')
data=json.loads(path.read_text(encoding='utf8'))
updates={
    'home': {
        'confirmAccess':'Confirm access','selectSavedAccount':'Select a saved account',
        'noSavedAccountDesc':'You have no saved local accounts yet. Create one below.',
        'tipOffline':'Local accounts do not require a password or internet connection.',
        'emptyInstancesTitle':'Your library is empty','emptyInstancesSub':'Create your first Minecraft instance to get started.',
        'createNewInstance':'Create a new instance','heroBadge':'High performance on Windows and Linux',
        'welcomeTitle':'Welcome to Luxmc','selectAccountToPlay':'Select an account to play',
        'versionUnavailable':'Version unavailable','totalPlayTime':'Total play time','instancesCount':'Instances',
        'installedMods':'Installed mods','customSkin':'Custom skin','noSkinDesc':'Customize your appearance in the customization tab.',
        'openSkins':'Open customization','activeInstance':'Active instance','viewInstance':'View instance details',
        'noActiveInstance':'No instance selected','noActiveInstanceDesc':'Click an instance in the library to select it.',
        'viewLibrary':'Go to library','manageInstance':'Manage','systemStatus':'System status','fastLaunch':'Fast launch',
        'performanceStatus':'Maximum performance','performanceDesc':'Hardware acceleration enabled',
        'latestReleases':'Latest releases','viewChangelog':'View changelog'
    },
    'settings': {'javaMemoryHint':'RAM reserved for the Minecraft virtual machine','jvmArguments':'JVM arguments','jvmArgumentsHint':'Advanced parameters passed to Java at launch'},
    'common': {'success':'Success','warning':'Warning'},
    'mods': {'installing':'Installing…','createInstance':'Create instance','sourceCurseForge':'CurseForge','sourceModrinth':'Modrinth','categories':'Categories','allLoaders':'All mod loaders','sortBy':'Sort by','downloads':'Downloads','relevance':'Relevance'}
}
for section,values in updates.items():data[section].update(values)
path.write_text(json.dumps(data,ensure_ascii=False,indent='\t')+'\n',encoding='utf8')
