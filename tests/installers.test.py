import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class Installers(unittest.TestCase):
    def run_installer(self, distro, fail_native=False, fail_download=False):
        with tempfile.TemporaryDirectory(prefix='luxmc-installer-test-') as tmp:
            base = Path(tmp)
            commands = base / 'commands'
            commands.mkdir()
            home = base / 'user'
            home.mkdir()
            log = base / 'commands.log'
            mock = commands / 'mock'
            mock.write_text('''#!/usr/bin/env bash
name="${0##*/}"
printf '%s\\n' "$name $*" >> "$TEST_LOG"
case "$name" in
  curl)
    if [[ "$*" == *api.github.com* ]]; then
      printf '%s\\n' '{"tag_name": "v2.1.0"}'
      exit 0
    fi
    while [ "$#" -gt 0 ]; do
      if [ "$1" = -o ]; then
        shift
        printf payload > "$1"
        [ "$FAIL_DOWNLOAD" = 0 ]
        exit
      fi
      shift
    done ;;
  sudo) exec "$@" ;;
  pacman|dpkg|dnf|apt-get)
    if [[ "$*" == *-U* || "$*" == *install* ]]; then
      [ "$FAIL_NATIVE" = 0 ]
      exit
    fi ;;
esac
exit 0
''')
            mock.chmod(0o755)
            for name in ['curl', 'sudo', 'pacman', 'dpkg', 'dnf', 'apt-get', 'update-desktop-database', 'gtk-update-icon-cache']:
                (commands / name).symlink_to(mock)
            script = (ROOT / 'website/install.sh').read_text().replace('$HOME', str(home))
            for name, marker in [('arch', 'arch-release'), ('fedora', 'fedora-release'), ('debian', 'debian_version')]:
                path = base / marker
                if name == distro:
                    path.touch()
                script = script.replace('/etc/' + marker, str(path))
            dest = home / '.local/bin/luxmc'
            dest.parent.mkdir(parents=True)
            dest.write_text('previous')
            env = dict(os.environ, PATH=str(commands) + ':' + os.environ['PATH'], TEST_LOG=str(log), FAIL_NATIVE=str(int(fail_native)), FAIL_DOWNLOAD=str(int(fail_download)))
            env.pop('XDG_DATA_HOME', None)
            result = subprocess.run(['bash'], input=script, text=True, capture_output=True, env=env)
            calls = log.read_text()
            if fail_download:
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(dest.read_text(), 'previous')
            else:
                self.assertEqual(result.returncode, 0, result.stderr)
                if fail_native or distro == 'generic':
                    self.assertEqual(dest.read_text(), 'payload')
                    self.assertIn('StartupWMClass=luxmc', (home / '.local/share/applications/luxmc.desktop').read_text())
                else:
                    self.assertNotIn('.AppImage', calls)
                    expected = {'arch': 'pacman -U', 'debian': 'apt-get install', 'fedora': 'dnf install'}[distro]
                    self.assertIn(expected, calls)
                    self.assertNotRegex(calls, r'pacman -U[^\n]*https://')

    def test_native_packages(self):
        for distro in ['arch', 'debian', 'fedora']:
            with self.subTest(distro=distro):
                self.run_installer(distro)

    def test_native_failure_falls_back(self):
        for distro in ['arch', 'debian', 'fedora']:
            with self.subTest(distro=distro):
                self.run_installer(distro, fail_native=True)

    def test_generic(self):
        self.run_installer('generic')

    def test_download_failure_preserves_binary(self):
        self.run_installer('generic', fail_download=True)

    def test_uninstall_cleans_integration_preserves_data(self):
        with tempfile.TemporaryDirectory(prefix='luxmc-uninstall-test-') as tmp:
            base = Path(tmp)
            home = base / 'user'
            local = base / 'local'
            system = base / 'system'
            commands = base / 'commands'
            commands.mkdir()
            log = base / 'commands.log'
            mock = commands / 'mock'
            mock.write_text("#!/usr/bin/env bash\nprintf '%s\\n' \"${0##*/} $*\" >> \"$TEST_LOG\"\nif [[ \"${0##*/}\" = sudo ]]; then exec \"$@\"; fi\nexit 0\n")
            mock.chmod(0o755)
            for name in ['pacman', 'sudo', 'update-desktop-database', 'gtk-update-icon-cache']:
                (commands / name).symlink_to(mock)
            targets = [home / '.local/bin/luxmc', local / 'bin/luxmc']
            for share in [home / '.local/share', local / 'share', system / 'share']:
                for name in ['luxmc', 'Luxmc', 'io.github.luxmc.Luxmc', 'luxmc-handler', 'luxmc-debug-handler']:
                    targets.append(share / f'applications/{name}.desktop')
                    for size in ['512x512', '128x128', '64x64', '32x32', '48x48', '16x16']:
                        targets.append(share / f'icons/hicolor/{size}/apps/{name}.png')
                    targets.append(share / f'pixmaps/{name}.png')
            for target in targets:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text('fixture')
            world = home / '.local/share/luxmc/world.dat'
            world.parent.mkdir(parents=True)
            world.write_text('world')
            script = (ROOT / 'website/uninstall.sh').read_text().replace('$HOME', str(home)).replace('/usr/local', str(local)).replace('/usr/share', str(system / 'share'))
            env = dict(os.environ, PATH=str(commands) + ':' + os.environ['PATH'], TEST_LOG=str(log))
            env.pop('XDG_DATA_HOME', None)
            result = subprocess.run(['bash'], input=script, text=True, capture_output=True, env=env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(all(not target.exists() for target in targets))
            self.assertEqual(world.read_text(), 'world')
            self.assertIn('pacman -Rns', log.read_text())
            self.assertIn('gtk-update-icon-cache', log.read_text())
            self.assertIn('update-desktop-database', log.read_text())


if __name__ == '__main__':
    unittest.main()
