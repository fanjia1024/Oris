# Oris Operator CLI

Use this example when you need a small command-line entrypoint for the Oris control plane.

## Choose this path when

- SRE or operators need direct job control from a terminal.
- Your execution server is already deployed and you do not want another web service.
- You want a concrete reference for `run/list/inspect/resume/replay/cancel` requests.

## What this example demonstrates

- `run`
- `list`
- `inspect`
- `resume`
- `replay`
- `cancel`

All commands call execution server APIs and print JSON responses.

## Run

Show the command surface:

```bash
cargo run -p oris_operator_cli -- --help
```

Example requests against a locally running starter server:

```bash
cargo run -p oris_operator_cli -- run demo-thread --input "hello from cli" --idempotency-key demo-key
cargo run -p oris_operator_cli -- list --limit 10
cargo run -p oris_operator_cli -- inspect demo-thread
```

Override the server address with `--server http://host:port`.

## Next edits

1. Add authentication headers/tokens for production control-plane access.
2. Add output formats (`--format json|table`) for operator workflows.
3. Wrap common incident actions into higher-level subcommands.
