//! Elora – dedizierter Server (M3.5, M4). Befehle im Terminal: `help`.
//!
//! Aufruf: `elora-server [--config server.toml] [--port 8303] [--map karte.emap]
//! [--name "…"] [--max-clients 8] [--high-bandwidth] [--key-file server_key.toml]
//! [--mode dm|tdm|ctf|lms|lts] [--instagib] [--score-limit n] [--time-limit min]
//! [--no-friendly-fire] [--no-votes]`

use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use elora_net::UdpSocket;
use elora_server::{GameServer, MapEntry, ServerConfig, config};
use elora_sim::Tuning;

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TuningFile {
    physics: Tuning,
}

/// Beim Master anmelden und die Anmeldung regelmäßig erneuern (eigener Thread, E-112).
fn register_loop(master: String, port: u16, ipv4_only: bool) {
    let spawned = std::thread::Builder::new()
        .name("master".into())
        .spawn(move || {
            let mut last_ok: Option<bool> = None;
            loop {
                let result = elora_master::client::register(
                    &master,
                    port,
                    elora_protocol::PROTOCOL_VERSION,
                    ipv4_only,
                );
                // nur Wechsel melden, nicht alle 20 s dasselbe
                let ok = result.as_ref().is_ok_and(|r| r.ok);
                if last_ok != Some(ok) {
                    match &result {
                        Ok(r) if r.ok => tracing::info!(%master, "beim Master angemeldet"),
                        Ok(r) => tracing::warn!(%master, reason = %r.message, "Master lehnt ab"),
                        Err(e) => tracing::warn!(%master, "{e:#}"),
                    }
                    last_ok = Some(ok);
                }
                std::thread::sleep(elora_master::REGISTER_INTERVAL);
            }
        });
    if let Err(e) = spawned {
        tracing::warn!("Anmeldung beim Master nicht gestartet: {e}");
    }
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
    // mitgelieferte Karten auch finden, wenn der Server aus einem anderen Ordner startet (M8.2)
    cfg.map = elora_server::paths::resolve(&cfg.map);
    cfg.maps_dir = elora_server::paths::resolve(&cfg.maps_dir);

    let maps = load_maps(&cfg)?;
    let map_name = maps[0].name.clone();
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
    let mut server = GameServer::new(socket, key, &cfg, maps, tuning, Instant::now())?;
    tracing::info!(
        name = %cfg.name,
        %addr,
        map = %map_name,
        max_clients = cfg.max_clients,
        snapshots_hz = if cfg.high_bandwidth { 50 } else { 25 },
        key = %elora_net::hex(server.endpoint.public_key()),
        "Server läuft"
    );

    tracing::info!(maps = %server.map_names().join(", "), mode = %server.rules.cfg.title(), "`help` zeigt die Konsolenbefehle");
    // lauscht der Server nur auf IPv4, meldet er sich auch nur über IPv4 an
    let ipv4_only = addr.is_ipv4();
    for master in cfg.masters.clone() {
        register_loop(master, cfg.port, ipv4_only);
    }

    let commands = console_input()?;
    'run: loop {
        let now = Instant::now();
        for line in commands.try_iter() {
            if line.trim() == "quit" {
                break 'run;
            }
            let out = server.command(&line, now);
            if !out.is_empty() {
                println!("{out}");
            }
        }
        server.update(now);
        // bis zum nächsten Tick kurz schlafen, Netzwerk aber häufig abfragen
        let wait = server
            .next_tick_at()
            .saturating_duration_since(Instant::now());
        std::thread::sleep(wait.min(Duration::from_millis(1)));
    }
    server.shutdown(elora_protocol::reason::SHUTDOWN, Instant::now());
    tracing::info!("beendet");
    Ok(())
}

/// Karte aus `--map` zuerst, dazu alle Karten aus `maps_dir` (Rotation, Abstimmungen).
fn load_maps(cfg: &ServerConfig) -> anyhow::Result<Vec<MapEntry>> {
    let name_of = |p: &Path| {
        p.file_name()
            .map(|n| n.to_string_lossy().trim_end_matches(".emap").to_string())
            .unwrap_or_default()
    };
    let data = std::fs::read(&cfg.map)
        .with_context(|| format!("Karte {} nicht lesbar", cfg.map.display()))?;
    anyhow::ensure!(
        data.len() <= elora_protocol::MAX_MAP,
        "Karte {} ist größer als {} MiB und kann nicht übertragen werden",
        cfg.map.display(),
        elora_protocol::MAX_MAP >> 20
    );
    let first = MapEntry::new(name_of(&cfg.map), data);
    let mut maps = vec![first];
    let mut paths: Vec<_> = std::fs::read_dir(&cfg.maps_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == elora_map::EXTENSION))
        .collect();
    paths.sort();
    for p in paths {
        let name = name_of(&p);
        if maps.iter().any(|m| m.name == name) {
            continue;
        }
        match std::fs::read(&p) {
            Ok(data)
                if data.len() <= elora_protocol::MAX_MAP && elora_map::decode(&data).is_ok() =>
            {
                maps.push(MapEntry::new(name, data));
            }
            _ => {
                tracing::warn!(path = %p.display(), "Karte übersprungen (nicht lesbar, ungültig oder zu groß)");
            }
        }
    }
    for r in &cfg.rotation {
        anyhow::ensure!(
            maps.iter().any(|m| &m.name == r),
            "Rotation: Karte `{r}` nicht in {} gefunden",
            cfg.maps_dir.display()
        );
    }
    Ok(maps)
}

/// Zeilen von der Standardeingabe (Konsole, E-072). Ist keine Eingabe angeschlossen
/// (z. B. vom Client gestartet), läuft der Server einfach weiter.
fn console_input() -> std::io::Result<mpsc::Receiver<String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("konsole".into())
        .spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                match std::io::stdin().read_line(&mut line) {
                    Ok(0) | Err(_) => return,
                    Ok(_) => {
                        if tx.send(line.clone()).is_err() {
                            return;
                        }
                    }
                }
            }
        })?;
    Ok(rx)
}
