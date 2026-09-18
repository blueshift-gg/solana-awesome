#!/usr/bin/env python3
"""Bump Cargo.toml requirements toward the latest published solana crates.

For every `solana-*` dependency, tries to raise the version requirement to
"<major>.<minor>" of the latest stable release on crates.io, so published
solana-awesome versions track upstream (patch releases keep floating).
Bumps are applied one at a time and kept only if the dependency tree still
resolves *and* no crate ends up in the tree twice — a bump that conflicts
with another crate's internal pins (e.g. clients lagging behind a core
crate) is held back and reported instead of breaking the build, and so is
one that merely resolves by adding a second copy of a crate, which would
silently split a re-exported type in two. Requirements only move forward.

A bump is also held back when the new floor is out of reach for some
*sibling* version this manifest still admits: weak pass-throughs
(`solana-foo?/serde`) put every optional crate into a downstream lockfile
even when its feature is off (rust-lang/cargo#10801), so a downstream that
pins, say, `solana-transaction` 4.1.x — which wants
`solana-instruction-error >=2.4, <2.5` — must still be able to resolve
this manifest with `solana-instruction-error` at its floor. Resolving at
the latest of everything cannot see that, so each candidate floor is checked
against the requirements every in-range version of every sibling declares
on crates.io. `--audit` runs that check on the manifest as it is.

If anything was bumped, the package version is bumped too: a minor bump
when any dependency changed in a breaking way (major for 1.x deps, minor
for 0.x deps), a patch bump otherwise.

Edits Cargo.toml in place preserving formatting, prints a markdown summary
to stdout, and restores Cargo.lock afterwards (resolution is a side effect
of probing, not an intended output). Detect whether it changed anything
with `git diff`.

Usage: python3 scripts/bump_requirements.py [--dry-run | --audit]
"""

import functools
import json
import re
import shutil
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
LOCKFILE = REPO_ROOT / "Cargo.lock"
REQUEST_INTERVAL = 0.5  # crates.io crawler policy: stay well under 1 req/s

DEP_LINE_RE = re.compile(
    r'^(?P<name>solana-[a-z0-9-]+) = \{ version = "(?P<req>[0-9.]+)"'
)
# Anchored to the [package] section so a dependency's own `version = "..."`
# line can never be mistaken for the package version.
PACKAGE_VERSION_RE = re.compile(
    r'^\[package\]$.*?^version = "(?P<v>\d+\.\d+\.\d+)"$',
    re.MULTILINE | re.DOTALL,
)

Floor = tuple[int, int]


class Manifest:
    """Owns the on-disk manifest and lockfile during probing.

    Every probe runs cargo, which rewrites Cargo.lock. The lock is snapshotted
    on entry and put back on exit so the script's only output is Cargo.toml.
    """

    def __init__(self, path: Path, lock: Path):
        self.path = path
        self.lock = lock
        self.original = path.read_text()
        self._lock_backup: bytes | None = None

    def __enter__(self) -> "Manifest":
        if self.lock.exists():
            self._lock_backup = self.lock.read_bytes()
        return self

    def __exit__(self, *exc) -> None:
        if self._lock_backup is not None:
            self.lock.write_bytes(self._lock_backup)
        elif self.lock.exists():
            self.lock.unlink()

    def write(self, text: str) -> None:
        self.path.write_text(text)

    def restore(self) -> None:
        self.write(self.original)


Version = tuple[int, int, int]


def api_get(path: str) -> dict:
    time.sleep(REQUEST_INTERVAL)
    req = urllib.request.Request(f"{API}{path}", headers=HEADERS)
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.load(resp)
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as e:
        sys.exit(f"error: GET {path} failed: {e}")


@functools.cache
def published_versions(crate: str) -> tuple[Version, ...]:
    """Every stable, non-yanked release, ascending."""
    versions = set()
    for v in api_get(f"/crates/{crate}/versions").get("versions", []):
        if v.get("yanked") or "-" in v["num"]:
            continue
        try:
            versions.add(parse_version(v["num"]))
        except ValueError:
            continue
    return tuple(sorted(versions))


def published_floors(crate: str) -> list[Floor]:
    """All (major, minor) floors with a stable, non-yanked release, descending."""
    return sorted({v[:2] for v in published_versions(crate)}, reverse=True)


