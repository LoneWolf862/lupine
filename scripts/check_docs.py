"""Check UTF-8 Markdown and local file links in a checkout or portable package.

Heading anchors and remote URLs are deliberately not checked; this runs offline.
"""
import argparse
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit


def check(root):
    files = [root / "README.md", *sorted((root / "docs").rglob("*.md"))]
    if not (root / "docs" / "README.md").is_file():
        raise ValueError("Missing docs/README.md")
    errors = []
    for path in files:
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as error:
            errors.append(f"{path.relative_to(root)}: {error}")
            continue
        for match in re.finditer(r'!?\[[^\]]*\]\(([^\s)]+)(?:\s+"[^"]*")?\)', text):
            link = urlsplit(match.group(1).strip("<>"))
            if link.scheme or link.netloc or not link.path:
                continue
            target = (path.parent / unquote(link.path)).resolve()
            if not target.is_relative_to(root) or not target.exists():
                errors.append(f"{path.relative_to(root)}: missing/outside local target {link.path}")
    if errors:
        raise ValueError("\n".join(errors))
    return len(files)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", nargs="?", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        count = check(args.root.resolve())
    except ValueError as error:
        raise SystemExit(str(error)) from error
    print(f"Checked {count} UTF-8 Markdown files and local file links (anchors not checked).")
