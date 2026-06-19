#!/usr/bin/env bash
# 复现/补充 vendored cc65 二进制(ca65 + ld65)。
# 在目标平台上运行,把当前 host 的 Rust target-triple 子目录填好。
#
#   用法:  ./build-cc65.sh [git-ref]
#   依赖:  git, make, cc/clang(Windows 用 MSYS2/MinGW)
set -euo pipefail

CC65_REF="${1:-cc3c40c54e51b2d9a22b63c85c418a2b11763377}"  # 锁定版本(V2.19)
HERE="$(cd "$(dirname "$0")" && pwd)"

# 解析 host target-triple(与 build_pipeline.rs 的子目录命名一致)。
if command -v rustc >/dev/null 2>&1; then
  TRIPLE="$(rustc -vV | awk '/^host:/{print $2}')"
else
  echo "需要 rustc 来确定 target-triple" >&2; exit 1
fi
OUT="$HERE/$TRIPLE"
mkdir -p "$OUT"

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
git clone https://github.com/cc65/cc65.git "$TMP/cc65"
( cd "$TMP/cc65" && git checkout "$CC65_REF" && make -j"$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)" bin )

EXE=""; case "$TRIPLE" in *windows*) EXE=".exe";; esac
cp "$TMP/cc65/bin/ca65$EXE" "$OUT/"
cp "$TMP/cc65/bin/ld65$EXE" "$OUT/"
cp "$TMP/cc65/LICENSE" "$HERE/LICENSE"
echo "vendored ca65/ld65 -> $OUT"
