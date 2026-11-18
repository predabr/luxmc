from pathlib import Path

root = Path(__file__).resolve().parents[2]
source = (root / "docs/validation/verify-home-light-news-native.py").read_text(encoding="utf-8")
source = source.replace("Path(os.environ['LOCALAPPDATA']) / 'Luxmc/luxmc.exe'", "Path(os.environ['LOCALAPPDATA']) / 'Luxmc-build/target/release/luxmc.exe'")
source = source.replace("    initialized = call(1, 'app_init')", "    specifications = call(10, 'get_system_specs')\n    assert specifications['launcherVersion'] == '3.0.0'\n    initialized = call(1, 'app_init')")
source = source.replace("report = {'guiSubsystem': True", "report = {'version': specifications['launcherVersion'], 'guiSubsystem': True")
source = source.replace("home-light-news-native.json", "v3-native-verify.json")
(root / "docs/validation/verify-v3-native.py").write_text(source, encoding="utf-8")
print("Native verification prepared for the build directory; no user installation required.")
