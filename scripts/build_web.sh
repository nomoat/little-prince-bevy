#!/usr/bin/env bash
# Builds the web version into dist/:
#   index.html + little_prince.js (wasm-bindgen) + little_prince_bg.wasm (wasm-opt) + assets/
#
#   scripts/build_web.sh            # WebGL2 (default; best for phones), wasm-opt -Oz
#   scripts/build_web.sh --webgpu   # WebGPU renderer instead
#   scripts/build_web.sh --speed    # wasm-opt -O3 (faster, bigger)
#   scripts/build_web.sh --fast     # skip wasm-opt (quick iteration)
# Env: CARGO_TARGET_DIR (default ./target), DIST (default ./dist)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.cargo/bin:$PATH"   # rustup toolchain (has wasm32 std)
FEATURES="webgl2"
OPT="-Oz"
for a in "$@"; do
  case "$a" in
    --webgpu) FEATURES="webgpu" ;;
    --speed) OPT="-O3" ;;
    --fast) OPT="" ;;
    *) echo "unknown option $a"; exit 2 ;;
  esac
done
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
DIST="${DIST:-$ROOT/dist}"
PROFILE=web-release

echo "== font subset"
scripts/subset_font.sh || echo "   (font subset skipped)"

echo "== cargo build ($FEATURES, $PROFILE)"
cargo build --ignore-rust-version --target wasm32-unknown-unknown --profile "$PROFILE" \
  --no-default-features --features "$FEATURES" --bin little_prince

echo "== wasm-bindgen"
rm -rf "$DIST"
mkdir -p "$DIST"
wasm-bindgen --target web --no-typescript --out-dir "$DIST" --out-name little_prince \
  "$TARGET_DIR/wasm32-unknown-unknown/$PROFILE/little_prince.wasm"

if [ -n "$OPT" ]; then
  echo "== wasm-opt $OPT"
  wasm-opt "$OPT" --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals --enable-reference-types --enable-multivalue \
    -o "$DIST/little_prince_bg.opt.wasm" "$DIST/little_prince_bg.wasm"
  mv "$DIST/little_prince_bg.opt.wasm" "$DIST/little_prince_bg.wasm"
fi

echo "== assets"
cp web/index.html "$DIST/"
mkdir -p "$DIST/assets"
rsync -a --exclude '_downloads' --exclude '_full_*' --exclude '.DS_Store' assets/ "$DIST/assets/"
du -sh "$DIST/little_prince_bg.wasm" "$DIST/assets" | sed 's/^/   /'
echo "done -> $DIST   (serve with scripts/serve.sh)"
