const fs = require('node:fs');
const path = require('node:path');

async function setupLauncherDemo(page, { login = false, room = false } = {}) {
    const png = name => `data:image/png;base64,${fs.readFileSync(path.join(__dirname, '..', 'static', name)).toString('base64')}`;
    await page.addInitScript(({ login, room, textures }) => {
        const stateKey = 'luxmc_video_demo_state';
        const timestamp = '2026-10-05T12:00:00.000Z';
        const demoUuid = '00000000-0000-4000-8000-000000000001';
        const account = { id: `luxmc:${demoUuid}`, uuid: demoUuid.replaceAll('-', ''), username: 'LuxPlayer', accessToken: '', refreshToken: '', expiresAt: null, createdAt: timestamp, updatedAt: timestamp, skinUrl: textures.alex, skinVariant: 'slim', capeUrl: null, avatarUrl: textures.grass };
        const baseProfile = { loaderVersion: null, javaPath: null, jvmArgs: null, resolutionW: 1280, resolutionH: 720, fullscreen: false, ramMb: 4096, autoOptimize: true, useVulkan: false, useGamemode: false, useMangohud: false, forceDedicatedGpu: false, useGamescope: false, forceFullVerification: false, preLaunchHook: null, postExitHook: null, favorite: false, notes: 'Instância de demonstração para o vídeo.', lastPlayed: timestamp, launchCount: 8, diskUsage: 1509949440, instanceGroup: 'Demonstração', createdAt: timestamp, updatedAt: timestamp };
        const profiles = [
            { ...baseProfile, id: 'demo-vanilla', name: 'Vanilla 1.21.1', icon: '/grass_block.png', mcVersion: '1.21.1', loader: 'vanilla', gameDir: 'C:\\Luxmc\\Demonstracao\\vanilla', modCount: 0, favorite: true, banner: '/vanilla_banner.png' },
            { ...baseProfile, id: 'demo-vp', name: 'Vanilla Perfected', icon: '/modpack_vanilla_perfected_icon.webp', mcVersion: '1.21.1', loader: 'fabric', loaderVersion: '0.16.9', gameDir: 'C:\\Luxmc\\Demonstracao\\vanilla-perfected', modCount: 8, favorite: true, banner: '/modpack_vanilla_perfected.png' },
            { ...baseProfile, id: 'demo-fo', name: 'Fabulously Optimized', icon: '/modpack_fo_icon.png', mcVersion: '1.21.1', loader: 'fabric', loaderVersion: '0.16.9', gameDir: 'C:\\Luxmc\\Demonstracao\\fabulously-optimized', modCount: 8, banner: '/modpack_fo.webp' }
        ];
        const friend = (id, username, status, extra = {}) => ({ id, username, status, incoming: false, avatarUrl: textures.grass, lastSeen: null, activity: null, mcVersion: null, loader: null, serverIp: null, serverPort: null, ...extra });
        const me = { id: demoUuid, username: 'LuxPlayer', avatarUrl: textures.grass };
        const contacts = [
            friend('00000000-0000-4000-8000-000000000002', 'AlexBuilder', 'in_game', { activity: 'Vanilla Perfected', mcVersion: '1.21.1', loader: 'fabric', serverIp: '127.0.0.1', serverPort: 54321 }),
            friend('00000000-0000-4000-8000-000000000003', 'SteveCraft', 'online'),
            friend('00000000-0000-4000-8000-000000000004', 'PixelExplorer', 'offline', { lastSeen: timestamp }),
            friend('00000000-0000-4000-8000-000000000005', 'NovaAventura', 'pending', { incoming: true }),
            friend('00000000-0000-4000-8000-000000000006', 'BlockArtist', 'pending')
        ];
        const members = [
            { id: 'demo-host', username: 'LuxPlayer', uuid: account.uuid, avatarUrl: textures.grass, joinedAt: 1791201600, isHost: true },
            { id: 'demo-guest', username: 'AlexBuilder', uuid: '00000000000040008000000000000002', avatarUrl: textures.grass, joinedAt: 1791201610, isHost: false }
        ];
        const demoRoom = { mode: 'host', invitation: 'LUXMC1234567890', roomCode: 'LUXMC1234567890', localAddress: null, expiresAt: 1893456000, pingMs: 18, transport: 'QUIC', members, maxPlayers: 10, roomLocked: false };
        const newsFeed = { entries: [
            { id: 'demo-news-1', title: 'Explore novos horizontes', date: '2026-10-05', category: 'Demonstração', text: 'Notícias, mapas e novidades em um só lugar. Cartão de demonstração.', readMoreLink: 'https://www.minecraft.net/article/luxmc-demo-1', newsPageImage: { url: 'https://launchercontent.mojang.com/luxmc-demo-news-1.jpg' } },
            { id: 'demo-news-2', title: 'Seu próximo mundo começa aqui', date: '2026-10-04', category: 'Demonstração', text: 'Acompanhe as novidades de Minecraft pelo launcher. Cartão de demonstração.', readMoreLink: 'https://www.minecraft.net/article/luxmc-demo-2', newsPageImage: { url: 'https://launchercontent.mojang.com/luxmc-demo-news-2.jpg' } },
            { id: 'demo-news-3', title: 'Mais aventuras com a sua turma', date: '2026-10-03', category: 'Demonstração', text: 'Organize seus amigos e descubra novas aventuras. Cartão de demonstração.', readMoreLink: 'https://www.minecraft.net/article/luxmc-demo-3', newsPageImage: { url: 'https://launchercontent.mojang.com/luxmc-demo-news-3.jpg' } }
        ] };
        const defaultSettings = { theme: 'default-dark', accentTheme: 'blue', customBackground: 'cosmos', language: 'pt-BR', languageMode: 'manual', animations: true, liveWallpaper: true, wallpaperFps: 30, wallpaperIntensity: 0.45, blur: true, interfaceOpacity: 18, density: 'default', soundEnabled: false, soundscapesEnabled: false, sfxVolume: 0.5, discordRichPresence: true, autoCheckUpdates: false, defaultResWidth: 1280, defaultResHeight: 720, startFullscreen: false, defaultRamMb: 4096, downloadThreads: 8, releaseChannel: 'stable', activeProfileId: 'demo-vp' };
        let state;
        try { state = JSON.parse(sessionStorage.getItem(stateKey) || 'null'); } catch {}
        state ||= { account: login ? null : account, profiles, contacts, room: room ? demoRoom : null, settings: defaultSettings, selectedProfileId: 'demo-vp', revision: 1 };
        state.backups ||= [
            { fileName: 'mundo-demo-2026-10-05.zip', filePath: 'C:\\Luxmc\\Demonstracao\\backups\\mundo-demo-2026-10-05.zip', sizeBytes: 25165824, createdAt: '2026-10-05 12:00:00', worldName: 'Mundo de demonstração' },
            { fileName: 'mundo-demo-2026-10-04.zip', filePath: 'C:\\Luxmc\\Demonstracao\\backups\\mundo-demo-2026-10-04.zip', sizeBytes: 23068672, createdAt: '2026-10-04 12:00:00', worldName: 'Mundo de demonstração' }
        ];
        const persist = () => sessionStorage.setItem(stateKey, JSON.stringify(state));
        persist();
        localStorage.setItem('luxmc.locale', 'pt-BR');
        localStorage.setItem('luxmc_theme', 'dark');
        localStorage.setItem('luxmc_accent', 'blue');
        localStorage.setItem('luxmc_background', 'cosmos');
        localStorage.setItem('luxmc_sidebar_expanded', 'true');
        localStorage.setItem('luxmc_minecraft_news_v2', JSON.stringify({ feed: newsFeed, fetchedAt: Date.now() }));
        for (const profile of state.profiles) localStorage.setItem(`luxmc_banner_${profile.id}`, profile.banner);
        if (!localStorage.getItem('luxmc_saved_skins')) {
            localStorage.setItem('luxmc_saved_skins', JSON.stringify([{ id: 'demo-alex', name: 'Alex • Demonstração', url: textures.alex, model: 'alex' }, { id: 'demo-steve', name: 'Steve • Demonstração', url: textures.steve, model: 'steve' }]));
            localStorage.setItem('luxmc_selected_skin_id', 'demo-alex');
            localStorage.setItem(`luxmc_selected_skin_id:${account.id}`, 'demo-alex');
            localStorage.setItem('luxmc_active_skin_data', JSON.stringify({ id: 'demo-alex', name: 'Alex • Demonstração', skinUrl: textures.alex, avatarUrl: textures.grass, type: 'alex', hasCape: false, capeType: 'none', customCapeUrl: '' }));
        }
        const catalogs = {
            modpack: [
                { slug: 'vanilla-perfected', title: 'Vanilla Perfected', description: 'Uma seleção de melhorias para a experiência Vanilla. Dados de demonstração.', iconUrl: '/modpack_vanilla_perfected_icon.webp', bannerUrl: '/modpack_vanilla_perfected.png', author: 'Vanilla Perfected', categories: ['fabric', 'optimization', 'lightweight'] },
                { slug: 'fabulously-optimized', title: 'Fabulously Optimized', description: 'Desempenho, gráficos e familiaridade em um modpack leve. Dados de demonstração.', iconUrl: '/modpack_fo_icon.png', bannerUrl: '/modpack_fo.webp', author: 'robino459', categories: ['fabric', 'optimization', 'lightweight'] },
                { slug: 'better-mc', title: 'Better MC', description: 'Aventura e exploração em novos mundos. Dados de demonstração.', iconUrl: '/modpack_bmc_icon.webp', bannerUrl: '/modpack_better_mc.webp', author: 'LunaPixelStudios', categories: ['forge', 'adventure', 'magic'] },
                { slug: 'cobblemon', title: 'Cobblemon', description: 'Explore um mundo de criaturas e descobertas. Dados de demonstração.', iconUrl: '/modpack_cobblemon_icon.png', bannerUrl: '/modpack_cobblemon.webp', author: 'Cobblemon', categories: ['fabric', 'adventure'] }
            ],
            mod: [
                { slug: 'sodium', title: 'Sodium', description: 'Renderização otimizada para Minecraft. Metadados de demonstração.', categories: ['fabric', 'optimization'], author: 'CaffeineMC' },
                { slug: 'fabric-api', title: 'Fabric API', description: 'APIs essenciais para mods Fabric. Metadados de demonstração.', categories: ['fabric', 'library'], author: 'FabricMC' },
                { slug: 'iris', title: 'Iris Shaders', description: 'Compatibilidade com shader packs. Metadados de demonstração.', categories: ['fabric', 'optimization'], author: 'IrisShaders' },
                { slug: 'modmenu', title: 'Mod Menu', description: 'Gerencie os mods instalados. Metadados de demonstração.', categories: ['fabric', 'utility'], author: 'TerraformersMC' }
            ],
            resourcepack: [{ slug: 'faithful', title: 'Faithful 32x', description: 'Texturas com estilo Vanilla. Dados de demonstração.', categories: ['32x', 'vanilla-like'] }],
            shader: [{ slug: 'complementary-reimagined', title: 'Complementary Reimagined', description: 'Iluminação e atmosfera para seus mundos. Dados de demonstração.', categories: ['iris', 'realistic'] }],
            datapack: [{ slug: 'terralith', title: 'Terralith', description: 'Novos biomas para explorar. Dados de demonstração.', categories: ['worldgen'] }],
            world: [{ slug: 'mundo-demo', title: 'Mundo de demonstração', description: 'Uma base preparada para mostrar o gerenciador de mundos.', categories: ['adventure'] }]
        };
        for (const [type, items] of Object.entries(catalogs)) items.forEach((item, index) => Object.assign(item, { source: 'modrinth', sourceId: `demo-${item.slug}`, downloads: (index + 1) * 145000, versions: ['1.21.1'], iconUrl: item.iconUrl || null, bannerUrl: item.bannerUrl || (type === 'shader' ? '/modpack_fo.webp' : type === 'world' ? '/bg_day.jpg' : '/vanilla_banner.png') }));
        const mods = ['Fabric API', 'Sodium', 'Iris Shaders', 'Lithium', 'FerriteCore', 'Mod Menu', 'Entity Culling', 'ImmediatelyFast'].map((name, index) => ({ name: `${name.toLowerCase().replaceAll(' ', '-')}-demo.jar`, path: `mods/${name.toLowerCase().replaceAll(' ', '-')}-demo.jar`, isDir: false, size: (index + 1) * 540000, icon: null }));
        const allProjects = Object.values(catalogs).flat();
        const createHead = source => new Promise((resolve, reject) => {
            const image = new Image();
            image.onload = () => {
                const canvas = document.createElement('canvas');
                canvas.width = 64; canvas.height = 64;
                const context = canvas.getContext('2d');
                context.imageSmoothingEnabled = false;
                context.drawImage(image, 8, 8, 8, 8, 0, 0, 64, 64);
                context.drawImage(image, 40, 8, 8, 8, 0, 0, 64, 64);
                resolve(canvas.toDataURL('image/png'));
            };
            image.onerror = reject;
            image.src = source;
        });
        const avatarReady = Promise.all([createHead(textures.alex), createHead(textures.steve)]).then(([alex, steve]) => {
            textures.alexHead = alex; textures.steveHead = steve;
            me.avatarUrl = alex; account.avatarUrl = alex;
            if (state.account) state.account.avatarUrl = state.account.skinVariant === 'classic' ? steve : alex;
            state.contacts.forEach((contact, index) => { contact.avatarUrl = index % 2 ? steve : alex; });
            members[0].avatarUrl = alex; members[1].avatarUrl = steve;
            if (state.room) state.room.members.forEach((member, index) => { member.avatarUrl = index % 2 ? steve : alex; });
            persist();
        });
        const callbacks = new Map();
        const listeners = new Map();
        const nativeListeners = new Map();
        let callbackId = 1;
        const emit = (event, payload) => {
            for (const handler of listeners.get(event) || []) handler(payload);
            for (const listener of nativeListeners.values()) if (listener.event === event) callbacks.get(listener.handler)?.({ event, id: listener.id, payload });
        };
        const currentProfile = id => state.profiles.find(profile => profile.id === id) || state.profiles[1];
        const portal = () => ({ id: demoUuid, username: 'LuxPlayer', socialId: demoUuid, preferences: { theme: state.settings.theme, accentTheme: state.settings.accentTheme, language: state.settings.language, animations: state.settings.animations }, revision: state.revision });
        const conflicts = { hasConflicts: false, conflicts: [], duplicates: [] };
        const storage = () => ({ totalBytes: 5964308480, categories: [{ category: 'application', bytes: 41943040, path: 'C:\\Luxmc\\Demonstracao\\Luxmc.exe' }, { category: 'java', bytes: 367001600, path: 'C:\\Luxmc\\Demonstracao\\java' }, { category: 'launcher_data', bytes: 23068672, path: 'C:\\Luxmc\\Demonstracao\\data' }, { category: 'instances', bytes: 4529848320, path: 'C:\\Luxmc\\Demonstracao' }, { category: 'cache', bytes: 1002438656, path: 'C:\\Luxmc\\Demonstracao\\cache' }], instances: state.profiles.map(profile => ({ id: profile.id, name: profile.name, icon: profile.icon, loader: profile.loader, mcVersion: profile.mcVersion, bytes: profile.diskUsage, path: profile.gameDir })), logsBytes: 20971520, cacheBytes: 1002438656, warnings: [] });
        const invoke = async (command, args = {}) => {
            window.launcherDemo.calls.push({ command, args });
            if (command === 'app_init') { await avatarReady; return { account: state.account, profiles: state.profiles, activeProfileId: state.selectedProfileId, devMode: false }; }
            if (command === 'app_system_locale') return 'pt-BR';
            if (command === 'plugin:app|version') return '3.0.0';
            if (command === 'plugin:app|name') return 'Luxmc';
            if (command === 'plugin:store|load') return 1;
            if (command === 'plugin:store|get') return args.key === 'app' ? [state.settings, true] : args.key === 'current_account' && state.account ? [{ ...state.account, minecraftToken: state.account.accessToken || '' }, true] : [null, false];
            if (command === 'plugin:store|set') { if (args.key === 'app') state.settings = args.value; if (args.key === 'current_account') state.account = { ...args.value, accessToken: args.value.minecraftToken || '' }; persist(); return null; }
            if (command === 'plugin:store|delete') { if (args.key === 'current_account') state.account = null; persist(); return true; }
            if (command === 'plugin:store|save') { persist(); return null; }
            if (command === 'plugin:event|listen') { const id = callbackId++; nativeListeners.set(id, { id, event: args.event, handler: args.handler }); return id; }
            if (command === 'plugin:event|unlisten') { nativeListeners.delete(args.eventId); return null; }
            if (command === 'plugin:event|emit') { emit(args.event, args.payload); return null; }
            if (command === 'plugin:window|is_maximized') return true;
            if (command === 'plugin:window|get_all_windows') return ['main'];
            if (command === 'plugin:window|is_visible' || command === 'plugin:window|is_focused') return true;
            if (command === 'plugin:window|inner_size') return { width: 1920, height: 1080 };
            if (command === 'plugin:window|scale_factor') return 1;
            if (command === 'profiles_list' || command === 'instances_list') return structuredClone(state.profiles);
            if (command === 'profiles_get') return structuredClone(currentProfile(args.id));
            if (command === 'profiles_update') { const profile = currentProfile(args.input.id); Object.assign(profile, args.input, { updatedAt: timestamp }); persist(); return structuredClone(profile); }
            if (command === 'profiles_create') { const profile = { ...baseProfile, ...args.input, id: `demo-created-${state.profiles.length + 1}`, gameDir: 'C:\\Luxmc\\Demonstracao\\nova', modCount: 0, banner: '/vanilla_banner.png' }; state.profiles.push(profile); persist(); return profile; }
            if (command === 'profiles_delete') { state.profiles = state.profiles.filter(profile => profile.id !== args.id); persist(); return null; }
            if (command === 'social_request') {
                await avatarReady;
                const request = args.request || {};
                if (request.action === 'stream_ticket') return { url: null };
                if (request.action === 'register') return { me };
                if (request.action === 'sync') return { me, friends: structuredClone(state.contacts) };
                if (request.action === 'search') return { users: state.contacts.filter(contact => contact.username.toLowerCase().includes((request.query || '').toLowerCase())) };
                if (request.action === 'accept') state.contacts = state.contacts.map(contact => contact.id === request.targetId ? { ...contact, status: 'online', incoming: false } : contact);
                if (request.action === 'remove' || request.action === 'block') state.contacts = state.contacts.filter(contact => contact.id !== request.targetId);
                if (request.action === 'room_create') return { code: '123456', expiresAt: 1893456000 };
                if (request.action === 'room_join') return { host: '127.0.0.1', port: 54321 };
                persist(); return { ok: true };
            }
            if (command === 'lux_account_sync') { if (args.preferences) { Object.assign(state.settings, args.preferences); state.revision += 1; persist(); } return portal(); }
            if (command === 'lux_account_login' || command === 'auth_offline_login' || command === 'auth_dev_login') { state.account = { ...account, username: args.username || 'LuxPlayer' }; persist(); return state.account; }
            if (command === 'auth_login') { state.account = { ...account, id: 'demo-microsoft-account', accessToken: 'DEMONSTRACAO_SEM_CREDENCIAL_REAL_'.repeat(6), refreshToken: '', expiresAt: 1893456000000 }; persist(); return state.account; }
            if (command === 'auth_accounts') return state.account ? [state.account] : [];
            if (command === 'auth_save_appearance') { Object.assign(state.account, { skinUrl: args.skinUrl, skinVariant: args.variant, capeUrl: args.capeUrl }); persist(); return null; }
            if (command === 'auth_resolve_texture' || command === 'auth_read_local_texture') return textures.alex;
            if (command === 'auth_change_skin' || command === 'auth_set_account_cape') return null;
            if (command === 'namemc_open_picker' || command === 'namemc_close_picker') return null;
            if (command === 'namemc_import_skin') return { skinId: '63455d7069b397c2', skinUrl: textures.alex };
            if (command === 'skins_list') return JSON.parse(localStorage.getItem('luxmc_saved_skins') || '[]').map(item => ({ ...item, skinUrl: item.url, modelType: item.model === 'alex' ? 'slim' : 'classic', avatarUrl: textures.grass, isCustom: true, createdAt: timestamp }));
            if (command === 'capes_list' || command === 'performance_history' || command === 'dependency_graph') return [];
            if (command === 'bedrock_state') return {provider:null,installations:[],instances:[],warning:null,supported:true};
            if (command === 'bedrock_versions') return [{id:'gdk-release-1.26.52.3',version:'1.26.52.3',channel:'release',packageType:'GDK',packageVersion:'1.26.5203.0',urls:[]},{id:'uwp-validation',version:'1.20.81.01',channel:'release',packageType:'UWP',packageVersion:'1.20.8101.0',urls:[]}];
            if (command === 'minecraft_uuid') return account.uuid;
            if (command === 'p2p_scan_lan_worlds') return [{ host: '127.0.0.1', port: 54321, motd: 'Mundo de demonstração' }];
            if (command === 'p2p_get_local_info') return { ip: '127.0.0.1', port: 54321 };
            if (command === 'p2p_get_host_link') return { localIp: '127.0.0.1', publicIp: null, port: 54321, code: '123456', link: '123456' };
            if (command === 'tunnel_status') return state.room ? structuredClone(state.room) : null;
            if (command === 'host_world') { state.room = structuredClone(demoRoom); persist(); return state.room; }
            if (command === 'join_world') { state.room = { ...structuredClone(demoRoom), mode: 'client', localAddress: '127.0.0.1:25565' }; persist(); return state.room; }
            if (command === 'stop_session') { state.room = null; persist(); return null; }
            if (command === 'tunnel_save_server') return '127.0.0.1:25565';
            if (command === 'tunnel_set_locked') { if (state.room) state.room.roomLocked = args.locked; persist(); return null; }
            if (command === 'tunnel_kick_member') { if (state.room) state.room.members = state.room.members.filter(member => member.id !== args.memberId); persist(); return null; }
            if (command === 'tunnel_refresh_invitation') return state.room || structuredClone(demoRoom);
            if (command === 'versions_list') return { latestRelease: '1.21.1', latestSnapshot: 'snapshot-demo', versions: [{ id: '1.21.1', versionType: 'release', releaseTime: '2024-08-08T10:00:00Z', url: '' }, { id: '1.20.4', versionType: 'release', releaseTime: '2023-12-07T10:00:00Z', url: '' }, { id: 'snapshot-demo', versionType: 'snapshot', releaseTime: timestamp, url: '' }] };
            if (command === 'versions_check_installed') return true;
            if (command === 'loaders_versions') return { versions: [{ id: '0.16.9', stable: true }, { id: '0.16.8', stable: true }] };
            if (command === 'mods_search' || command === 'mods_search_typed') { const items = catalogs[args.contentType || 'modpack'] || catalogs.modpack; return items.filter(item => item.title.toLowerCase().includes((args.query || '').toLowerCase())).slice(args.offset || 0, (args.offset || 0) + (args.limit || 36)); }
            if (command === 'mods_versions') return [{ id: 'demo-version-1', name: 'Versão de demonstração • Minecraft 1.21.1', versionNumber: '1.0-demo', loaders: ['fabric'], files: [{ url: '', filename: 'demo.mrpack', size: 1048576, sha1: '' }] }, { id: 'demo-version-previous', name: 'Versão anterior • Demonstração', versionNumber: '0.9-demo', loaders: ['fabric'], files: [{ url: '', filename: 'demo-previous.mrpack', size: 1048576, sha1: '' }] }];
            if (command === 'mods_project_details') { const item = allProjects.find(project => project.sourceId === args.projectId) || catalogs.modpack[1]; return { ...item, id: item.sourceId, body: `## ${item.title}\n\n${item.description}\n\nEsta gravação utiliza dados de demonstração.`, bodyType: 'markdown', loaders: item.categories.filter(category => ['fabric', 'forge'].includes(category)), gameVersions: ['1.21.1'], latestVersion: '1.0-demo', sourceUrl: null, author: { name: item.author || 'Demonstração', avatarUrl: textures.grass, role: 'Projeto' }, gallery: [{ url: item.bannerUrl, title: 'Galeria de demonstração' }] }; }
            if (command === 'mods_list') return mods.map((mod, index) => ({ profileId: args.profileId, projectId: `demo-mod-${index}`, versionId: 'demo-version', fileName: mod.name, sha1: '', source: 'modrinth', installedAt: timestamp }));
            if (command === 'mods_resolve_names' || command === 'mods_resolve_icons') return 0;
            if (command === 'mods_check_missing_deps') return { missing: [], allOk: true };
            if (command === 'mods_check_updates') return [];
            if (command === 'modpack_check_update') return { hasUpdate: false, currentVersion: '1.0-demo', latestVersion: '1.0-demo', changelog: 'Dados de demonstração.', source: 'modrinth', projectId: args.profileId === 'demo-vanilla' ? null : 'demo-fabulously-optimized', versionId: 'demo-version-1' };
            if (command === 'modpack_version_diff') return { available: true, added: [], removed: [], updated: [] };
            if (command === 'curseforge_status' || command === 'curseforge_validate_key') return true;
            if (command === 'instance_file_tree') {
                const folder = args.subPath || '';
                if (folder === 'mods') return args.profileId === 'demo-vanilla' ? [] : structuredClone(mods);
                if (folder === 'resourcepacks') return [{ name: 'Faithful 32x • Demonstração.zip', path: 'resourcepacks/faithful-demo.zip', isDir: false, size: 33554432, icon: null }];
                if (folder === 'shaderpacks') return [{ name: 'Complementary Reimagined • Demonstração.zip', path: 'shaderpacks/complementary-demo.zip', isDir: false, size: 8388608, icon: null }];
                if (folder === 'datapacks') return [{ name: 'Terralith • Demonstração.zip', path: 'datapacks/terralith-demo.zip', isDir: false, size: 2097152, icon: null }];
                return ['mods', 'resourcepacks', 'shaderpacks', 'saves', 'screenshots', 'config'].map(name => ({ name, path: name, isDir: true, size: 0, icon: null })).concat([{ name: 'options.txt', path: 'options.txt', isDir: false, size: 1600, icon: null }]);
            }
            if (command === 'instance_worlds_list') return [{ name: 'Mundo de demonstração', folderName: 'mundo-demo', iconBase64: textures.grass, lastPlayed: 1791201600000, gameMode: 'Survival', sizeBytes: 37748736, seed: 123456789, spawnX: 0, spawnY: 72, spawnZ: 0, versionName: '1.21.1', difficulty: 'Normal', hardcore: false, playerHealth: 20, playerLevel: 12, dayCount: 32, snapshotsCount: 2, playerInventory: [] }];
            if (command === 'instance_list_world_backups') return structuredClone(state.backups);
            if (command === 'instance_backup_world') {
                const fileName = `${args.worldFolder || 'mundo-demo'}-demonstracao-${state.backups.length + 1}.zip`;
                const entry = { fileName, filePath: `C:\\Luxmc\\Demonstracao\\backups\\${fileName}`, sizeBytes: 25165824, createdAt: '2026-10-05 12:00:00', worldName: 'Mundo de demonstração' };
                state.backups.unshift(entry); persist(); return structuredClone(entry);
            }
            if (command === 'instances_screenshots') return ['/modpack_fo.webp', '/modpack_vanilla_perfected.png', '/bg_day.jpg'].map((url, index) => ({ name: `Captura de demonstração ${index + 1}`, path: url, modified: timestamp, dataUrl: url, thumbPath: null }));
            if (command === 'instance_health_check') return { clientJar: true, natives: true, modsOk: true, issues: [] };
            if (command === 'instance_shield_scan') return { totalScanned: 8, isClean: true, threats: [], scanTimeMs: 120 };
            if (command === 'doctor_check_instance_conflicts') return conflicts;
            if (command === 'doctor_instance_readiness') return { ready: true, repairable: false, requiredJava: 21, allocatedRamMb: 4096, recommendedRamMb: 4096, gpuVendor: 'GPU de demonstração', gpuDriver: 'Driver de demonstração', blockers: [], warnings: [], conflicts };
            if (command === 'instance_options_get') return { gamma: 1, fov: 80, renderDistance: 12, simulationDistance: 10, maxFps: 144, guiScale: 0, fullscreen: false, vsync: false, autoJump: false, bobView: true, soundMaster: 0.8, soundMusic: 0.4 };
            if (command === 'instance_config_read') return { relativePath: args.relativePath || 'options.txt', content: 'gamma:1.0\nfov:0.0\nrenderDistance:12', language: 'properties' };
            if (command === 'instance_file_read') return 'Arquivo de demonstração.\n';
            if (command === 'keybinds_list') return { keybinds: [{ id: 'key_key.forward', label: 'Andar para frente', category: 'Movimento', rawKey: 'key.keyboard.w', displayKey: 'W', isConflict: false, conflictingWith: [] }], totalConflicts: 0 };
            if (command === 'java_scan') return { runtimes: [{ major: 21, installed: true, path: 'C:\\Luxmc\\Demonstracao\\java\\bin\\javaw.exe', versionString: '21.0.4 • Demonstração', isSystem: false }, { major: 17, installed: false, path: null, versionString: null, isSystem: false }, { major: 8, installed: false, path: null, versionString: null, isSystem: false }] };
            if (command === 'get_system_specs') return { osDistro: 'Windows 11', kernelVersion: 'Demonstração', arch: 'x86_64', totalRamMb: 16384, launcherVersion: '3.0.0', gpuVendor: 'GPU de demonstração', gpuRenderer: 'GPU de demonstração', gpuSupportsZink: false };
            if (command === 'env_check') return { ok: true, issues: [] };
            if (command === 'optimizer_detect_gpu') return { vendor: 'GPU de demonstração', renderer: 'GPU de demonstração', driver: 'Driver de demonstração', supportsZink: false };
            if (command === 'optimizer_get_flags') return ['-XX:+UseG1GC', '-XX:MaxGCPauseMillis=200'];
            if (command === 'optimizer_get_perf_pack') return { available: true, loader: args.loader || 'fabric', mcVersion: args.mcVersion || '1.21.1', reason: null, mods: [{ slug: 'sodium', title: 'Sodium', description: 'Renderização otimizada.' }, { slug: 'lithium', title: 'Lithium', description: 'Otimizações do jogo.' }] };
            if (command === 'jvm_args_validate') return { valid: true, rejected: [], normalized: args.args || '', suggestions: [] };
            if (command === 'storage_full_report') return storage();
            if (command === 'storage_breakdown') return storage().categories;
            if (command === 'app_data_directory') return 'C:\\Luxmc\\Demonstracao\\data';
            if (command === 'directory_exists') return true;
            if (command === 'app_update_environment') return { mode: 'windows' };
            if (command === 'changelog_get') return [{ version: '3.0.0', date: '2026-10-05', title: 'Luxmc • Demonstração', highlights: ['Instâncias organizadas', 'Personalização de skins', 'Amigos e salas'] }];
            if (command === 'minecraft_news') return newsFeed;
            if (command === 'gaming_stats_get') return null;
            if (command === 'launch_game') throw Error('Demonstração: esta gravação não inicia o Minecraft.');
            if (command === 'instance_export_share_code') return 'LUXMC-DEMO-123456';
            if (command === 'plugin:dialog|open') return 'C:/Luxmc/Demonstracao/Alex.png';
            if (command === 'plugin:dialog|message' || command === 'plugin:dialog|confirm') return false;
            if (command.endsWith('_list') || command.endsWith('_get_all') || ['deep_links_take', 'screenshots_list', 'instance_capsules_list', 'instance_benchmarks_list', 'world_backups_list', 'instance_world_snapshots', 'instance_config_files', 'optimizer_install_perf_pack'].includes(command)) return [];
            if (command.startsWith('storage_clear_')) return 0;
            return null;
        };
        window.launcherDemo = {
            label: 'DADOS DE DEMONSTRAÇÃO', calls: [], get state() { return state; }, textures,
            emit, selectNameMc: () => emit('namemc-skin-selected', 'https://namemc.com/skin/63455d7069b397c2'),
            setRoom: active => { state.room = active ? structuredClone(demoRoom) : null; persist(); emit('tunnel-state', state.room); }
        };
        window.electronAPI = { invoke, on: (event, handler) => { if (!listeners.has(event)) listeners.set(event, new Set()); listeners.get(event).add(handler); return () => listeners.get(event)?.delete(handler); }, window: { minimize: async () => {}, maximize: async () => {}, close: async () => {}, isMaximized: async () => true } };
        window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: id => nativeListeners.delete(id) };
        window.__TAURI_INTERNALS__ = {
            invoke, metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } }, convertFileSrc: file => file,
            transformCallback: (callback, once = false) => { const id = callbackId++; const handler = payload => { if (once) callbacks.delete(id); callback(payload); }; callbacks.set(id, handler); window[`_${id}`] = handler; return id; },
            unregisterCallback: id => { callbacks.delete(id); delete window[`_${id}`]; }, runCallback: (id, payload) => callbacks.get(id)?.(payload)
        };
        Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: async text => { window.launcherDemo.clipboardText = text; } } });
    }, { login, room, textures: { alex: png('alex.png'), steve: png('steve.png'), grass: png('grass_head.png') } });
    await page.route('https://**/*', async route => {
        const url = route.request().url();
        if (url.startsWith('https://launchercontent.mojang.com/')) {
            const number = /luxmc-demo-news-([1-3])/.exec(url)?.[1] || '1';
            await route.fulfill({ status: 200, contentType: 'image/jpeg', body: fs.readFileSync(path.join(__dirname, '..', 'static', `news_${number}.jpg`)) });
            return;
        }
        if (/latest\.json|api\.github\.com\/repos\/.+\/releases/.test(url)) {
            const current = { version: '3.0.0', tag_name: 'v3.0.0', notes: 'Dados de demonstração.', body: 'Dados de demonstração.', html_url: 'https://github.com/predabr/luxmc', assets: [{ name: 'Lux MC Launcher.exe', browser_download_url: 'https://example.invalid/Lux%20MC%20Launcher.exe', size: 20000000 }], platforms: { 'windows-x86_64': { url: 'https://example.invalid/Lux%20MC%20Launcher.exe' } } };
            await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(/\/releases(?:\?|$)/.test(url) ? [current] : current) });
            return;
        }
        await route.fulfill({ status: 404, contentType: 'application/json', body: '{}' });
    });
}

module.exports = { setupLauncherDemo };
