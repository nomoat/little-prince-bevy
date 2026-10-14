#!/usr/bin/env bash
# Serves dist/ on http://localhost:${1:-8080}/ (gzip for wasm/js). Phones: use your LAN IP.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec python3 "$ROOT/scripts/serve.py" "${1:-8080}" "${DIST:-$ROOT/dist}"
