import { afterEach, beforeEach, expect, it, vi } from 'vitest';
const mocks = vi.hoisted(() => ({ tree: vi.fn(), icons: vi.fn(), game: {isLaunching:false,isGameRunning:false}, settings:{value:{preloadContentIcons:true}} }));
vi.mock('$lib/api/instances', () => ({instanceFileTree:mocks.tree,instanceContentIcons:mocks.icons}));
vi.mock('$lib/stores/app.svelte', () => ({appState:mocks.game}));
vi.mock('$lib/stores/settings.svelte', () => ({settings:mocks.settings}));
beforeEach(() => {
    vi.resetModules(); vi.clearAllMocks(); vi.useFakeTimers();
    vi.spyOn(document,'hidden','get').mockReturnValue(false);
    mocks.game.isLaunching = false; mocks.game.isGameRunning = false;
});
afterEach(() => { vi.clearAllTimers(); vi.useRealTimers(); vi.unstubAllGlobals(); vi.restoreAllMocks(); });

it('warms every local content section before waiting for provider icons and retains newer file-tree icons', async () => {
    const calls: string[] = [];
    mocks.tree.mockImplementation(async (_id:string,folder:string) => [{name:folder+(folder === 'mods' ? '.jar' : '.zip'),isDir:false,iconKey:folder,icon:null,iconResolved:false},{name:'memory_repo',isDir:true,iconKey:folder+':internal',icon:null,iconResolved:false}].filter(entry => !entry.isDir || folder === 'mods'));
    mocks.icons.mockImplementation(async (_id:string,folder:string,names:string[],lookup:boolean) => {
        calls.push(`${folder}:${Boolean(lookup)}`);
        expect(names).not.toContain('memory_repo');
        return names.map(name => ({name,iconKey:folder,icon:lookup?'https://example.test/'+folder+'.png':null,resolved:Boolean(lookup)}));
    });
    vi.stubGlobal('Image',class { decoding=''; src=''; onload=null; onerror=null; });
    const {prefetchInstanceContent,hydrateContentIcons} = await import('./contentPrefetch');
    await prefetchInstanceContent('instance');
    expect(calls.slice(0,3)).toEqual(['mods:false','resourcepacks:false','shaderpacks:false']);
    expect(calls.filter(call=>call.endsWith(':true'))).toHaveLength(3);
    const entries = [{name:'mods.jar',path:'/mods.jar',isDir:false,size:1,iconKey:'mods',icon:null,iconResolved:false}];
    expect(hydrateContentIcons(entries)[0].icon).toBe('https://example.test/mods.png');
    expect(hydrateContentIcons([{...entries[0],icon:'https://example.test/new.png'}])[0].icon).toBe('https://example.test/new.png');
});

it('bounds image downloads and suspends queued work while Minecraft runs', async () => {
    const requested: Array<{onload:()=>void;onerror:()=>void;src:string}> = [];
    vi.stubGlobal('Image',class { decoding=''; onload=()=>{}; onerror=()=>{}; set src(value:string) { if(value) requested.push(this as never); } });
    const {warmContentImages} = await import('./contentPrefetch');
    warmContentImages(Array.from({length:500},(_,index)=>({icon:`https://example.test/${index}.png`})));
    expect(requested).toHaveLength(4);
    mocks.game.isGameRunning=true;
    requested.slice().forEach(image=>image.onload());
    expect(requested).toHaveLength(4);
    mocks.game.isGameRunning=false;
    warmContentImages([]);
    expect(requested).toHaveLength(8);
});

it('notifies open lists when a missing image arrives and retains it after a stale negative response', async () => {
    const { rememberContentIcons, hydrateContentIcons, subscribeContentIcons } = await import('./contentPrefetch');
    const received = vi.fn();
    const unsubscribe = subscribeContentIcons(received);
    rememberContentIcons([{ iconKey: 'first', icon: null, resolved: true }]);
    expect(received).not.toHaveBeenCalled();
    rememberContentIcons([{ iconKey: 'first', icon: '/grass_head.png', resolved: true }]);
    expect(received).toHaveBeenCalledTimes(1);
    rememberContentIcons([{ iconKey: 'first', icon: null, resolved: true }]);
    expect(hydrateContentIcons([{ name: 'First.zip', path: '/First.zip', size: 1, isDir: false, icon: null, iconKey: 'first' }])[0].icon).toBe('/grass_head.png');
    unsubscribe();
    rememberContentIcons([{ iconKey: 'second', icon: '/grass_head.png', resolved: true }]);
    expect(received).toHaveBeenCalledTimes(1);
});
