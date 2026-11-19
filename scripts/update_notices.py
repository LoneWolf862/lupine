"""Refresh bundled dependency notices from the locked Cargo source packages."""
import json
import hashlib
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--locked", "--format-version", "1"], cwd=root, text=True
))
parts = ["Lupine third-party dependency notices\nGenerated from Cargo.lock by scripts/update_notices.py.\n"
         "Includes build-time and platform-specific dependencies for completeness.\n"
         "Lupine's own license is in LICENSE (LUPINE_LICENSE.txt in adventure exports).\n"]
texts = {}
for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
    if package["source"] is None:
        continue
    directory = Path(package["manifest_path"]).parent
    parts.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\n"
                 f"License expression: {package.get('license') or 'see license file'}\n"
                 f"Source: {package.get('repository') or package['source']}\n")
    licenses = sorted(p for p in directory.iterdir() if p.is_file()
                      and p.name.upper().startswith(("LICENSE", "COPYING", "NOTICE", "UNLICENSE")))
    if package.get("license_file"):
        extra = directory / package["license_file"]
        if extra.is_file() and extra not in licenses:
            licenses.append(extra)
    if not licenses:
        fallback = root / "scripts" / "licenses" / f"{package.get('license')}.txt"
        if fallback.is_file():
            licenses.append(fallback)
            parts.append("Standard license text from https://www.boost.org/LICENSE_1_0.txt\n")
        else:
            raise SystemExit(f"No license text found for {package['name']}; inspect before distribution")
    for path in licenses:
        text = path.read_text(encoding="utf-8").strip()
        key = hashlib.sha256(text.encode()).hexdigest()
        if key not in texts:
            texts[key] = (len(texts) + 1, text)
        parts.append(f"Included license text {texts[key][0]} ({path.name})\n")
for number, text in texts.values():
    parts.append(f"\n{'=' * 72}\nLicense text {number}\n\n{text}\n")
(root / "THIRD_PARTY_NOTICES.txt").write_text("".join(parts), encoding="utf-8")
