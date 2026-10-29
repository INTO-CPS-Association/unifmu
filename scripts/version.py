#!/usr/bin/env python3
"""Read or prepare the shared Cargo workspace release version."""

import argparse
from pathlib import Path
import re
import subprocess
import tomllib


ROOT = Path(__file__).resolve().parent.parent
STABLE_VERSION = re.compile(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)")


def parse_version(value):
    match = STABLE_VERSION.fullmatch(value.removeprefix("v"))
    if not match:
        raise ValueError(f"Expected a stable version such as 0.15.0 or v0.15.0, got {value!r}")
    return tuple(map(int, match.groups()))


def select_version(current, tags, requested):
    current_version = parse_version(current)
    released = [
        parse_version(tag) for tag in tags
        if STABLE_VERSION.fullmatch(tag.removeprefix("v"))
    ]
    latest = max(released, default=(-1, -1, -1))
    if requested:
        version = parse_version(requested)
        if version < current_version or version <= latest:
            raise ValueError("Release version must not precede the workspace version and must exceed existing stable tags")
    else:
        major, minor, patch = max(current_version, latest)
        version = (major, minor, patch + 1)
    return ".".join(map(str, version))


def set_version(manifest, version):
    section = re.compile(r"(^\[workspace\.package\]\s*\n)(.*?)(?=^\[|\Z)", re.M | re.S)
    match = section.search(manifest)
    if not match:
        raise ValueError("Cargo.toml has no [workspace.package] section")
    body, count = re.subn(r'^version\s*=\s*"[^"]+"', f'version = "{version}"', match[2], flags=re.M)
    if count != 1:
        raise ValueError("Expected exactly one workspace.package.version")
    return manifest[:match.start(2)] + body + manifest[match.end(2):]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepare", action="store_true", help="Set the next release version")
    parser.add_argument("--version", help="Override the next-patch default when preparing a release")
    args = parser.parse_args()
    if args.version and not args.prepare:
        parser.error("--version requires --prepare")
    manifest_path = ROOT / "Cargo.toml"
    manifest = manifest_path.read_text(encoding="utf-8")
    current = tomllib.loads(manifest)["workspace"]["package"]["version"]
    try:
        parse_version(current)
        if args.prepare:
            tags = subprocess.check_output(["git", "tag", "--list"], cwd=ROOT, text=True).splitlines()
            current = select_version(current, tags, args.version)
            manifest_path.write_text(set_version(manifest, current), encoding="utf-8")
    except ValueError as error:
        parser.error(str(error))
    print(current)


if __name__ == "__main__":
    main()
