#!/usr/bin/env bash
set -euo pipefail

cargo run -p oris-runtime \
  --example generate_runtime_api_contract \
  --features "execution-server" \
  --quiet
