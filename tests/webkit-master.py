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
init = '(()=>{' + mock + '})();window.testSkinData=' + json.dumps(skin) + ';'
if os.environ.get('LUXMC_WEBKIT_FORCE_RAF') == '1':
    init += 'HTMLVideoElement.prototype.requestVideoFrameCallback=undefined;'
init += 'localStorage.setItem("luxmc_saved_skins",JSON.stringify([{id:"custom-test",name:"Personalizada",model:"steve",url:window.testSkinData}]));'
init += "for(const method of ['drawImage','getImageData']){const original=CanvasRenderingContext2D.prototype[method];CanvasRenderingContext2D.prototype[method]=function(...args){try{const result=original.apply(this,args);if(method==='drawImage' && args[0] instanceof HTMLVideoElement && this.canvas.parentElement===args[0].parentElement){window.wallpaperDraws=(window.wallpaperDraws||0)+1}return result}catch(error){const key=method+':'+error.name+':'+error.message;window.canvasFailures??={};window.canvasFailures[key]=(window.canvasFailures[key]||0)+1;throw error}};}"

feed = Path('src/lib/data/minecraft-news.json').read_text()
init += 'window.newsFixture=' + feed + ';'
init += """
localStorage.removeItem('luxmc_minecraft_news_v2');
localStorage.setItem('luxmc_saved_skins',JSON.stringify([{id:'default_steve',name:'Steve',model:'steve',url:'/steve.png'},{id:'custom-test',name:'Personalizada',model:'steve',url:window.testSkinData}]));
window.browserErrors=[];window.addEventListener('error',e=>window.browserErrors.push(String(e.error||e.message)));window.addEventListener('unhandledrejection',e=>window.browserErrors.push(String(e.reason)));
const originalInvoke=window.electronAPI.invoke;
const invoke=async(command,args)=>{
    let result;
    if(command==='plugin:dialog|open') result=window.uploadPath;
    else if(command==='auth_read_local_texture') result=window.uploadTexture||window.testSkinData;
    else if(command==='minecraft_news') {if(window.failNews) throw new Error('offline');result=window.newsFixture;}
    else if(['instance_file_tree','instance_worlds_list','instances_screenshots'].includes(command)) result=[];
    else if(command==='java_scan') result={runtimes:[]};
    else if(command==='versions_check_installed') result=true;
    else if(command==='mods_search') result=Array.from({length:200},(_,i)=>({source:'modrinth',sourceId:String(i),slug:'pack-'+i,title:'Pack '+i,description:'Descrição de teste',author:'Fixture',downloads:100,iconUrl:'/grass_head.png',categories:['fabric'],versions:['1.20.1'],bannerUrl:null,gameVersions:['1.20.1'],loaders:['fabric']}));
    else return originalInvoke(command,args);
    window.testCalls.push({command,args}); return result;
};
window.electronAPI.invoke=invoke; window.__TAURI_INTERNALS__.invoke=invoke;
window.__TAURI_INTERNALS__.convertFileSrc=()=>{throw new Error('Local textures must use native PNG bytes');};
"""
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


def start():
    evaluate(Path('tests/master.browser.cjs').read_text() + ";'started';")
    return False

def poll():
    def done(webview, task, _):
        global passed
        try:
            raw = webview.evaluate_javascript_finish(task).to_string()
            if raw != 'null':
                result = json.loads(raw)
                passed = result['passed']
                print('MASTER_RESULT', json.dumps(result, ensure_ascii=False), flush=True)
                GLib.idle_add(window.destroy)
        except Exception as error: print(error, flush=True)
    view.evaluate_javascript('JSON.stringify(window.masterResult || null)', -1, None, None, None, done, None)
    return True

def timeout():
    evaluate('JSON.stringify({progress:window.masterProgress, errors:window.browserErrors, body:document.body.innerText.slice(-2500)})')
    GLib.timeout_add_seconds(2, lambda: window.destroy())
    print('MASTER_RESULT timeout', flush=True)
    return False

view.load_uri('http://127.0.0.1:1420/')
GLib.timeout_add_seconds(8,start)
GLib.timeout_add_seconds(2,poll)
GLib.timeout_add_seconds(110,timeout)
Gtk.main()
sys.exit(0 if passed else 1)
