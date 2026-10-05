import json
import os
import sys
from pathlib import Path
import gi

gi.require_version('Gtk', '3.0')
gi.require_version('WebKit2', '4.1')
from gi.repository import Gtk, WebKit2, GLib

source = Path('tests/mods-catalog.browser.cjs').read_text()
mock = source.split('await page.addInitScript(() => {', 1)[1].split('\n    });', 1)[0]
manager = WebKit2.UserContentManager()
manager.add_script(WebKit2.UserScript.new('(()=>{' + mock + '})();', WebKit2.UserContentInjectedFrames.TOP_FRAME, WebKit2.UserScriptInjectionTime.START, None, None))
view = WebKit2.WebView(web_context=WebKit2.WebContext.new_ephemeral(), user_content_manager=manager)
window = Gtk.Window(title='Luxmc — teste do catálogo WebKit')
window.set_default_size(1180, 900)
window.add(view)
destroy_handler = window.connect('destroy', Gtk.main_quit)
window.show_all()
passed = False

script = """(() => {
    const list = document.querySelector('[data-catalog-results]');
    if (!list || !list.querySelector('.catalog-card')) return JSON.stringify({ready:false});
    if (window.scrollReport) return JSON.stringify(window.scrollReport);
    if (!window.measuringScroll) {
        window.measuringScroll = true;
        (async () => {
            await new Promise(resolve => setTimeout(resolve, 2000));
            const root = document.querySelector('main');
            const frames = []; let previous = performance.now(); let maxCards = 0;
            for (let index = 0; index < 120; index++) {
                await new Promise(requestAnimationFrame);
                const now = performance.now(); frames.push(now-previous); previous=now;
                root.scrollTop += 60;
                maxCards = Math.max(maxCards, list.querySelectorAll('.catalog-card').length);
            }
            root.scrollTop = root.scrollHeight;
            await new Promise(requestAnimationFrame); await new Promise(requestAnimationFrame);
            frames.sort((a,b)=>a-b);
            window.scrollReport = {ready:true, count:list.querySelectorAll('.catalog-card').length, maxCards, lastMod:list.textContent.includes('Pack 35'), frameP95:frames[113], horizontalOverflow:root.scrollWidth > root.clientWidth};
        })();
    }
    return JSON.stringify({ready:false});
})();"""

def done(webview, task, _):
    global passed
    try:
        report = json.loads(webview.evaluate_javascript_finish(task).to_string())
        if report.get('ready'):
            print(json.dumps(report), flush=True)
            passed = report['lastMod'] and report['maxCards'] <= 20 and report['frameP95'] < 150 and not report['horizontalOverflow']
            GLib.idle_add(lambda: Gtk.main_quit() or False)
    except Exception as error:
        print(str(error), flush=True)
        GLib.idle_add(lambda: Gtk.main_quit() or False)

def poll():
    view.evaluate_javascript(script, -1, None, None, None, done, None)
    return True

GLib.timeout_add(1000, poll)
GLib.timeout_add_seconds(45, lambda: window.destroy() or False)
view.load_uri(os.environ.get('LUXMC_BASE_URL', 'http://127.0.0.1:1420') + '/mods')
Gtk.main()
window.disconnect(destroy_handler)
window.destroy()
sys.exit(0 if passed else 1)
