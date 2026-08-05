# Examples

## Clearwater plant-of-record smoke

After changing fixtures (or promoting a lab export):

```sh
./scripts/smoke-clearwater.sh
```

See [fixtures/README.md](../fixtures/README.md) for the full promote workflow.

## Headless session run

```sh
# Steady-state plant eval
cargo run -p energy-sim-cli -- hydro eval --config fixtures/plants/clearwater-diversion.json

# Run station for 120 s with lights + EV load; write checkpoint / series / events
cargo run -p energy-sim-cli -- session run \
  --config fixtures/stations/clearwater-diversion.json \
  --duration-secs 120 \
  --out-dir ./run-demo/ \
  --commands examples/commands-lights-ev.json

# Status from checkpoint
cargo run -p energy-sim-cli -- session status --checkpoint ./run-demo/checkpoint.json

# Resume another 60 s
cargo run -p energy-sim-cli -- session resume \
  --checkpoint ./run-demo/checkpoint.json \
  --duration-secs 60 \
  --out-dir ./run-demo/

# Re-export series
cargo run -p energy-sim-cli -- session export \
  --checkpoint ./run-demo/checkpoint.json \
  --format csv \
  --out ./run-demo/series-copy.csv
```

Open `series.csv` in a spreadsheet or plotting tool to see spin-up curves and grid margin.
