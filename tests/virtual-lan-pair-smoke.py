import ctypes
import json
import os
import pathlib
import secrets
import socket
import subprocess
import time

ctypes.windll.kernel32.SetErrorMode(0x8003)
root = pathlib.Path(os.environ["TEMP"]) / "luxmc-lan-loader-validation-v2.6.4"
environment = os.environ.copy()
environment["ET_NETWORK_NAME"] = "luxmc-pair-" + secrets.token_hex(16)
environment["ET_NETWORK_SECRET"] = secrets.token_hex(32)
children = []
ports = []
try:
    for index in range(2):
        with socket.socket() as reservation:
            reservation.bind(("127.0.0.1", 0))
            port = reservation.getsockname()[1]
        ports.append(port)
        args = [str(root / "easytier-core.exe"), "--no-tun", "--ipv4", f"10.100.1.{index + 1}/24", "--hostname", f"Luxmc-pair-{index}", "--peers", "tcp://dreamlife.indevs.in:11010", "--no-listener", "--rpc-portal", f"127.0.0.1:{port}", "--rpc-portal-whitelist", "127.0.0.1", "--relay-network-whitelist", "--console-log-level", "off"]
        children.append(subprocess.Popen(args, cwd=root, env=environment, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, creationflags=0x08000000))
    for attempt in range(24):
        result = subprocess.run([str(root / "easytier-cli.exe"), "--rpc-portal", f"127.0.0.1:{ports[0]}", "--output", "json", "peer"], cwd=root, capture_output=True, timeout=4, creationflags=0x08000000)
        if result.returncode == 0:
            peers = json.loads(result.stdout)
            if any(peer.get("hostname") == "Luxmc-pair-1" for peer in peers):
                print(json.dumps({"twoNodesDiscoveredThroughRelay": True, "adapterCreated": False}))
                break
        time.sleep(1)
    else:
        raise RuntimeError("The configured public relay did not connect two nodes in 24 seconds")
finally:
    for child in children:
        child.terminate()
        child.wait(timeout=5)
