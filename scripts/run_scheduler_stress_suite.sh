#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

cargo test -p oris-runtime --features "sqlite-persistence,execution-server" \
  kernel::runtime::api_handlers::tests::scheduler_stress_ \
  -- --nocapture --test-threads=1
