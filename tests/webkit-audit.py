import base64
import json
import os
import re
import sys
import time
from pathlib import Path
import gi

gi.require_version('Gtk', '3.0')
gi.require_version('WebKit2', '4.1')
from gi.repository import Gtk, WebKit2, GLib

fixture = Path(sys.argv[1] if len(sys.argv) > 1 else 'static/alex.png')
duration = int(os.environ.get('LUXMC_WEBKIT_SECONDS', '75'))
wallpaper = sys.argv[2] if len(sys.argv) > 2 else 'http://127.0.0.1:1420/tests/fixtures/wallpaper.mp4'
source = Path('tests/audit.browser.cjs').read_text()
mock = source.split('await page.addInitScript(()=>{', 1)[1].split('\n });', 1)[0]
skin = 'data:image/png;base64,' + base64.b64encode(fixture.read_bytes()).decode()
init = '(()=>{' + mock + '})();window.testSkinData=' + json.dumps(skin) + ';window.testWallpaperSource=' + json.dumps(wallpaper) + ';'
init += 'localStorage.setItem("luxmc_custom_wallpaper",'+json.dumps(wallpaper)+');localStorage.setItem("luxmc_custom_wallpaper_type","video");localStorage.setItem("luxmc_background","custom");'
if os.environ.get('LUXMC_WEBKIT_ASSET_URL'):
    init += 'window.__TAURI_INTERNALS__.convertFileSrc=()=>'+json.dumps(os.environ['LUXMC_WEBKIT_ASSET_URL'])+';'
if os.environ.get('LUXMC_WEBKIT_FORCE_RAF') == '1':
    init += 'HTMLVideoElement.prototype.requestVideoFrameCallback=undefined;'
init += 'localStorage.setItem("luxmc_saved_skins",JSON.stringify([{id:"custom-test",name:"Personalizada",model:"steve",url:window.testSkinData}]));'
init += "for(const method of ['drawImage','getImageData']){const original=CanvasRenderingContext2D.prototype[method];CanvasRenderingContext2D.prototype[method]=function(...args){try{const result=original.apply(this,args);if(method==='drawImage' && args[0] instanceof HTMLVideoElement && this.canvas.parentElement===args[0].parentElement){window.wallpaperDraws=(window.wallpaperDraws||0)+1}return result}catch(error){const key=method+':'+error.name+':'+error.message;window.canvasFailures??={};window.canvasFailures[key]=(window.canvasFailures[key]||0)+1;throw error}};}"
manager = WebKit2.UserContentManager()
manager.add_script(WebKit2.UserScript.new(init, WebKit2.UserContentInjectedFrames.TOP_FRAME, WebKit2.UserScriptInjectionTime.START, None, None))
view = WebKit2.WebView.new_with_user_content_manager(manager)
view.get_settings().set_enable_webgl(True)
window = Gtk.Window(title='Luxmc — verificação WebKitGTK')
window.set_default_size(1280, 800)
window.add(view)
window.connect('destroy', Gtk.main_quit)
window.show_all()
results = []
passed = False

def evaluate(script):
    def done(webview, task, _):
        try:
            value = webview.evaluate_javascript_finish(task).to_string()
            print(value, flush=True)
            results.append(value)
        except Exception as error:
            print('EVALUATION ERROR', error, flush=True)
    view.evaluate_javascript(script, -1, None, None, None, done, None)

def cpu_snapshot():
    total = 0
    parents = {os.getpid()}
    records = []
    for path in Path('/proc').glob('[0-9]*/stat'):
        try:
            tail = path.read_text().rsplit(')', 1)[1].split()
            records.append((int(path.parent.name), int(tail[1]), int(tail[11])+int(tail[12])))
        except (OSError, ValueError): pass
    for _ in range(5):
        parents.update(pid for pid, parent, _ in records if parent in parents)
    return sum(ticks for pid, _, ticks in records if pid in parents)

