"""Scripted native CLI authoring, migration, manual saves and integrated ZIP launch.

No manual TOML/FIG authoring. Artifacts remain under a new ignored target directory.
Run after cargo build --locked --release. This is not human beginner testing.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib
import uuid
import zipfile

repo = Path(__file__).resolve().parents[1]
exe = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else repo / 'target/release' / ('lupine.exe' if os.name == 'nt' else 'lupine')
root = repo / 'target' / ('phase4-acceptance-' + uuid.uuid4().hex[:8])
root.mkdir(parents=True)
story = root / 'Adventure with spaces'
data = root / 'userdata'
env = dict(os.environ, APPDATA=str(data), XDG_DATA_HOME=str(data), TERM='dumb')

def run(args, script, label, cwd=root):
    result = subprocess.run(list(map(str, args)), input=script, text=True, encoding='utf-8',
                            capture_output=True, cwd=cwd, env=env, timeout=180)
    (root / (label + '.txt')).write_text(result.stdout + result.stderr, encoding='utf-8')
    assert result.returncode == 0, f'{label}: {result.stderr}'
    return result.stdout

script = (repo / 'tests/fixtures/rpg_creator_acceptance.txt').read_text(encoding='utf-8')
script = script.replace('\n2\n1\nbuy healing', '\nt\nn\nsign\nsupplies\nmana\nsign\n2\nThe altar restores your mana.\n0\ns\nyes\n0\nm\nitems\nsign\naltar\nyes\n0\n2\n1\nbuy healing')
script = script.replace('buy mana', 'use altar').replace('{PROJECT}', str(story))
script = script.replace('Test Sign\n', 'Mana Altar\n')
text = run([exe, 'new'], script, 'creator-gameplay')
for unexpected in ['Save needs attention', 'manager stopped', 'Migration stopped', 'Migration refused']:
    assert unexpected not in text, unexpected
gameplay = text.split('Starting a fresh play session.')[-1]
for expected in ['The test door unlocks.', 'The altar restores your mana.', 'Loaded: final', 'You strike', 'You cast']:
    assert expected in gameplay, expected
assert 'Migration saved. Full validation PASSED.' in text
assert 'Validation successful' in text
saves = list(data.rglob('*.rork'))
assert len(saves) == 8, 'Only explicitly requested save slots should exist'
final_path = next(p for p in saves if p.name.startswith('final.'))
saved = tomllib.loads(final_path.read_text(encoding='utf-8'))
assert saved['save_format_version'] == 6
assert 'altar' in saved['objects']['placements'] and 'sign' not in saved['objects']['placements']
save_hashes = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in saves}
text = run([exe, 'play', story], 'load final\nwallet\nlook\nquit\n', 'restart-manual-load')
assert 'Loaded: final' in text
assert save_hashes == {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in data.rglob('*.rork')}

# Exercise insufficient funds through the same guided offer and binding, then restore
# its original price. Definition changes do not execute transactions or save progress.
failure_script = 'r\n3\nsupplies\n4\nmana\n4\n100\n0\ns\nyes\n0\n0\n1\nuse altar\nwallet\nquit\nr\n3\nsupplies\n4\nmana\n4\n1\n0\ns\nyes\n0\n0\n0\n'
failure = run([exe, 'creator', story], failure_script, 'binding-insufficient-funds')
assert 'Insufficient tokens' in failure and 'Tokens: 2' in failure
assert 'Save needs attention' not in failure
assert save_hashes == {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in data.rglob('*.rork')}

def snapshot(path):
    return {str(p.relative_to(path)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in path.rglob('*') if p.is_file()}

before = snapshot(story)
platform = 'windows-x64' if os.name == 'nt' else 'linux-x64'
number = '1' if os.name == 'nt' else '2'
destination = root / 'Ready to share'
text = run([exe, 'creator', story], f'e\n{number}\n1\n\n{destination}\nyes\n0\n', 'zip-export')
assert 'EXPORT COMPLETE!' in text
assert before == snapshot(story)
archive = next(destination.glob('*.zip'))
assert platform in archive.name
extracted = root / 'Extracted package with spaces'
with zipfile.ZipFile(archive) as z:
    assert z.testzip() is None
    names = z.namelist()
    assert len(names) == len(set(names))
    assert all(not n.startswith('/') and '..' not in n.split('/') for n in names)
    assert all(not n.endswith('.rork') and '.git' not in n.split('/') and 'target' not in n.split('/') for n in names)
    assert 'adventure/art/' in names and 'adventure/config/' in names
    assert 'THIRD_PARTY_NOTICES.txt' in names
    z.extractall(extracted)  # Generated archive, with every path checked above.
# The scaffold README is creator guidance, outside the playable-content allowlist.
assert {k: v for k, v in before.items() if k != 'README.txt'} == snapshot(extracted / 'adventure')
native = extracted / exe.name
assert hashlib.sha256(native.read_bytes()).digest() == hashlib.sha256(exe.read_bytes()).digest()
metadata = tomllib.loads((extracted / 'lupine.runtime.toml').read_text(encoding='utf-8-sig'))
assert metadata['target'] == platform
assert metadata['sha256'] == hashlib.sha256(native.read_bytes()).hexdigest()
if os.name != 'nt':
    native.chmod(0o755)
text = run([native, 'play', extracted / 'adventure'], 'look\nload final\nwallet\nquit\n', 'packaged-play')
assert 'Loaded: final' in text
unrelated = root / 'Unrelated working directory'
unrelated.mkdir()
launcher = [os.environ.get('COMSPEC', 'cmd.exe'), '/d', '/c', str(extracted / 'Play.cmd')] if os.name == 'nt' else ['sh', str(extracted / 'Play.sh')]
text = run(launcher, 'look\nquit\n\n', 'native-launcher', cwd=unrelated)
assert 'Powered by Lupine' in text
for adventure in [repo / 'story_example', *(p for p in (repo / 'samples').iterdir() if (p / 'game.toml').is_file())]:
    assert 'Valid story:' in run([exe, 'validate', adventure], '', 'validate-' + adventure.name)
assert save_hashes == {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in data.rglob('*.rork')}
summary = dict(platform=platform, archive=str(archive), directory=str(root),
               all_content_authored_through_menus=True, source_unchanged=True,
               native_launcher_passed=True, manual_save_slots=8, automatic_saves=False,
               independent_zip_extraction_passed=True, human_beginner_test=False)
(root / 'summary.json').write_text(json.dumps(summary, indent=2), encoding='utf-8')
print(json.dumps(summary, indent=2))
