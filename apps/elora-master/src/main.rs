//! `elora-master` – master server for the internet server list (E-112).
//!
//! Usage: `elora-master [--bind 0.0.0.0:8300] [--behind-proxy]`
//!
//! In public, run it behind a reverse proxy with HTTPS (E-127) and then set
//! `--behind-proxy` so that the sender address is taken from `X-Forwarded-For`.

use std::net::SocketAddr;

use anyhow::Context as _;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let bind: SocketAddr = args
        .iter()
        .position(|a| a == "--bind")
        .and_then(|i| args.get(i + 1))
        .map_or(Ok(SocketAddr::from(([0, 0, 0, 0], 8300))), |s| s.parse())
        .context("--bind expects address:port")?;
    let behind_proxy = args.iter().any(|a| a == "--behind-proxy");
    elora_master::service::run(bind, behind_proxy)
}
