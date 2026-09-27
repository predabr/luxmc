import argparse
import os
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument("--data-dir", type=Path, required=True)
parser.add_argument("--skin", type=Path)
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
for dependency in dependencies:
    if not (libraries / dependency).is_file():
        raise FileNotFoundError(libraries / dependency)
matrix = [("1.5.25", 8), ("3.11.49", 17), ("4.0.43", 17),
          ("6.0.54", 21), ("7.0.63", 21), ("9.0.75", 25), ("10.0.77", 25)]
with tempfile.TemporaryDirectory(prefix="luxmc-appearance-test-") as output:
    subprocess.run([str(java / "17/bin/javac"), "--release", "8", "-d", output,
                    str(root / "tests/java/AppearanceProbe.java")], check=True)
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
