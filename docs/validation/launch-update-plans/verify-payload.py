import hashlib
import json
from pathlib import Path

compiled = Path('C:/Users/preda/AppData/Local/Luxmc-build/validation/launch-update-plans/luxmc.exe').read_bytes()
packaged = Path('docs/validation/launch-update-plans/payload/luxmc.exe').read_bytes()
assert len(packaged) == len(compiled)
differences = [index for index, (left, right) in enumerate(zip(packaged, compiled)) if left != right]
assert len(differences) == 3
offset = differences[0]
assert differences == list(range(offset, offset + 3))
assert packaged[offset - 7:offset + 3] == b'PE_VAR_NSS'
assert compiled[offset - 7:offset + 3] == b'PE_VAR_UNK'
assert packaged[:offset] + b'UNK' + packaged[offset + 3:] == compiled, 'Installer program differs beyond the Tauri bundle marker'
report = {
    'compiledSha256': hashlib.sha256(compiled).hexdigest(),
    'packagedSha256': hashlib.sha256(packaged).hexdigest(),
    'onlyDifference': 'Tauri bundle marker UNK -> NSS',
    'compiledProgramVerified': True,
    'helperBundled': False
}
Path('docs/validation/launch-update-plans/payload-verification.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
print(json.dumps(report))
