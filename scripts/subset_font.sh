#!/usr/bin/env bash
# Builds assets/fonts/ui.ttf: the CJK font cut down to the characters the game
# uses (every char found in src/**/*.rs and assets/**/*.ron plus ASCII and
# common punctuation). Keeps the web download small (~1-2 MB instead of 24 MB).
# Needs the full font (scripts/fetch_assets.sh --font) and python fonttools
# (installed into .venv on first run).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FULL="$ROOT/assets/fonts/_full_LXGWWenKai-Regular.ttf"
OUT="$ROOT/assets/fonts/ui.ttf"
if [ ! -s "$FULL" ]; then
  if [ -s "$OUT" ]; then echo "subset_font: full font missing, keeping existing ui.ttf"; exit 0; fi
  echo "subset_font: $FULL missing (run scripts/fetch_assets.sh --font)"; exit 1
fi
if [ ! -x "$ROOT/.venv/bin/pyftsubset" ]; then
  python3 -m venv "$ROOT/.venv"
  "$ROOT/.venv/bin/pip" -q install fonttools
fi
CHARS="$(mktemp)"
python3 - "$ROOT" > "$CHARS" <<'EOF'
import sys, os
root = sys.argv[1]
chars = set(chr(c) for c in range(0x20, 0x7f))
chars |= set("，。、；：？！“”‘’（）《》〈〉【】—…·～「」『』　①②③④⑤⑥⑦⑧⑨⑩←→↑↓★☆●○◆◇■□▲△▼▽♪♫✦✧·•×÷°±≈≤≥")
for base in ("src", "assets"):
    for dp, _, fs in os.walk(os.path.join(root, base)):
        if "_downloads" in dp:
            continue
        for f in fs:
            if f.endswith((".rs", ".ron", ".txt")):
                try:
                    chars |= set(open(os.path.join(dp, f), encoding="utf-8").read())
                except Exception:
                    pass
print("".join(sorted(c for c in chars if c >= " ")))
EOF
"$ROOT/.venv/bin/pyftsubset" "$FULL" --text-file="$CHARS" --output-file="$OUT" \
  --layout-features='*' --no-hinting --desubroutinize
rm -f "$CHARS"
echo "subset_font: $(wc -c < "$OUT" | tr -d ' ') bytes -> assets/fonts/ui.ttf"
