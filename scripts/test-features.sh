#!/usr/bin/env bash
#
# Build and test solana-awesome across its feature matrix.
#
# This crate is nothing but feature-gated re-exports, so what breaks is a
# feature *combination*, not a function: a `#[cfg]` naming a module whose crate
# is not enabled, a test that quietly needs a pass-through nobody turned on, a
# dependency bump that splits the tree across two incompatible wincode majors.
# Each suite below covers one of those failure modes.
#
#   ci           the gate daily-deps.yml runs: --all-features, full, no features
#   groups       every group feature (core, clients, onchain, ...) on its own
#   leaves       every single-crate feature on its own — the slow, thorough one
#   passthrough  full + each pass-through (serde, borsh, wincode, ...) on its own
#   wincode      one wincode major across the whole dependency tree
#   all          all of the above
#
# Feature lists come from `cargo metadata`, so a crate added to Cargo.toml is
# covered the moment it lands — there is no list here to keep in sync by hand.

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$REPO_ROOT"

usage() {
    sed -n '3,20p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: scripts/test-features.sh [options] [suite...]     (default suite: ci)

Options:
  -t, --test         run `cargo test` instead of `cargo check` in the matrix
                     suites (ci always tests)
  -f, --fail-fast    stop at the first failure instead of running the matrix out
  -l, --list         print the feature classification and exit
  -j, --jobs N       pass -j N to cargo
  -q, --quiet        print only the summary
  -h, --help         this

Examples:
  scripts/test-features.sh                  # the CI gate, ~a minute warm
  scripts/test-features.sh all              # everything, go get a coffee
  scripts/test-features.sh leaves -f        # first feature that fails alone
  scripts/test-features.sh groups passthrough -t
EOF
}

# --- options ---------------------------------------------------------------

SUITES=()
ACTION=check
FAIL_FAST=0
QUIET=0
JOBS=()

while [ $# -gt 0 ]; do
    case "$1" in
        -t|--test)      ACTION=test ;;
        -f|--fail-fast) FAIL_FAST=1 ;;
        -q|--quiet)     QUIET=1 ;;
        -j|--jobs)      JOBS=(-j "${2:?--jobs needs a number}"); shift ;;
        -l|--list)      SUITES=(list) ;;
        -h|--help)      usage; exit 0 ;;
        -*)             echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
        ci|groups|leaves|passthrough|wincode|all|list) SUITES+=("$1") ;;
        *)              echo "unknown suite: $1" >&2; usage >&2; exit 2 ;;
    esac
    shift