@functools.cache
def dependencies_of(crate: str, version: Version) -> tuple[tuple[str, str], ...]:
    """(crate, requirement) for each non-optional normal/build dependency of
    one published release — the ones cargo has to satisfy whenever that
    release is in a tree."""
    deps = api_get(f"/crates/{crate}/{fmt_version(version)}/dependencies")
    return tuple(
        (d["crate_id"], d["req"])
        for d in deps.get("dependencies", [])
        if d.get("kind", "normal") != "dev" and not d.get("optional")
    )


def parse_version(text: str) -> Version:
    parts = [int(p) for p in text.split(".")]
    if not 1 <= len(parts) <= 3:
        raise ValueError(text)
    return tuple(parts + [0] * (3 - len(parts)))  # type: ignore[return-value]


def fmt_version(v: Version) -> str:
    return ".".join(map(str, v))


def floor_of(req: str) -> Floor:
    parts = req.split(".")
    return int(parts[0]), (int(parts[1]) if len(parts) > 1 else 0)


def breaking_axis(v: Version) -> tuple[int, int]:
    """(major, 0) for 1.x+ and (0, minor) for 0.x: two versions unify only if
    they share this, and cargo will never hold two copies that do."""
    return (v[0], 0) if v[0] > 0 else (0, v[1])


def floor_admits(floor: Floor, v: Version) -> bool:
    """Whether a `"major.minor"` requirement can resolve to `v`."""
    return v[:2] >= floor and breaking_axis(v) == breaking_axis((*floor, 0))


_COMPARATOR_RE = re.compile(r"^(\^|~|=|>=|<=|>|<)?\s*([0-9]+(?:\.[0-9]+){0,2}|\*)$")


def req_admits(req: str, v: Version) -> bool:
    """Whether a crates.io requirement string (`^1.2.3`, `>=2.4.0, <2.5.0`,
    `~3.1`, `=4.2.2`, `*`) accepts `v`. Unparseable input is treated as
    accepting everything so an exotic requirement never blocks a bump."""
    for part in req.split(","):
        m = _COMPARATOR_RE.match(part.strip().replace(" ", ""))
        if not m:
            return True
        op, text = m.group(1) or "^", m.group(2)
        if text == "*":
            continue
        want = parse_version(text)
        n = text.count(".") + 1
        if op == "^":
            if want[0] > 0 or n == 1:
                hi = (want[0] + 1, 0, 0)
            elif want[1] > 0 or n == 2:
                hi = (0, want[1] + 1, 0)
            else:
                hi = (0, 0, want[2] + 1)
            ok = want <= v < hi
        elif op == "~":
            hi = (want[0] + 1, 0, 0) if n == 1 else (want[0], want[1] + 1, 0)
            ok = want <= v < hi
        else:
            ok = {
                "=": v == want,
                ">=": v >= want,
                "<=": v <= want,
                ">": v > want,
                "<": v < want,
            }[op]
        if not ok:
            return False
    return True


def req_lower_bound(req: str) -> Version | None:
    """The smallest version a requirement names, or None if it has no floor."""
    bounds = []
    for part in req.split(","):
        m = _COMPARATOR_RE.match(part.strip().replace(" ", ""))
        if m and m.group(2) != "*" and (m.group(1) or "^") in ("^", "~", "=", ">=", ">"):
            bounds.append(parse_version(m.group(2)))
    return min(bounds) if bounds else None


def sibling_conflict(name: str, floor: Floor, reqs: dict[str, str]) -> str | None:
    """Why `name` at `floor` cannot sit next to some sibling version this
    manifest admits, or None if every sibling version can still resolve it.

    Only same-breaking-axis conflicts count: a sibling wanting another major
    of `name` merely resolves a second copy, which cargo allows and the
    duplicate check elsewhere will judge; two copies of one major it will not
    hold, so that is the case that breaks a downstream `cargo update`.
    """
    admitted = [v for v in published_versions(name) if floor_admits(floor, v)]
    if not admitted:
        return f"no published {name} release matches {floor[0]}.{floor[1]}"
    for sibling, sibling_req in reqs.items():
        if sibling == name:
            continue
        sibling_floor = floor_of(sibling_req)
        for version in published_versions(sibling):
            if not floor_admits(sibling_floor, version):
                continue
            for dep, req in dependencies_of(sibling, version):
                if dep != name:
                    continue
                low = req_lower_bound(req)
                if low is None or breaking_axis(low) != breaking_axis(admitted[0]):
                    continue
                if not any(req_admits(req, v) for v in admitted):
                    return f"`{sibling}` {fmt_version(version)} wants `{req}`"
    return None


