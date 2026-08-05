#!/usr/bin/env bash
# Smoke-check Clearwater plant + station fixtures (plant of record).
# Usage (from repo root):
#   ./scripts/smoke-clearwater.sh
#   ./scripts/smoke-clearwater.sh --out-dir /tmp/clearwater-smoke
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

OUT_DIR="${OUT_DIR:-$ROOT/target/clearwater-smoke}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir)
      OUT_DIR="$2"
      shift 2
      ;;
    -h|--help)
      sed -n '2,6p' "$0"
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 2
      ;;
  esac
done

PLANT="$ROOT/fixtures/plants/clearwater-diversion.json"
STATION="$ROOT/fixtures/stations/clearwater-diversion.json"
COMMANDS="$ROOT/examples/commands-lights-ev.json"

for f in "$PLANT" "$STATION" "$COMMANDS"; do
  if [[ ! -f "$f" ]]; then
    echo "missing fixture: $f" >&2
    exit 1
  fi
done

echo "== Clearwater plant eval =="
EVAL_OUT="$(cargo run -q -p energy-sim-cli -- hydro eval --config "$PLANT")"
echo "$EVAL_OUT" | head -c 400
echo ""
if ! echo "$EVAL_OUT" | grep -q '"electricalPowerKw"'; then
  echo "eval output missing electricalPowerKw" >&2
  exit 1
fi
# Plant of record should deliver positive power at default operator inputs.
if echo "$EVAL_OUT" | grep -q '"electricalPowerKw": 0'; then
  echo "unexpected zero electrical power from Clearwater plant" >&2
  exit 1
fi

echo ""
echo "== Station session run (30 s, lights+holo+EV) → $OUT_DIR =="
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
cargo run -q -p energy-sim-cli -- session run \
  --config "$STATION" \
  --duration-secs 30 \
  --out-dir "$OUT_DIR" \
  --commands "$COMMANDS"

for f in checkpoint.json series.csv events.jsonl; do
  if [[ ! -s "$OUT_DIR/$f" ]]; then
    echo "missing or empty output: $OUT_DIR/$f" >&2
    exit 1
  fi
done

if ! head -1 "$OUT_DIR/series.csv" | grep -q 'sim_time_s'; then
  echo "series.csv missing expected header" >&2
  exit 1
fi

echo ""
echo "== Session status =="
cargo run -q -p energy-sim-cli -- session status --checkpoint "$OUT_DIR/checkpoint.json"

echo ""
echo "Clearwater smoke OK"
echo "  plant:   $PLANT"
echo "  station: $STATION"
echo "  out:     $OUT_DIR"