done
[ ${#SUITES[@]} -gt 0 ] || SUITES=(ci)
if [[ " ${SUITES[*]} " == *" all "* ]]; then
    SUITES=(ci groups passthrough leaves wincode)
fi

command -v jq >/dev/null || { echo "error: this script needs jq" >&2; exit 1; }

# --- output ----------------------------------------------------------------

if [ -t 1 ]; then
    BOLD=$'\e[1m'; RED=$'\e[31m'; GREEN=$'\e[32m'; YELLOW=$'\e[33m'; DIM=$'\e[2m'; OFF=$'\e[0m'
else
    BOLD=''; RED=''; GREEN=''; YELLOW=''; DIM=''; OFF=''
fi

LOG_DIR=$(mktemp -d "${TMPDIR:-/tmp}/solana-awesome-matrix.XXXXXX")
PASSED=0
FAILURES=()

say()     { [ "$QUIET" -eq 1 ] || printf '%s\n' "$*"; }
heading() { [ "$QUIET" -eq 1 ] || printf '\n%s==> %s%s\n' "$BOLD" "$*" "$OFF"; }
elapsed() { printf '%dm%02ds' $(( $1 / 60 )) $(( $1 % 60 )); }

# run_step <label> <command...> — run it, log it, remember whether it passed.
run_step() {
    local label=$1; shift
    local log="$LOG_DIR/$(printf '%s' "$label" | tr -c 'A-Za-z0-9._-' '_').log"
    local start=$SECONDS

    [ "$QUIET" -eq 1 ] || printf '  %-52s ' "$label"
    if "$@" >"$log" 2>&1; then
        PASSED=$(( PASSED + 1 ))
        say "${GREEN}ok${OFF} ${DIM}$(elapsed $(( SECONDS - start )))${OFF}"
    else
        FAILURES+=("$label|$log")
        say "${RED}FAILED${OFF} ${DIM}$(elapsed $(( SECONDS - start )))${OFF}"
        if [ "$FAIL_FAST" -eq 1 ]; then
            printf '\n%s%s%s\n' "$RED" "$label failed:" "$OFF" >&2
            tail -n 40 "$log" >&2
            summary; exit 1
        fi
    fi
}

# cargo_step <label> <cargo-args...> — a `cargo check|test` over one feature set.
cargo_step() {
    local label=$1; shift
    run_step "$label" cargo "$ACTION" "${JOBS[@]}" --no-default-features "$@"
}

# --- feature classification ------------------------------------------------
#
# A feature's own definition says what kind it is, so this needs no hand-kept
# list: `dep:solana-x` is a single-crate leaf, `solana-x?/feat` is a weak
# pass-through, and a list of plain feature names is a group.

FEATURES_JSON=$(cargo metadata --no-deps --format-version 1 --offline 2>/dev/null \
    || cargo metadata --no-deps --format-version 1)

features_of_kind() {
    printf '%s' "$FEATURES_JSON" | jq -r --arg kind "$1" '
        .packages[0].features
        | to_entries
        | map(select(.key != "default" and (.value | length) > 0))
        | map(select(
            if   (.value | all(startswith("dep:"))) then $kind == "leaf"
            elif (.value | all(contains("?/")))     then $kind == "passthrough"
            else                                         $kind == "group"
            end))
        | .[].key' | sort
}

# --- suites ----------------------------------------------------------------

suite_list() {
    local kind
    for kind in group leaf passthrough; do
        printf '%s%s%s\n' "$BOLD" "$kind" "$OFF"
        features_of_kind "$kind" | sed 's/^/  /'
    done
}

# The gate daily-deps.yml runs. --all-features subsumes any smaller check, so it
# goes first; `full` alone then proves no test silently needs a pass-through.
suite_ci() {
    heading "ci gate"
    run_step "cargo test --all-features"        cargo test "${JOBS[@]}" --all-features
    run_step "cargo test --features full"       cargo test "${JOBS[@]}" --no-default-features --features full
    run_step "cargo check --no-default-features" cargo check "${JOBS[@]}" --no-default-features
}

suite_groups() {
    heading "groups — each on its own ($ACTION)"
    local f
    while read -r f; do
        [ -n "$f" ] && cargo_step "$f" --features "$f"
    done < <(features_of_kind group)
}

# Every crate must compile with only its own feature on: that is what catches a
# `#[cfg(feature = "x")]` block reaching for a module gated behind some other
# feature that the full build happens to have enabled.
suite_leaves() {
    heading "leaves — each crate feature alone ($ACTION)"
    local f
    while read -r f; do
        [ -n "$f" ] && cargo_step "$f" --features "$f"
    done < <(features_of_kind leaf)
}

# Pass-throughs are weak forwards, so they mean nothing without crates enabled;
# `full` + one at a time is what tells them apart.
suite_passthrough() {
    heading "pass-throughs — full + each on its own ($ACTION)"
    local f
    while read -r f; do
        [ -n "$f" ] && cargo_step "full,$f" --features "full,$f"
    done < <(features_of_kind passthrough)
}

# Core and client crates must agree on one wincode 0.x (see the README version
# pinning notes); a split fails the build in confusing ways, so name it directly.
suite_wincode() {
    heading "wincode — one major across the tree"
    local versions count
    versions=$(cargo tree --all-features --prefix none 2>/dev/null \
        | grep -oE '^wincode v[0-9]+\.[0-9]+' | sort -u || true)
    count=$(printf '%s' "$versions" | grep -c . || true)
    printf '  %-52s ' "single wincode version"
    if [ "$count" -le 1 ]; then
        PASSED=$(( PASSED + 1 ))
        say "${GREEN}ok${OFF} ${DIM}${versions:-none in tree}${OFF}"
    else
        local log="$LOG_DIR/wincode.log"
        printf '%s\n' "$versions" > "$log"
        FAILURES+=("wincode version split|$log")
        say "${RED}FAILED${OFF} ${DIM}$(printf '%s' "$versions" | tr '\n' ' ')${OFF}"
        [ "$FAIL_FAST" -eq 1 ] && { summary; exit 1; }
    fi
}

# --- summary ---------------------------------------------------------------

summary() {
    local total=$(( PASSED + ${#FAILURES[@]} ))
    printf '\n%s%s%s\n' "$BOLD" "───────────────────────────────────────────" "$OFF"
    if [ ${#FAILURES[@]} -eq 0 ]; then
        printf '%s%d/%d passed%s in %s\n' "$GREEN" "$PASSED" "$total" "$OFF" "$(elapsed $SECONDS)"
        rm -rf "$LOG_DIR"
    else
        printf '%s%d/%d passed, %d failed%s in %s\n\n' \
            "$YELLOW" "$PASSED" "$total" "${#FAILURES[@]}" "$OFF" "$(elapsed $SECONDS)"
        local entry
        for entry in "${FAILURES[@]}"; do
            printf '  %s✗%s %-40s %s%s%s\n' \
                "$RED" "$OFF" "${entry%%|*}" "$DIM" "${entry#*|}" "$OFF"
        done
        printf '\n'
    fi
}

# --- go --------------------------------------------------------------------

for suite in "${SUITES[@]}"; do
    case "$suite" in
        list)        suite_list; exit 0 ;;
        ci)          suite_ci ;;
        groups)      suite_groups ;;
        leaves)      suite_leaves ;;
        passthrough) suite_passthrough ;;
        wincode)     suite_wincode ;;
    esac
done

summary
[ ${#FAILURES[@]} -eq 0 ]
