"""Assemble and verify a native development package; never publish a release.

Run after cargo build --locked --release. Only tracked docs, templates and playable
examples are copied, so local adventures, saves and build output cannot leak in.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from check_docs import check


def package(binary, destination=None):
    root = Path(__file__).resolve().parents[1]
    version = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    binary = binary.resolve(strict=True)
    manifest = subprocess.check_output([str(binary), "runtime-manifest"], text=True, encoding="utf-8")
    metadata = tomllib.loads(manifest)
    native = "windows-x64" if os.name == "nt" else "linux-x64"
    if (metadata["engine"] != "lupine" or metadata["export_format"] != 1
            or metadata["target"] != native or metadata["version"] != version):
        raise ValueError("Runtime target/version does not match this native development build")
    if metadata["sha256"] != hashlib.sha256(binary.read_bytes()).hexdigest():
        raise ValueError("Runtime digest does not match the executable")
    name = f"Lupine-{version}-{native}"
    destination = (destination or root / "dist" / name).resolve()
    # New destinations only. Existing packages and their evidence are never overwritten.
    if destination.exists():
        raise ValueError(f"Destination already exists: {destination}")
    tracked = subprocess.check_output(["git", "ls-files", "-z"], cwd=root).decode("utf-8").split("\0")
    games = {Path(p).parent.as_posix() for p in tracked if p.startswith("samples/") and p.endswith("/game.toml")}
    paths = []
    for entry in filter(None, tracked):
        path = Path(entry)
        if (entry in {"README.md", "LICENSE", "THIRD_PARTY_NOTICES.txt"}
                or path.parts[0] in {"docs", "templates", "story_example"}
                or any(entry.startswith(game + "/") for game in games)):
            if path.suffix.lower() == ".rork" or any(p in {"target", ".git", ".lupine-room-edit"} for p in path.parts):
                raise ValueError(f"Development/save data in package inventory: {entry}")
            if not (root / path).is_file() or (root / path).is_symlink():
                raise ValueError(f"Package source must be a regular file: {entry}")
            paths.append(path)
    check(root)
    destination.mkdir(parents=True)
    marker = destination / "PACKAGE_INCOMPLETE.txt"
    marker.write_text("Development package assembly did not complete. Do not distribute.\n", encoding="utf-8")
    for path in paths:
        target = destination / path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / path, target)
        if target.read_bytes() != (root / path).read_bytes():
            raise ValueError(f"Package copy mismatch: {path}")
    executable = destination / ("lupine.exe" if os.name == "nt" else "lupine")
    shutil.copy2(binary, executable)
    if hashlib.sha256(executable.read_bytes()).hexdigest() != metadata["sha256"]:
        raise ValueError("Copied runtime digest mismatch")
    (destination / "lupine.runtime.toml").write_text(manifest, encoding="utf-8")
    (destination / "START_HERE.txt").write_text(
        f"Lupine {version} development build for {native}\n"
        "Extract the complete package. No Rust, account or internet is needed.\n"
        "Windows: launch lupine.exe. Linux: chmod u+x lupine, then ./lupine.\n"
        "Startup offers Play, Create or Edit, Help and Exit.\n"
        "Offline documentation: README.md and docs/README.md. Open Markdown in a text editor.\n"
        "Creator guide: docs/CREATOR_QUICKSTART.md; formats: docs/CONTENT_REFERENCE.md.\n"
        "Playable examples: story_example/ and samples/; starter files: templates/starter/.\n"
        "Saves are manual: save <name>, saves, load <name>. Never autosaves.\n"
        "Creator E exports ZIPs or folders using matching local runtimes.\n"
        "Keep lupine.runtime.toml beside this executable for offline cross-platform export.\n"
        "Linux requires a compatible x86_64 glibc system (CI builds on Ubuntu 22.04).\n"
        "This package is not an official release and does not replace v0.9.0.\n",
        encoding="utf-8",
    )
    documents = check(destination)
    # Check the assembled runtime against its copied examples, independently of the checkout.
    for game in ["story_example", *sorted(games)]:
        result = subprocess.run([str(executable), "validate", str(destination / game)],
                                cwd=destination, capture_output=True, text=True, encoding="utf-8", timeout=30)
        if result.returncode or "Valid story:" not in result.stdout:
            raise ValueError(f"Packaged {game} failed validation: {result.stdout}{result.stderr}")
    marker.unlink()
    return dict(name=name, path=str(destination), target=native, version=version,
                documents=documents, examples=1 + len(games), sha256=metadata["sha256"])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path(__file__).resolve().parents[1] / "target" / "release" / ("lupine.exe" if os.name == "nt" else "lupine"))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    print(json.dumps(package(args.binary, args.output), indent=2))
