import argparse
import json
import os
from pathlib import Path
import statistics
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser()
parser.add_argument("--data-dir", type=Path, required=True)
parser.add_argument("--version", required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
manifest = json.loads((args.data_dir / "versions" / args.version / f"{args.version}.json").read_text())
major = manifest.get("javaVersion", {}).get("majorVersion", 17)
suffix = ".exe" if os.name == "nt" else ""
java = args.data_dir / "java" / str(major) / "bin" / f"java{suffix}"
compiler = args.data_dir / "java" / "17" / "bin" / f"javac{suffix}"
libraries = []
for library in manifest["libraries"]:
    path = library.get("downloads", {}).get("artifact", {}).get("path")
    if not path:
        parts = library["name"].split(":")
        group, artifact, version = parts[:3]
        classifier = "-" + parts[3] if len(parts) > 3 else ""
        path = group.replace(".", "/") + "/" + artifact + "/" + version + "/" + artifact + "-" + version + classifier + ".jar"
    if path and (args.data_dir / "libraries" / path).is_file():
        libraries.append(str(args.data_dir / "libraries" / path))
classpath = os.pathsep.join(libraries)
with tempfile.TemporaryDirectory(prefix="luxmc-hardware-startup-") as directory:
    subprocess.run([str(compiler), "--release", "8", "-cp", classpath, "-d", directory, str(root / "tests/java/StartupHardwareProbe.java")], check=True)
    results = {}
    for name, flags in [("default", []), ("counterFallback", ["-Doshi.os.windows.hkeyperfdata=false"])]:
        samples = []
        for sample in range(3):
            started = time.perf_counter()
            completed = subprocess.run([str(java), "-Djava.awt.headless=true", *flags, "-cp", directory + os.pathsep + classpath, "StartupHardwareProbe"], capture_output=True, text=True, timeout=45)
            if completed.returncode != 0:
                raise RuntimeError(f"Hardware probe failed ({name}), exit {completed.returncode}")
            metrics = json.loads(completed.stdout.strip().splitlines()[-1])
            metrics["totalSeconds"] = time.perf_counter() - started
            samples.append(metrics)
        results[name] = {"medianTotalSeconds": statistics.median(item["totalSeconds"] for item in samples), "medianHardwareSeconds": statistics.median(item["hardwareSeconds"] for item in samples), "samples": samples}
    print(json.dumps({"version": args.version, "javaMajor": major, "results": results}, indent=2))
