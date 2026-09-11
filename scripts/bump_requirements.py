#!/usr/bin/env python3
"""Bump Cargo.toml requirements toward the latest published solana crates.

For every `solana-*` dependency, tries to raise the version requirement to
"<major>.<minor>" of the latest stable release on crates.io, so published
solana-awesome versions track upstream (patch releases keep floating).
Bumps are applied one at a time and kept only if the dependency tree still
resolves (`cargo update --dry-run`) *and* no crate ends up in the tree twice
— a bump that conflicts with another crate's internal pins (e.g. clients
lagging behind a core crate) is held back and reported instead of breaking
the build, and so is one that merely resolves by adding a second copy of a
crate, which would silently split a re-exported type in two. Requirements
only move forward.

If anything was bumped, the package version is bumped too: a patch bump
for minor-level updates, a minor bump when any dependency changed major
(breaking for downstream users of a 0.x crate).

Edits Cargo.toml in place preserving formatting, prints a markdown summary
to stdout, and never writes Cargo.lock (the next cargo command re-resolves
minimally). Detect whether it changed anything with `git diff`.

Usage: python3 scripts/bump_requirements.py [--dry-run]
"""

import json
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

API = "https://crates.io/api/v1"
HEADERS = {
    "User-Agent": "solana-awesome-update-agent (https://github.com/Nagaprasadvr/solana-awesome)"
}
REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST = REPO_ROOT / "Cargo.toml"
REQUEST_INTERVAL = 0.5  # crates.io crawler policy: stay well under 1 req/s

DEP_LINE_RE = re.compile(
    r'^(?P<name>solana-[a-z0-9-]+) = \{ version = "(?P<req>[0-9.]+)"'
)
PACKAGE_VERSION_RE = re.compile(r'^version = "(?P<v>\d+\.\d+\.\d+)"$', re.MULTILINE)


def published_floors(crate: str) -> list[tuple[int, int]]:
    """All (major, minor) floors with a stable, non-yanked release, descending."""
    time.sleep(REQUEST_INTERVAL)
    req = urllib.request.Request(f"{API}/crates/{crate}/versions", headers=HEADERS)
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            data = json.load(resp)
    except urllib.error.URLError as e:
        sys.exit(f"error: GET /crates/{crate}/versions failed: {e}")
    floors = set()
    for v in data["versions"]:
        if v.get("yanked") or "-" in v["num"]:
            continue
        major, minor, *_ = v["num"].split(".")
        floors.add((int(major), int(minor)))
    return sorted(floors, reverse=True)


def floor_of(req: str) -> tuple[int, int]:
    parts = req.split(".")
    return int(parts[0]), int(parts[1]) if len(parts) > 1 else 0


def resolves() -> bool:
    # No `check=True`: a non-zero exit is the expected answer for a bump that
    # conflicts with another crate's pins, and it has to come back as False so
    # the caller can hold that bump back. Raising here aborts the whole run.
    return (
        subprocess.run(
            ["cargo", "update", "--dry-run", "--quiet"],
            cwd=REPO_ROOT,
            capture_output=True,
        ).returncode
        == 0
    )


def version_counts() -> dict[str, int]:
    """How many distinct versions of each crate the full-feature tree holds.

    Resolving is not enough on its own: a bump can resolve happily by adding a
    *second* copy of a crate, and then `solana_awesome::instruction_error` and
    the error type `solana_awesome::transaction` hands back are different
    types. Same story one level down, where two `wincode` 0.x majors stop the
    tree compiling at all. So a bump is only kept if no crate gains a copy.
    """
    proc = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--all-features", "-q"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return {}
    counts: dict[str, int] = {}
    for pkg in {(p["name"], p["version"]) for p in json.loads(proc.stdout)["packages"]}:
        counts[pkg[0]] = counts.get(pkg[0], 0) + 1
    return counts


def accepted(baseline: dict[str, int]) -> bool:
    """True if the tree resolves and no crate picked up an extra version."""
    if not resolves():
        return False
    counts = version_counts()
    return bool(counts) and all(n <= baseline.get(name, n) for name, n in counts.items())


