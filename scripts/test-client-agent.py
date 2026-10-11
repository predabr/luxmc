import argparse
import os
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument("--data-dir", type=Path, required=True)
parser.add_argument("--skin", type=Path)
parser.add_argument("--available", action="store_true")
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
skin_file = args.skin.resolve() if args.skin else root / "static/steve.png"
libraries = args.data_dir / "libraries"
java = args.data_dir / "java"
dependencies = [
    "com/google/code/gson/gson/2.10.1/gson-2.10.1.jar",
    "com/google/guava/guava/31.1-jre/guava-31.1-jre.jar",
    "com/google/guava/failureaccess/1.0.1/failureaccess-1.0.1.jar",
    "org/apache/commons/commons-lang3/3.17.0/commons-lang3-3.17.0.jar",
    "org/apache/logging/log4j/log4j-api/2.17.0/log4j-api-2.17.0.jar",
    "org/apache/logging/log4j/log4j-core/2.17.0/log4j-core-2.17.0.jar",
    "org/slf4j/slf4j-api/2.0.17/slf4j-api-2.0.17.jar",
]
for index, dependency in enumerate(dependencies):
    if not (libraries / dependency).is_file():
        if args.available:
            folder = (libraries / dependency).parent.parent
            artifact = folder.name
            candidates = sorted(folder.glob(f"*/{artifact}-*.jar"), reverse=True)
            if candidates:
                dependencies[index] = str(candidates[0].relative_to(libraries))
                continue
        raise FileNotFoundError(libraries / dependency)
matrix = [("1.5.25", 8), ("3.11.49", 17), ("4.0.43", 17),
          ("6.0.54", 21), ("7.0.63", 21), ("9.0.75", 25), ("10.0.77", 25)]
if args.available:
    matrix = []
    for archive in sorted((libraries / "com/mojang/authlib").glob("*/authlib-*.jar")):
        version = archive.parent.name
        major = int(version.split('.')[0])
        matrix.append((version, 17 if major < 6 else 21 if major < 9 else 25))
    if not matrix:
        raise FileNotFoundError("No installed Authlib test libraries")
with tempfile.TemporaryDirectory(prefix="luxmc-appearance-test-") as output:
    subprocess.run([str(java / "17/bin/javac"), "--release", "8", "-d", output,
                    str(root / "tests/java/AppearanceProbe.java"), str(root / "tests/java/SharedAppearanceProbe.java")], check=True)
    for version, runtime in matrix:
        authlib = libraries / f"com/mojang/authlib/{version}/authlib-{version}.jar"
        if not authlib.is_file():
            raise FileNotFoundError(authlib)
        classpath = os.pathsep.join([output, str(authlib)] + [str(libraries / item) for item in dependencies])
        for model, cape in [("slim", True), ("default", False)]:
            print(f"Authlib {version}, Java {runtime}, model {model}, cape {cape}", flush=True)
            command = [str(java / str(runtime) / "bin/java"), "-Djava.awt.headless=true",
                       "-Dluxmc.appearance.uuid=01234567-89ab-cdef-0123-456789abcdef",
                       f"-Dluxmc.appearance.skin={skin_file}",
                       f"-Dluxmc.appearance.model={model}"]
            if cape:
                command.append(f"-Dluxmc.appearance.cape={root / 'static/alex.png'}")
            command += [f"-javaagent:{root / 'src-tauri/assets/luxmc-client-agent.jar'}=appearance-only",
                        "-cp", classpath, "AppearanceProbe"]
            subprocess.run(command, check=True, timeout=30)
        print(f"Shared and room appearance: Authlib {version}", flush=True)
        subprocess.run([str(java / str(runtime) / "bin/java"), "-Djava.awt.headless=true",
                        "-Dluxmc.appearance.uuid=01234567-89ab-cdef-0123-456789abcdef",
                        f"-javaagent:{root / 'src-tauri/assets/luxmc-client-agent.jar'}=appearance-only",
                        "-cp", classpath, "SharedAppearanceProbe", str(skin_file)], check=True, timeout=30)
