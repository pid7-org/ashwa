#!/usr/bin/env bash
set -euo pipefail

readonly PROG="${0##*/}"

die()  { printf '%s: %s\n' "$PROG" "$*" >&2; exit 1; }
usage(){ printf 'usage: %s <version>\n' "$PROG" >&2; exit 1; }

[ "$#" -eq 1 ] || usage

readonly NEW="${1#v}"
readonly OLD=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)

[ -n "$OLD" ]        || die "cannot read version from Cargo.toml"
[ "$OLD" != "$NEW" ] || { printf '%s: already at %s\n' "$PROG" "$NEW"; exit 0; }

bump() { [ -f "$1" ] && sed -i "s/$2/$3/" "$1"; }

printf '%s -> %s\n' "$OLD" "$NEW"

bump Cargo.toml             "^version = \"$OLD\""            "version = \"$NEW\""
bump npm/package.json        "\"version\": \"$OLD\""          "\"version\": \"$NEW\""
bump npm/native/package.json "\"version\": \"$OLD\""          "\"version\": \"$NEW\""
bump npm/native/index.js     "$OLD"                           "$NEW"
bump pypi/pyproject.toml     "^version = \"$OLD\""            "version = \"$NEW\""
bump README.md               "ashwa = \"$OLD\""               "ashwa = \"$NEW\""

command -v cargo &>/dev/null && cargo check --workspace -q 2>/dev/null || true

printf 'done\n'