def is_breaking(old: Floor, new: Floor) -> bool:
    """Cargo treats the leftmost non-zero component as the breaking axis.

    So 1.2 -> 2.0 is breaking, and so is 0.2 -> 0.3, but 1.2 -> 1.3 is not.
    """
    if old[0] != new[0]:
        return True
    return old[0] == 0 and old[1] != new[1]


def version_counts() -> dict[str, int]:
    """How many distinct versions of each crate the full-feature tree holds.

    Empty dict means the tree did not resolve — `cargo metadata` performs a
    full resolution, so a non-zero exit is the same signal `cargo update
    --dry-run` would give, without the second resolve.

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
    for name, _version in {
        (p["name"], p["version"]) for p in json.loads(proc.stdout)["packages"]
    }:
        counts[name] = counts.get(name, 0) + 1
    return counts


def accepted(baseline: dict[str, int]) -> bool:
    """True if the tree resolves and no crate picked up an extra version."""
    counts = version_counts()
    return bool(counts) and all(
        n <= baseline.get(name, n) for name, n in counts.items()
    )


def deps_section(text: str) -> tuple[int, int]:
    """Byte span of the [dependencies] section, header line included."""
    marker = "[dependencies]\n"
    start = text.find(marker)
    if start == -1:
        sys.exit(f"error: no [dependencies] section in {MANIFEST}")
    m = re.compile(r"^\[", re.MULTILINE).search(text, start + len(marker))
    return start, m.start() if m else len(text)


def bump_line(text: str, name: str, old_req: str, new_req: str) -> str:
    start, end = deps_section(text)
    old_prefix = f'{name} = {{ version = "{old_req}"'
    new_prefix = f'{name} = {{ version = "{new_req}"'
    section = text[start:end]
    if old_prefix not in section:
        sys.exit(f"error: could not locate `{name}` requirement to rewrite")
    return text[:start] + section.replace(old_prefix, new_prefix, 1) + text[end:]


def manifest_reqs(text: str) -> dict[str, str]:
    """`solana-*` dependency name -> `"major.minor"` requirement, as written."""
    start, end = deps_section(text)
    reqs = {}
    for line in text[start:end].splitlines():
        m = DEP_LINE_RE.match(line)
        if m:
            reqs[m.group("name")] = m.group("req")
    return reqs


def collect_candidates(text: str) -> list[tuple[str, str, list[Floor]]]:
    candidates = []
    for name, old_req in manifest_reqs(text).items():
        newer = [f for f in published_floors(name) if f > floor_of(old_req)]
        if newer:
            candidates.append((name, old_req, newer))
    return candidates


def apply_bumps(
    manifest: Manifest,
    candidates: list[tuple[str, str, list[Floor]]],
    baseline: dict[str, int],
) -> tuple[str, list, list, bool]:
    """Keep, per dep, the highest published floor that still resolves against
    everything already applied without splitting any crate across two versions.
    """
    text = manifest.original
    applied: list[tuple[str, str, str]] = []
    held_back: list[tuple[str, str, str, str, str]] = []
    breaking = False

    for name, old_req, newer in candidates:
        chosen = None
        rejected: list[tuple[Floor, str]] = []  # why each floor fell through
        for floor in newer:
            conflict = sibling_conflict(name, floor, manifest_reqs(text))
            if conflict:
                rejected.append((floor, conflict))
                continue
            attempt = bump_line(text, name, old_req, f"{floor[0]}.{floor[1]}")
            manifest.write(attempt)
            if accepted(baseline):
                chosen = floor
                text = attempt
                baseline = version_counts()
                break
            rejected.append((floor, "conflicts with other crates' pins"))
        manifest.write(text)  # drop the last rejected attempt
        latest_req = f"{newer[0][0]}.{newer[0][1]}"
        why = explain(rejected)
        if chosen:
            new_req = f"{chosen[0]}.{chosen[1]}"
            applied.append((name, old_req, new_req))
            breaking = breaking or is_breaking(floor_of(old_req), chosen)
            if chosen != newer[0]:
                held_back.append((name, new_req, latest_req, "partially", why))
        else:
            held_back.append((name, old_req, latest_req, "fully", why))

    return text, applied, held_back, breaking


def explain(rejected: list[tuple[Floor, str]]) -> str:
    """One reason, or one per floor when the floors fell for different ones."""
    if len({why for _, why in rejected}) <= 1:
        return rejected[0][1] if rejected else ""
    return "; ".join(f"{f[0]}.{f[1]}: {why}" for f, why in rejected)


def bump_package_version(text: str, breaking: bool) -> tuple[str, str, str] | None:
    """Returns (new_text, old_version, new_version), or None if not found."""
    m = PACKAGE_VERSION_RE.search(text)
    if not m:
        return None
    major, minor, patch = map(int, m.group("v").split("."))
    new_v = f"{major}.{minor + 1}.0" if breaking else f"{major}.{minor}.{patch + 1}"
    return (
        text[: m.start("v")] + new_v + text[m.end("v") :],
        m.group("v"),
        new_v,
    )


def report(dry_run, applied, held_back, breaking, pkg_bump) -> None:
    print(f"## Requirement bumps ({'dry run' if dry_run else 'applied'})\n")
    if applied:
        print("| Crate | Old | New |")
        print("|---|---|---|")
        for name, old_req, new_req in applied:
            print(f"| `{name}` | `{old_req}` | `{new_req}` |")
        if pkg_bump:
            old_v, new_v = pkg_bump
            kind = "minor (a dependency changed breaking)" if breaking else "patch"
            print(f"\nPackage version: `{old_v}` -> `{new_v}` ({kind} bump).")
        else:
            print("\nwarning: no [package] version found; package version not bumped.")
    else:
        print("None could be applied.")
    if held_back:
        print("\n### Held back\n")
        print("| Crate | Now at | Latest | | Why |")
        print("|---|---|---|---|---|")
        for name, cur_req, latest_req, kind, why in held_back:
            print(f"| `{name}` | `{cur_req}` | `{latest_req}` | {kind} held back | {why} |")
        print(
            "\nThese resolve once the lagging crates catch up upstream"
            " (usually the client group, or a sibling's older minors"
            " dropping out of the range this manifest admits)."
        )


def audit(text: str) -> int:
    """Check every requirement as written against its siblings' ranges.

    Prints one line per conflict and returns the count, so CI can gate on
    it: a non-zero exit means some downstream pin this manifest claims to
    support cannot actually resolve it.
    """
    reqs = manifest_reqs(text)
    conflicts = 0
    for name, req in reqs.items():
        why = sibling_conflict(name, floor_of(req), reqs)
        if why:
            conflicts += 1
            print(f"{name} = \"{req}\": {why}")
    print(
        f"{conflicts} conflict(s) across {len(reqs)} requirements"
        if conflicts
        else f"ok: all {len(reqs)} requirements fit every sibling version they admit"
    )
    return conflicts


def main() -> None:
    dry_run = "--dry-run" in sys.argv[1:]
    if not MANIFEST.exists():
        sys.exit(f"error: {MANIFEST} not found")
    if "--audit" in sys.argv[1:]:
        sys.exit(1 if audit(MANIFEST.read_text()) else 0)
    if shutil.which("cargo") is None:
        sys.exit("error: cargo not found on PATH")

    with Manifest(MANIFEST, LOCKFILE) as manifest:
        candidates = collect_candidates(manifest.original)
        if not candidates:
            print("No requirement bumps available: everything is on the latest minor.")
            return

        baseline = version_counts()
        if not baseline:
            manifest.restore()
            sys.exit("error: `cargo metadata` failed on the unmodified manifest")

        pkg_bump = None
        try:
            text, applied, held_back, breaking = apply_bumps(
                manifest, candidates, baseline
            )
            if applied:
                bumped = bump_package_version(text, breaking)
                if bumped:
                    text, old_v, new_v = bumped
                    pkg_bump = (old_v, new_v)
        except BaseException:
            manifest.restore()
            raise

        manifest.write(manifest.original if dry_run else text)

    report(dry_run, applied, held_back, breaking, pkg_bump)


if __name__ == "__main__":
    main()
