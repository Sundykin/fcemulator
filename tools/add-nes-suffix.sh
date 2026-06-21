#!/usr/bin/env bash
#
# add-nes-suffix.sh — give NES ROM files a `.nes` extension when they're missing one.
#
#   ./tools/add-nes-suffix.sh [DIR] [-n|--dry-run]
#
# DIR defaults to /Users/sunmeng/workspace/fc/roms.
#
# Rules (safe + idempotent — re-running does nothing):
#   • Skip files that already end in `.nes` (case-insensitive)  ← the suffix check.
#   • Only rename real iNES / NES 2.0 ROMs (magic "NES\x1a"); anything else
#     (e.g. .DS_Store, notes) is left untouched.
#   • Never overwrite an existing `<name>.nes`.
#
set -u

DRY=0
DIR=""
for a in "$@"; do
  case "$a" in
    -n|--dry-run) DRY=1 ;;
    -*) echo "unknown flag: $a" >&2; exit 2 ;;
    *) DIR="$a" ;;
  esac
done
DIR="${DIR:-/Users/sunmeng/workspace/fc/roms}"

if [ ! -d "$DIR" ]; then
  echo "not a directory: $DIR" >&2
  exit 1
fi

renamed=0; have=0; notrom=0; collision=0

while IFS= read -r -d '' f; do
  base="$(basename "$f")"

  # already carries the .nes suffix? (case-insensitive) → nothing to do
  shopt -s nocasematch
  if [[ "$base" == *.nes ]]; then
    have=$((have + 1)); shopt -u nocasematch; continue
  fi
  shopt -u nocasematch

  # is it actually a NES ROM? (iNES / NES 2.0 header magic) → else leave alone
  if [ "$(head -c 4 "$f" 2>/dev/null | xxd -p 2>/dev/null)" != "4e45531a" ]; then
    notrom=$((notrom + 1)); echo "skip (not a ROM): $base"; continue
  fi

  target="$f.nes"
  if [ -e "$target" ]; then
    collision=$((collision + 1)); echo "skip (target exists): $base.nes"; continue
  fi

  if [ "$DRY" = 1 ]; then
    echo "would rename: $base  ->  $base.nes"
  else
    mv -n -- "$f" "$target" && echo "renamed: $base  ->  $base.nes"
  fi
  renamed=$((renamed + 1))
done < <(find "$DIR" -maxdepth 1 -type f -print0)

echo "----"
verb=$([ "$DRY" = 1 ] && echo "would rename" || echo "renamed")
echo "$verb: $renamed | already .nes: $have | not a ROM: $notrom | collisions: $collision"