def deps_section(text: str) -> tuple[int, int]:
    start = text.index("[dependencies]\n")
    end = re.compile(r"^\[", re.MULTILINE).search(text, start + 1).start()
    return start, end


def bump_line(text: str, name: str, old_req: str, new_req: str) -> str:
    start, end = deps_section(text)
    old_line_prefix = f'{name} = {{ version = "{old_req}"'
    new_line_prefix = f'{name} = {{ version = "{new_req}"'
    return (
        text[:start]
        + text[start:end].replace(old_line_prefix, new_line_prefix, 1)
        + text[end:]
    )


def main() -> None:
    dry_run = "--dry-run" in sys.argv[1:]
    original = MANIFEST.read_text()
    start, end = deps_section(original)

    candidates: list[tuple[str, str, list[tuple[int, int]]]] = []
    for line in original[start:end].splitlines():
        m = DEP_LINE_RE.match(line)
        if not m:
            continue
        name, old_req = m.group("name"), m.group("req")
        newer = [f for f in published_floors(name) if f > floor_of(old_req)]
        if newer:
            candidates.append((name, old_req, newer))

    if not candidates:
        print("No requirement bumps available: everything is on the latest minor.")
        return

    text = original
    applied: list[tuple[str, str, str]] = []
    held_back: list[tuple[str, str, str, str]] = []
    major_changed = False
    pkg = None
    baseline = version_counts()
    if not baseline:
        sys.exit("error: `cargo metadata` failed on the unmodified manifest")
    try:
        # For each dep, keep the highest published floor that still resolves
        # against everything already applied *without* splitting any crate
        # across two versions; record what stays out of reach.
        for name, old_req, newer in candidates:
            chosen = None
            for floor in newer:
                attempt = bump_line(text, name, old_req, f"{floor[0]}.{floor[1]}")
                MANIFEST.write_text(attempt)
                if accepted(baseline):
                    chosen = floor
                    text = attempt
                    baseline = version_counts()
                    break
            latest_req = f"{newer[0][0]}.{newer[0][1]}"
            if chosen:
                new_req = f"{chosen[0]}.{chosen[1]}"
                applied.append((name, old_req, new_req))
                major_changed = major_changed or chosen[0] > floor_of(old_req)[0]
                if chosen != newer[0]:
                    held_back.append((name, new_req, latest_req, "partially"))
            else:
                held_back.append((name, old_req, latest_req, "fully"))

        if applied:
            pkg = PACKAGE_VERSION_RE.search(text)
            if not pkg:
                print(
                    "warning: no package version found in Cargo.toml; not bumping package version"
                )
                return
            major, minor, patch = map(int, pkg.group("v").split("."))
            new_pkg = (
                f"{major}.{minor + 1}.0"
                if major_changed
                else f"{major}.{minor}.{patch + 1}"
            )
            text = text[: pkg.start()] + f'version = "{new_pkg}"' + text[pkg.end() :]
    finally:
        MANIFEST.write_text(original if dry_run else text)

    print(f"## Requirement bumps ({'dry run' if dry_run else 'applied'})\n")
    if applied:
        print("| Crate | Old | New |")
        print("|---|---|---|")
        for name, old_req, new_req in applied:
            print(f"| `{name}` | `{old_req}` | `{new_req}` |")
        kind = "minor (a dependency changed major)" if major_changed else "patch"
        print(f"\nPackage version: `{pkg.group('v')}` -> `{new_pkg}` ({kind} bump).")
    else:
        print("None could be applied.")
    if held_back:
        print("\n### Held back (conflicts with other crates' pins)\n")
        print("| Crate | Now at | Latest | |")
        print("|---|---|---|---|")
        for name, cur_req, latest_req, kind in held_back:
            print(f"| `{name}` | `{cur_req}` | `{latest_req}` | {kind} held back |")
        print(
            "\nThese resolve once the lagging crates catch up upstream"
            " (usually the client group)."
        )


if __name__ == "__main__":
    main()
