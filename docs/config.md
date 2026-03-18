# Kleis Runtime Configuration

## IPC Timeouts (DAP/LSP/CLI)

Three IPC timeouts are exposed via config and env:

- `ipc_short_ms` (`KLEIS_IPC_TIMEOUT_SHORT_MS`): quick poll right after the eval thread starts (e.g., first stop event), default 500 ms.
- `ipc_medium_ms` (`KLEIS_IPC_TIMEOUT_MEDIUM_MS`): stepping (`next` / `stepIn` / `stepOut`) should stop quickly, default 5,000 ms.
- `ipc_long_ms` (`KLEIS_IPC_TIMEOUT_LONG_MS`): long wait for `continue` (breakpoint or completion), default 30,000 ms.

These map to the existing `recv_timeout` calls in `src/bin/kleis.rs`.

## Ports

- Server host/port: `server.host`, `server.port` (`KLEIS_SERVER_HOST`, `KLEIS_SERVER_PORT`). Default `127.0.0.1:3000`.

## Z3

- Default timeout for Z3 queries: `z3.timeout_ms` (`KLEIS_Z3_TIMEOUT_MS`). Default 30,000 ms.

## Config file locations (searched in order)

1. Path from `KLEIS_CONFIG` env var
2. `$HOME/.config/kleis/config.toml`
3. `config/kleis.toml` (relative)
4. Built-in defaults

Example `config.toml`:
```toml
[server]
host = "127.0.0.1"
port = 3001

[z3]
timeout_ms = 20000

[timeouts]
ipc_short_ms = 800
ipc_medium_ms = 7000
ipc_long_ms = 45000
```

