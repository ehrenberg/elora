//! Netzwerk-Thread des Clients: besitzt Socket und Verbindung, stempelt empfangene
//! Pakete mit ihrer Ankunftszeit und sendet ohne Rücksicht auf die Bildrate.

use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use elora_net::{ClientEndpoint, ClientEvent, Conditions, Stats, UdpSocket};

enum Command {
    Send(Vec<u8>, bool),
    Conditions(Conditions),
    Disconnect,
}

/// Verbindung zu einem Server, betrieben in einem eigenen Thread.
#[derive(Debug)]
pub struct Connection {
    pub server: SocketAddr,
    commands: Sender<Command>,
    events: Receiver<(ClientEvent, Instant)>,
    stats: Arc<Mutex<Option<Stats>>>,
}

impl Connection {
    /// Löst `address` auf und startet den Verbindungsaufbau.
    ///
    /// # Errors
    /// Bei ungültiger Adresse oder nicht verfügbarem Socket.
    pub fn open(
        address: &str,
        expected_key: Option<Vec<u8>>,
        conditions: Conditions,
    ) -> anyhow::Result<Self> {
        let server = address
            .to_socket_addrs()
            .with_context(|| format!("Adresse `{address}` ungültig"))?
            .next()
            .with_context(|| format!("Adresse `{address}` nicht auflösbar"))?;
        let bind: SocketAddr = if server.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()?;
        let mut socket = UdpSocket::bind(bind).context("Socket nicht verfügbar")?;
        socket.conditioner.conditions = conditions;
        let (commands, rx_cmd) = mpsc::channel();
        let (tx_events, events) = mpsc::channel();
        let stats = Arc::new(Mutex::new(None));
        let shared = stats.clone();
        std::thread::Builder::new()
            .name("netz".into())
            .spawn(move || {
                let mut ep = ClientEndpoint::connect(socket, server, expected_key, Instant::now());
                loop {
                    let now = Instant::now();
                    for e in ep.poll(now) {
                        let done = matches!(e, ClientEvent::Disconnected(_));
                        if tx_events.send((e, now)).is_err() || done {
                            return;
                        }
                    }
                    loop {
                        match rx_cmd.try_recv() {
                            Ok(Command::Send(data, reliable)) => ep.send(&data, reliable),
                            Ok(Command::Conditions(c)) => {
                                ep.socket_mut().conditioner.conditions = c;
                            }
                            Ok(Command::Disconnect) | Err(mpsc::TryRecvError::Disconnected) => {
                                ep.disconnect(elora_protocol::reason::LEFT, Instant::now());
                                return;
                            }
                            Err(mpsc::TryRecvError::Empty) => break,
                        }
                    }
                    ep.flush(Instant::now());
                    if let Ok(mut s) = shared.lock() {
                        *s = ep.stats();
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            })?;
        Ok(Self {
            server,
            commands,
            events,
            stats,
        })
    }

    pub fn send(&self, data: Vec<u8>, reliable: bool) {
        let _ = self.commands.send(Command::Send(data, reliable));
    }

    pub fn set_conditions(&self, c: Conditions) {
        let _ = self.commands.send(Command::Conditions(c));
    }

    /// Empfangene Ereignisse mit Ankunftszeit.
    pub fn events(&self) -> Vec<(ClientEvent, Instant)> {
        self.events.try_iter().collect()
    }

    pub fn stats(&self) -> Option<Stats> {
        self.stats.lock().ok().and_then(|s| *s)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Disconnect);
    }
}

/// Bekannte Server-Schlüssel (TOFU, E-062): `adresse = "hex"`.
#[derive(Debug, Default)]
pub struct KnownServers {
    keys: HashMap<String, String>,
}

pub const KNOWN_SERVERS_FILE: &str = "known_servers.toml";

impl KnownServers {
    pub fn load(path: &Path) -> Self {
        let keys = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();
        Self { keys }
    }

    pub fn get(&self, server: SocketAddr) -> Option<Vec<u8>> {
        let hex = self.keys.get(&server.to_string())?;
        (0..hex.len())
            .step_by(2)
            .map(|i| {
                hex.get(i..i + 2)
                    .and_then(|b| u8::from_str_radix(b, 16).ok())
            })
            .collect()
    }

    /// Speichert (oder ersetzt) den Schlüssel eines Servers.
    ///
    /// # Errors
    /// Wenn die Datei nicht geschrieben werden kann.
    pub fn trust(&mut self, path: &Path, server: SocketAddr, key: &[u8]) -> anyhow::Result<()> {
        self.keys.insert(server.to_string(), elora_net::hex(key));
        let text = format!(
            "# Elora – bekannte Server-Schlüssel (E-062). Ändert sich ein Schlüssel,\n\
             # warnt der Client vor einem möglichen Angriff.\n{}",
            toml::to_string(&self.keys)?
        );
        std::fs::write(path, text).with_context(|| format!("{} nicht schreibbar", path.display()))
    }
}
