#!/usr/bin/env bash
# Downloads the CC0 (and one OFL font) source packs and copies the files the
# manifest refers to into assets/. Safe to re-run: downloads are cached in
# assets/_downloads (gitignored). Every source is listed in assets/CREDITS.md.
#
#   scripts/fetch_assets.sh          # fetch everything
#   scripts/fetch_assets.sh --font   # only the font
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DL="$ROOT/assets/_downloads"
mkdir -p "$DL"

fetch() { # url file
  if [ ! -s "$DL/$2" ]; then
    echo "  downloading $2"
    curl -fsSL --retry 3 -o "$DL/$2.part" "$1" && mv "$DL/$2.part" "$DL/$2"
  fi
}

# ---- Font (SIL OFL 1.1 — the only non-CC0 item; see CREDITS.md) ---------------
fetch "https://github.com/lxgw/LxgwWenKai/releases/download/v1.522/LXGWWenKai-Regular.ttf" LXGWWenKai-Regular.ttf
mkdir -p "$ROOT/assets/fonts"
cp "$DL/LXGWWenKai-Regular.ttf" "$ROOT/assets/fonts/_full_LXGWWenKai-Regular.ttf"
# The committed font is a subset (all characters used by the game's string
# tables). scripts/subset_font.sh regenerates it; build_web.sh calls it.
"$ROOT/scripts/subset_font.sh"
[ "${1:-}" = "--font" ] && exit 0

# ---- Kenney packs (CC0) -------------------------------------------------------
fetch "https://kenney.nl/media/pages/assets/nature-kit/37ac38a37b-1677698939/kenney_nature-kit.zip" kenney_nature-kit.zip
fetch "https://kenney.nl/media/pages/assets/impact-sounds/87b4ddecda-1677589768/kenney_impact-sounds.zip" kenney_impact-sounds.zip
fetch "https://kenney.nl/media/pages/assets/interface-sounds/fa43c1dd4d-1677589452/kenney_interface-sounds.zip" kenney_interface-sounds.zip
fetch "https://kenney.nl/media/pages/assets/music-jingles/f37e530b9e-1677590399/kenney_music-jingles.zip" kenney_music-jingles.zip

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Nature Kit: every glTF model, so chapters can pick props by manifest key.
unzip -q -o "$DL/kenney_nature-kit.zip" "Models/GLTF format/*" -d "$TMP/nature"
mkdir -p "$ROOT/assets/models/nature"
cp "$TMP/nature/Models/GLTF format/"*.glb "$ROOT/assets/models/nature/"

# Sounds: a curated set (footsteps, impacts, UI).
unzip -q -o "$DL/kenney_impact-sounds.zip" -d "$TMP/impact"
unzip -q -o "$DL/kenney_interface-sounds.zip" -d "$TMP/ui"
unzip -q -o "$DL/kenney_music-jingles.zip" -d "$TMP/jingles"
mkdir -p "$ROOT/assets/audio/sfx" "$ROOT/assets/audio/jingles"
for s in footstep_grass_000 footstep_grass_001 footstep_grass_002 footstep_grass_003 footstep_grass_004 \
         footstep_wood_000 footstep_wood_001 footstep_wood_002 footstep_snow_000 footstep_snow_001 \
         footstep_concrete_000 footstep_concrete_001 footstep_carpet_000 footstep_carpet_001 \
         impactBell_heavy_000 impactGlass_light_000 impactWood_light_000 impactSoft_medium_000 \
         impactMetal_light_000 impactPlank_medium_000 impactTin_medium_000; do
  cp "$TMP/impact/Audio/$s.ogg" "$ROOT/assets/audio/sfx/"
done
for s in click_001 click_002 select_001 confirmation_001 confirmation_002 open_001 close_001 \
         back_001 tick_001 toggle_001 drop_001 glass_001 pluck_001 maximize_001 question_001; do
  cp "$TMP/ui/Audio/$s.ogg" "$ROOT/assets/audio/sfx/"
done
# Jingles: the "8-bit"/"steel"/"pizzicato" folders vary by pack version; copy
# the soft piano / sax ones that exist.
find "$TMP/jingles" -name '*.ogg' | grep -i -E 'jingles_(pizzi|steel)' | head -16 | while read -r f; do
  cp "$f" "$ROOT/assets/audio/jingles/"
done
echo "assets fetched into $ROOT/assets"
