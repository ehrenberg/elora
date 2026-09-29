//! Elora – dedizierter Server (M3.5).
//!
//! Aufruf: `elora-server [--config server.toml] [--port 8303] [--map karte.emap.toml]
//! [--name "…"] [--max-clients 8] [--high-bandwidth] [--key-file server_key.toml]`

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use elora_net::UdpSocket;
use elora_server::{GameServer, ServerConfig, config};
use elora_sim::Tuning;

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TuningFile {
    physics: Tuning,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut cfg = match args.iter().position(|a| a == "--config") {
        Some(i) => ServerConfig::load(Path::new(
            args.get(i + 1).context("--config erwartet einen Pfad")?,
        ))?,
        None => ServerConfig::default(),
    };
    cfg.apply_args(&args)?;

    let map_source = std::fs::read_to_string(&cfg.map)
        .with_context(|| format!("Karte {} nicht lesbar", cfg.map.display()))?;
    let map_name = cfg
        .map
        .file_name()
        .map(|n| {
            n.to_string_lossy()
                .trim_end_matches(".emap.toml")
                .to_string()
        })
        .unwrap_or_default();
    let tuning = match &cfg.tuning {
        Some(p) => {
            let src = std::fs::read_to_string(p)
                .with_context(|| format!("{} nicht lesbar", p.display()))?;
            toml::from_str::<TuningFile>(&src)
                .with_context(|| format!("{} ist ungültig", p.display()))?
                .physics
        }
        None => Tuning::default(),
    };
    let key = config::load_or_create_key(&cfg.key_file)?;
    let addr = cfg.addr()?;
    let socket = UdpSocket::bind(addr).with_context(|| format!("Port {addr} nicht verfügbar"))?;
    let mut server = GameServer::new(
        socket,
        key,
        &cfg,
        &map_name,
        map_source,
        tuning,
        Instant::now(),
    )?;
    tracing::info!(
        name = %cfg.name,
        %addr,
        map = %map_name,
        max_clients = cfg.max_clients,
        snapshots_hz = if cfg.high_bandwidth { 50 } else { 25 },
        key = %elora_net::hex(server.endpoint.public_key()),
        "Server läuft"
    );

    let running = Arc::new(AtomicBool::new(true));
    {
        let running = running.clone();
        let _ = ctrlc_fallback(move || running.store(false, Ordering::SeqCst));
    }

    while running.load(Ordering::SeqCst) {
        let now = Instant::now();
        server.update(now);
        // bis zum nächsten Tick kurz schlafen, Netzwerk aber häufig abfragen
        let wait = server
            .next_tick_at()
            .saturating_duration_since(Instant::now());
        std::thread::sleep(wait.min(Duration::from_millis(1)));
    }
    server.shutdown("Server wird beendet", Instant::now());
    tracing::info!("beendet");
    Ok(())
}

/// Beenden per Zeile „quit“ auf der Standardeingabe. Ist keine Eingabe angeschlossen
/// (z. B. vom Client gestartet), läuft der Server weiter, bis er beendet wird.
fn ctrlc_fallback(stop: impl FnOnce() + Send + 'static) -> std::io::Result<()> {
    std::thread::Builder::new()
        .name("stdin".into())
        .spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                match std::io::stdin().read_line(&mut line) {
                    Ok(0) | Err(_) => return,
                    Ok(_) if line.trim() == "quit" => break,
                    Ok(_) => {}
                }
            }
            stop();
        })?;
    Ok(())
}
