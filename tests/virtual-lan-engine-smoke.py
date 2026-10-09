import ctypes
import json
import os
import pathlib
import secrets
import shutil
import socket
import subprocess
import time

root = pathlib.Path(os.environ["TEMP"]) / "luxmc-lan-loader-validation-v2.6.4"
ctypes.windll.kernel32.SetErrorMode(0x8003)
assert set(p.name for p in root.iterdir()) == {"easytier-core.exe", "easytier-cli.exe", "wintun.dll", "Packet.dll"}
missing = pathlib.Path(os.environ["TEMP"]) / "luxmc-lan-loader-missing-packet-validation"
missing.mkdir(exist_ok=True)
assert not (missing / "Packet.dll").exists()
shutil.copyfile(root / "easytier-core.exe", missing / "easytier-core.exe")
broken = subprocess.run([str(missing / "easytier-core.exe"), "--version"], cwd=missing, capture_output=True, timeout=8, creationflags=0x08000000)
assert broken.returncode & 0xFFFFFFFF == 0xC0000135
print(json.dumps({"missingLibraryDetected": True, "windowsLoaderCode": "0xC0000135", "systemDialogSuppressed": True}))
with socket.socket() as reservation:
    reservation.bind(("127.0.0.1", 0))
    port = reservation.getsockname()[1]
environment = os.environ.copy()
environment["ET_NETWORK_NAME"] = "luxmc-smoke-" + secrets.token_hex(16)
environment["ET_NETWORK_SECRET"] = secrets.token_hex(32)
command = [str(root / "easytier-core.exe"), "--no-tun", "--ipv4", "10.100.1.1/24", "--hostname", "Luxmc smoke", "--dev-name", "Luxmc LAN", "--peers", "tcp://dreamlife.indevs.in:11010", "--no-listener", "--rpc-portal", f"127.0.0.1:{port}", "--rpc-portal-whitelist", "127.0.0.1", "--relay-network-whitelist", "--console-log-level", "off"]
process = subprocess.Popen(command, cwd=root, env=environment, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, creationflags=0x08000000)
try:
    for attempt in range(20):
        if process.poll() is not None:
            raise RuntimeError("Core exited: " + str(process.returncode) + " " + process.stderr.read().decode(errors="replace")[:1000])
        result = subprocess.run([str(root / "easytier-cli.exe"), "--rpc-portal", f"127.0.0.1:{port}", "--output", "json", "peer"], cwd=root, capture_output=True, timeout=5, creationflags=0x08000000)
        if result.returncode == 0:
            data = json.loads(result.stdout)
            node = subprocess.run([str(root / "easytier-cli.exe"), "--rpc-portal", f"127.0.0.1:{port}", "--output", "json", "node", "info"], cwd=root, capture_output=True, timeout=5, creationflags=0x08000000)
            assert node.returncode == 0
            info = json.loads(node.stdout)
            assert environment["ET_NETWORK_SECRET"] in str(info.get("config", "")), "Private network secret was not applied"
            print(json.dumps({"coreStarted": True, "rpcResponded": True, "adapterCreated": False, "peerType": type(data).__name__, "peerFields": list(data[0]) if isinstance(data, list) and data else [], "nodeFields": list(info), "nodeAddress": info.get("ipv4_addr")}))
            pathlib.Path("docs/validation/lan-cli-node-fixture.json").write_text(json.dumps({"ipv4_addr": info.get("ipv4_addr")}), encoding="utf-8")
            pathlib.Path("docs/validation/lan-cli-peers-fixture.json").write_text(json.dumps([{key: item.get(key) for key in ["ipv4", "hostname"]} for item in data]), encoding="utf-8")
            break
        time.sleep(0.5)
    else:
        raise RuntimeError("RPC did not respond")
finally:
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)
