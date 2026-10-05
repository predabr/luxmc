import json
import os
import subprocess
import tempfile
import time
from pathlib import Path

binary = Path(os.environ.get('LUXMC_NATIVE_BINARY', 'src-tauri/target/release/luxmc')).resolve()
with tempfile.TemporaryDirectory(prefix='luxmc-close-') as directory:
    root = Path(directory)
    environment = os.environ.copy()
    for name, folder in [('XDG_DATA_HOME', 'data'), ('XDG_CONFIG_HOME', 'config'), ('XDG_CACHE_HOME', 'cache')]:
        environment[name] = str(root / folder)
    with (root / 'launcher.log').open('w') as log:
        process = subprocess.Popen([str(binary)], env=environment, stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 25
            visible = False
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f'Launcher encerrou antes da janela: {process.returncode}')
                clients = json.loads(subprocess.check_output(['hyprctl', '-j', 'clients'], text=True))
                if any(client.get('pid') == process.pid for client in clients):
                    visible = True
                    break
                time.sleep(0.1)
            if not visible:
                raise RuntimeError('A janela de teste não apareceu')
            time.sleep(3)
            started = time.monotonic()
            subprocess.run(['hyprctl', 'eval', f'hl.dispatch(hl.dsp.window.close({{ window = "pid:{process.pid}" }}))'], check=True, capture_output=True)
            code = process.wait(timeout=10)
            elapsed = (time.monotonic() - started) * 1000
            if code != 0:
                raise RuntimeError(f'Launcher encerrou com código {code}')
            print(json.dumps({'nativeCloseMs': round(elapsed, 1), 'exitCode': code, 'isolatedData': True, 'windowAppeared': visible}), flush=True)
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
