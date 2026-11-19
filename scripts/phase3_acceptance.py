"""Agent/script acceptance through the real executable; not a human usability test.

Build first: cargo build --locked --release
Run: python scripts/phase3_acceptance.py [path-to-lupine]
Artifacts stay in a fresh ignored target/phase3-acceptance-* directory.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import uuid
import zipfile

repo = Path(__file__).resolve().parents[1]
exe = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else repo / "target/release" / ("lupine.exe" if os.name == "nt" else "lupine")
root = repo / "target" / f"phase3-acceptance-{uuid.uuid4().hex[:8]}"
root.mkdir(parents=True)
story = root / "Story with spaces"
shutil.copytree(repo / "story_example", story)
(story / "items/iron_key.toml").write_text('id="iron_key"\nname="Iron Key"\naliases=["key"]\ndescription="A neutral test key."\nportable=true\nstarting_room="foyer"\n', encoding="utf-8")
(story / "rooms/annex.toml").write_text('id="annex"\ntitle="Test Annex"\ndescription="A neutral test location."\n[exits]\nsouth="workshop"\n', encoding="utf-8")
with (story / "rooms/workshop.toml").open("a", encoding="utf-8") as file:
    file.write('\nnorth="annex"\n')
(story / "personal.rork").write_text("not for distribution", encoding="utf-8")
def snapshot(path):
    return {str(p.relative_to(path)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in path.rglob("*") if p.is_file()}
before = snapshot(story)
env = dict(os.environ, APPDATA=str(root / "userdata"), XDG_DATA_HOME=str(root / "userdata"), TERM="dumb")
def run(binary, args, script, label, cwd=root):
    result = subprocess.run([str(binary), *map(str,args)], input=script, text=True, encoding="utf-8",
                            capture_output=True, cwd=cwd, env=env, timeout=90)
    (root / f"{label}.txt").write_text(result.stdout + result.stderr, encoding="utf-8")
    assert result.returncode == 0, result.stderr
    return result.stdout
platform = "windows-x64" if os.name == "nt" else "linux-x64"
number = "1" if os.name == "nt" else "2"
destination = root / "Export package"
script = f"""1
b
{story}
take key
cancel
take key
1
exmain door
use brass_key on workshop_door
north
north
south
south
save checkpoint
drop brass_key
load checkpoint
yes
quit
2
2
{story}
e
{number}
2

{destination}
yes
0
0
4
"""
text = run(exe, [], script, "startup-play-creator-export")
for expected in ["Welcome to Lupine!", "Which key", "Command canceled", "You take the brass key",
                 "Did you mean 'examine door'", "Sample Workshop", "Test Annex", "Saved:",
                 "Loaded: checkpoint", "LUPINE ADVENTURE CREATOR", "EXPORT COMPLETE!"]:
    assert expected in text, f"Missing {expected}; inspect {root}"
assert before == snapshot(story), "source adventure changed"
package = destination / platform
assert not list(package.rglob("*.rork")), "personal save exported"
archive = root / f"Export-Test-{platform}.zip"
with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as z:
    for p in package.rglob("*"):
        z.write(p, p.relative_to(package))
extracted = root / "Extracted package with spaces"
with zipfile.ZipFile(archive) as z:
    z.extractall(extracted)  # This script just created and controls every member.
packaged_exe = extracted / exe.name
if os.name != "nt":
    packaged_exe.chmod(0o755)
unrelated = root / "Empty launch directory"
unrelated.mkdir()
text = run(packaged_exe, [], "1\n1\nlook\nquit\n4\n", "extracted-startup", cwd=unrelated)
assert "Powered by Lupine" in text
text = run(packaged_exe, ["play", extracted / "adventure"], "look\nquit\n", "extracted-direct-play")
assert "Test Foyer" in text
text = run(exe, ["play", repo / "samples/rpg_persistent"],
           "take all\nlearn fireball_scroll\nsouth\nattack goblin with dagger\nuse fireball on goblin\nsave combat\nload combat\nquit\n", "rpg-combat")
for expected in ["You learn Fireball", "You strike", "You cast Fireball on", "Saved:", "Loaded: combat"]:
    assert expected in text, f"Missing {expected}; inspect {root}"
version = run(exe, ["--version"], "", "version").strip()
manifest = run(exe, ["runtime-manifest"], "", "runtime-manifest")
for adventure in [repo / "story_example", *(p for p in (repo / "samples").iterdir() if (p / "game.toml").is_file())]:
    assert "Valid story:" in run(exe, ["validate", adventure], "", "validate-" + adventure.name)
summary = {"version": version, "executable_bytes": exe.stat().st_size, "archive": str(archive),
           "source_unchanged": True, "personal_saves_excluded": True,
           "scripted_native_launch": True, "human_interactive_test": False,
           "history": "requires separate live terminal test", "directory": str(root)}
(root / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
print(json.dumps(summary, indent=2))