cpu_start = None
def begin_cpu():
    global cpu_start
    cpu_start = (time.monotonic(), cpu_snapshot())
    evaluate("window.frameGaps=[];let previous=performance.now();function measure(now){window.frameGaps.push(now-previous);previous=now;requestAnimationFrame(measure)};requestAnimationFrame(measure);'measure';")
    return False

def end_cpu():
    elapsed=time.monotonic()-cpu_start[0]
    print('CPU_RESULT', json.dumps({'percentOneCore':(cpu_snapshot()-cpu_start[1])/os.sysconf('SC_CLK_TCK')/elapsed*100,'seconds':elapsed}),flush=True)
    evaluate("JSON.stringify((()=>{const a=window.frameGaps.sort((a,b)=>a-b);return {frameP50:a[Math.floor(a.length*.5)],frameP95:a[Math.floor(a.length*.95)],over50ms:a.filter(x=>x>50).length,frames:a.length}})())")
    return False

if os.environ.get('LUXMC_WEBKIT_MEASURE') == '1':
    GLib.timeout_add_seconds(20,begin_cpu)
    GLib.timeout_add_seconds(70,end_cpu)

def start():
    evaluate('''(async()=>{
        const {themeStore}=await import('/src/lib/stores/theme.svelte.ts');
        themeStore.setCustomWallpaper(''' + json.dumps(wallpaper) + ''','video');
        const {goto}=await import('/node_modules/@sveltejs/kit/src/runtime/app/navigation.js');
        await goto('/skins');
        setTimeout(()=>{
            [...document.querySelectorAll('[role="button"]')].find(e=>e.textContent.includes('Personalizada'))?.click();
            const video=document.querySelector('video');window.loops=0;let previous=video?.currentTime||0;video?.addEventListener('timeupdate',()=>{if(video.currentTime+0.5<previous)window.loops++;previous=video.currentTime;});
        },2500);
    })();'started';''')
    return False

def sample():
    evaluate('''JSON.stringify((()=>{
        const c=document.querySelector('[aria-label="Visualizador 3D de Skin"] canvas');
        const v=document.querySelector('video');const frame=v?.parentElement.querySelector('canvas');
        let skinPixels=0;
        if(c){const copy=document.createElement('canvas');copy.width=c.width;copy.height=c.height;const ctx=copy.getContext('2d');ctx.drawImage(c,0,0);const data=ctx.getImageData(0,0,copy.width,copy.height).data;for(let i=3;i<data.length;i+=4)if(data[i]>0)skinPixels++;}
        let videoSignature="";if(frame){const encoded=frame.toDataURL();let hash=2166136261;for(let i=0;i<encoded.length;i++)hash=Math.imul(hash^encoded.charCodeAt(i),16777619);videoSignature=(hash>>>0).toString(16);}
        let videoPixel=frame?[...frame.getContext('2d').getImageData(frame.width/2,frame.height/2,1,1).data]:[];
        let noiseFraction=0;
        if(frame && frame.width>1){const row=frame.getContext('2d').getImageData(0,Math.floor(frame.height/2),frame.width,1).data;let noisy=0;for(let i=4;i<row.length;i+=4){if(Math.abs(row[i]-row[i-4])+Math.abs(row[i+1]-row[i-3])+Math.abs(row[i+2]-row[i-2])>180)noisy++;}noiseFraction=noisy/(frame.width-1);}
        return {wallpaperDraws:window.wallpaperDraws,sampledAt:performance.now(),canvasFailures:window.canvasFailures,skinPixels,videoPixel,videoSignature,noiseFraction,loops:window.loops,videoTime:v?.currentTime,videoSource:v?.currentSrc,videoPaused:v?.paused,videoReady:v?.readyState,hidden:document.hidden,videoError:v?.error?.message,recovered:window.recoveredUnexpectedPause,gamePause:window.gamePausePreserved,gameResume:window.gameResumeRecovered,previewCached:window.previewCached,previewDetails:window.previewDetails,saves:window.testCalls.filter(c=>c.command==='auth_save_appearance').length};
    })())''')
    return True

