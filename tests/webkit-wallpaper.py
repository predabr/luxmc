import base64
import json
import sys
from pathlib import Path
import gi

gi.require_version('Gtk', '3.0')
gi.require_version('WebKit2', '4.1')
from gi.repository import Gtk, WebKit2, GLib

video_path = sys.argv[1]
poster_path = sys.argv[2]
poster = 'data:image/png;base64,' + base64.b64encode(Path(poster_path).read_bytes()).decode()
source = Path('tests/audit.browser.cjs').read_text()
mock = source.split('await page.addInitScript(()=>{', 1)[1].split('\n });', 1)[0]
init = '(()=>{' + mock + '})();'
init += 'localStorage.setItem("luxmc_custom_wallpaper",' + json.dumps(video_path) + ');'
init += 'localStorage.setItem("luxmc_custom_wallpaper_type","video");localStorage.setItem("luxmc_background","custom");'
init += 'const originalInvoke=window.electronAPI.invoke;const invoke=(command,args)=>command==="wallpaper_prepare_poster"?Promise.resolve(' + json.dumps(poster) + '):originalInvoke(command,args);window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;'
manager = WebKit2.UserContentManager()
manager.add_script(WebKit2.UserScript.new(init, WebKit2.UserContentInjectedFrames.TOP_FRAME, WebKit2.UserScriptInjectionTime.START, None, None))
view = WebKit2.WebView.new_with_user_content_manager(manager)
window = Gtk.Window(title='Luxmc wallpaper loop check')
window.set_default_size(1280, 800)
window.add(view)
window.connect('destroy', Gtk.main_quit)
window.show_all()
result = {'passed': False}

def evaluate(script, callback):
    def done(webview, task, _):
        try:
            callback(webview.evaluate_javascript_finish(task).to_string())
        except Exception as error:
            print('EVALUATION_ERROR', error, flush=True)
    view.evaluate_javascript(script, -1, None, None, None, done, None)

def start():
    evaluate('''(()=>{const video=document.querySelector('video');if(!video)return 'missing video';const image=video.parentElement.querySelector('img');window.loopState={seeks:0,loops:0,covered:0,uncovered:0,missingPoster:0,canvas:video.parentElement.querySelectorAll('canvas').length,errors:[]};let previous=video.currentTime;video.addEventListener('seeking',()=>{window.loopState.seeks++;setTimeout(()=>{if(video.style.opacity==='0'&&image?.complete&&image.naturalWidth>0)window.loopState.covered++;else window.loopState.uncovered++},0)});video.addEventListener('timeupdate',()=>{if(video.currentTime+0.5<previous)window.loopState.loops++;previous=video.currentTime});video.addEventListener('error',()=>window.loopState.errors.push(video.error?.message||'video error'));setInterval(()=>{if(video.style.opacity==='0'&&!(image?.complete&&image.naturalWidth>0))window.loopState.missingPoster++},40);return 'started'})()''', lambda value: print(value, flush=True))
    return False

def poll():
    def received(raw):
        global result
        try:
            state=json.loads(raw)
            if state and state.get('loops',0)>=3:
                result={'passed':state['canvas']==0 and state['covered']>=3 and state['uncovered']==0 and state['missingPoster']==0 and not state['errors'],**state}
                print('WEBKIT_WALLPAPER_RESULT',json.dumps(result),flush=True)
                window.destroy()
        except Exception as error:
            print('POLL_ERROR',error,flush=True)
    evaluate('JSON.stringify(window.loopState||null)',received)
    return True

def timeout():
    evaluate('JSON.stringify(window.loopState||null)',lambda raw:print('WEBKIT_WALLPAPER_TIMEOUT',raw,flush=True))
    GLib.timeout_add_seconds(2,lambda:window.destroy())
    return False

view.load_uri('http://127.0.0.1:1420/')
GLib.timeout_add_seconds(6,start)
GLib.timeout_add_seconds(2,poll)
GLib.timeout_add_seconds(60,timeout)
Gtk.main()
sys.exit(0 if result['passed'] else 1)
