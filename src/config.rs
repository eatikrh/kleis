use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct KleisConfig {
    pub server: ServerConfig,
    pub z3: Z3Config,
    pub timeouts: TimeoutConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct Z3Config {
    /// Default timeout for Z3 queries (ms)
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Short IPC timeout for quick polls (ms)
    pub ipc_short_ms: u64,
    /// Medium IPC timeout (ms)
    pub ipc_medium_ms: u64,
    /// Long IPC timeout (ms)
    pub ipc_long_ms: u64,
}

impl Default for KleisConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            z3: Z3Config { timeout_ms: 30_000 },
            timeouts: TimeoutConfig {
                ipc_short_ms: 500,
                ipc_medium_ms: 5_000,
                ipc_long_ms: 30_000,
            },
        }
    }
}

/// Load configuration with the following precedence:
/// 1. Env `KLEIS_CONFIG` path
/// 2. `$HOME/.config/kleis/config.toml`
/// 3. `config/kleis.toml` relative to CWD
/// 4. Defaults
/// Env overrides for common fields are applied last:
///   - KLEIS_SERVER_HOST / KLEIS_SERVER_PORT
///   - KLEIS_Z3_TIMEOUT_MS
///   - KLEIS_IPC_TIMEOUT_SHORT_MS / _MEDIUM_MS / _LONG_MS
pub fn load() -> KleisConfig {
    let mut cfg = KleisConfig::default();

    for path in candidate_paths() {
        if let Some(partial) = read_partial(&path) {
            cfg.apply_partial(partial);
            break;
        }
    }

    apply_env_overrides(&mut cfg);
    cfg
}

fn candidate_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    if let Ok(p) = std::env::var("KLEIS_CONFIG") {
        paths.push(std::path::PathBuf::from(p));
    }

    if let Ok(home) = std::env::var("HOME") {
        paths.push(std::path::Path::new(&home).join(".config/kleis/config.toml"));
    }

    paths.push(std::path::PathBuf::from("config/kleis.toml"));

    paths
}

#[derive(Debug, Deserialize)]
struct PartialConfig {
    server: Option<PartialServer>,
    z3: Option<PartialZ3>,
    timeouts: Option<PartialTimeouts>,
}

#[derive(Debug, Deserialize)]
struct PartialServer {
    host: Option<String>,
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct PartialZ3 {
    timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct PartialTimeouts {
    ipc_short_ms: Option<u64>,
    ipc_medium_ms: Option<u64>,
    ipc_long_ms: Option<u64>,
}

fn read_partial(path: &std::path::Path) -> Option<PartialConfig> {
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str::<PartialConfig>(&content).ok()
}

impl KleisConfig {
    fn apply_partial(&mut self, partial: PartialConfig) {
        if let Some(s) = partial.server {
            if let Some(host) = s.host {
                self.server.host = host;
            }
            if let Some(port) = s.port {
                self.server.port = port;
            }
        }

        if let Some(z3) = partial.z3 {
            if let Some(timeout_ms) = z3.timeout_ms {
                self.z3.timeout_ms = timeout_ms;
            }
        }

        if let Some(t) = partial.timeouts {
            if let Some(v) = t.ipc_short_ms {
                self.timeouts.ipc_short_ms = v;
            }
            if let Some(v) = t.ipc_medium_ms {
                self.timeouts.ipc_medium_ms = v;
            }
            if let Some(v) = t.ipc_long_ms {
                self.timeouts.ipc_long_ms = v;
            }
        }
    }
}

fn apply_env_overrides(cfg: &mut KleisConfig) {
    if let Ok(host) = std::env::var("KLEIS_SERVER_HOST") {
        cfg.server.host = host;
    }
    if let Ok(port) = std::env::var("KLEIS_SERVER_PORT") {
        if let Ok(p) = port.parse::<u16>() {
            cfg.server.port = p;
        }
    }
    if let Ok(timeout) = std::env::var("KLEIS_Z3_TIMEOUT_MS") {
        if let Ok(v) = timeout.parse::<u64>() {
            cfg.z3.timeout_ms = v;
        }
    }
    if let Ok(v) = std::env::var("KLEIS_IPC_TIMEOUT_SHORT_MS") {
        if let Ok(v) = v.parse::<u64>() {
            cfg.timeouts.ipc_short_ms = v;
        }
    }
    if let Ok(v) = std::env::var("KLEIS_IPC_TIMEOUT_MEDIUM_MS") {
        if let Ok(v) = v.parse::<u64>() {
            cfg.timeouts.ipc_medium_ms = v;
        }
    }
    if let Ok(v) = std::env::var("KLEIS_IPC_TIMEOUT_LONG_MS") {
        if let Ok(v) = v.parse::<u64>() {
            cfg.timeouts.ipc_long_ms = v;
        }
    }
}