def finish():
    global passed
    samples=[]
    for item in results:
        try: samples.append(json.loads(item))
        except Exception: pass
    valid=[item for item in samples if isinstance(item,dict) and item.get('skinPixels',0)>0]
    success=bool(valid) and valid[-1].get('loops',0)>=3 and all(item['saves']==0 for item in valid) and all(sum(item['videoPixel'][:3])>0 for item in valid)
    passed=success and len({item.get('videoSignature') for item in valid})>=3 and all(item.get('noiseFraction',1)<0.1 for item in valid) and any(item.get('recovered') for item in valid) and len({item.get('videoSignature') for item in valid[-6:]})>=3
    moving=[(b['wallpaperDraws']-a['wallpaperDraws'])/((b['sampledAt']-a['sampledAt'])/1000) for a,b in zip(valid,valid[1:]) if a.get('wallpaperDraws') is not None and b.get('wallpaperDraws') is not None]
    passed = passed and bool(moving) and sum(moving)/len(moving)>10
    if duration >= 150:
        passed = passed and any(item.get('gamePause') for item in valid) and any(item.get('gameResume') for item in valid) and any(item.get('previewCached') for item in valid)
    print('WEBKIT_RESULT', json.dumps({'passed':passed,'averageWallpaperFps':sum(moving)/len(moving) if moving else 0,'samples':len(valid),'distinctFrames':len({item.get('videoSignature') for item in valid})}), flush=True)
    window.destroy()
    return False

view.load_uri('http://127.0.0.1:1420/')
GLib.timeout_add_seconds(8,start)
GLib.timeout_add_seconds(5,sample)
def unexpected_pause():
    evaluate("(()=>{const v=document.querySelector('video');v.pause();const before=v.currentTime;setTimeout(()=>{window.recoveredUnexpectedPause=!v.paused && v.currentTime!==before},3500)})();'pause injected';")
    return False

GLib.timeout_add_seconds(35,unexpected_pause)
def game_pause():
    evaluate("""(async()=>{
        const {appState}=await import('/src/lib/stores/app.svelte.ts');
        appState.isGameRunning=true;
        setTimeout(()=>{
            const video=document.querySelector('video');
            const canvas=video.parentElement.querySelector('canvas');
            const time=video.currentTime;const frame=canvas.toDataURL();
            setTimeout(()=>{
                window.gamePausePreserved=video.paused && video.currentTime===time && canvas.toDataURL()===frame;
                appState.isGameRunning=false;
                setTimeout(()=>{window.gameResumeRecovered=!video.paused && video.currentTime!==time && canvas.toDataURL()!==frame},2500);
            },2500);
        },500);
    })();'game pause';""")
    return False

def preview_check():
    evaluate("""(async()=>{
        const {mount,unmount}=await import('/node_modules/svelte/src/index-client.js');
        const {default:VideoWallpaper}=await import('/src/lib/components/visuals/VideoWallpaper.svelte');
        const source=window.testWallpaperSource;
        const host=document.createElement('div');host.style.cssText='position:fixed;right:20px;bottom:20px;width:240px;height:135px;z-index:9999';document.body.append(host);
        const instance=mount(VideoWallpaper,{target:host,props:{src:source,preview:true}});
        setTimeout(()=>{
            const canvas=host.querySelector('canvas');const video=host.querySelector('video');
            window.previewDetails={source,width:canvas?.width,paused:video?.paused,src:video?.getAttribute("src")};
            window.previewCached=Boolean(canvas?.width===480 && video?.paused && !video?.getAttribute('src'));
            unmount(instance);host.remove();
        },1500);
    })();'preview test';""")
    return False

if duration >= 150:
    GLib.timeout_add_seconds(85,game_pause)
    GLib.timeout_add_seconds(110,preview_check)
GLib.timeout_add_seconds(duration,finish)
Gtk.main()

sys.exit(0 if passed else 1)
