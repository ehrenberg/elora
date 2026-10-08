//! Server browser (M7.7): internet (master, E-112), LAN (broadcast) and favorites.
//!
//! Addresses come from the master, the LAN search or the favorites; each server
//! is then queried itself via UDP (name, map, mode, players, ping – M7.6).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use elora_net::{InfoProbe, Socket, UdpSocket};
use elora_protocol::ServerInfo;

/// Ports that the LAN search queries (like the original: 8303–8310).
pub const LAN_PORTS: std::ops::RangeInclusive<u16> = 8303..=8310;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Internet,
    Lan,
    Favorites,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    #[default]
    Ping,
    Players,
    Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Querying,
    Online { info: ServerInfo, ping: Duration },
    Unreachable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub addr: SocketAddr,
    pub state: State,
}

impl Entry {
    fn players(&self) -> u32 {
        match &self.state {
            State::Online { info, .. } => info.clients,
            _ => 0,
        }
    }
}

/// Filters of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Filter {
    pub hide_empty: bool,
    pub hide_full: bool,
}

/// Browser state; the queries run over a separate UDP socket.
pub struct Browser<S: Socket = UdpSocket> {
    pub tab: Tab,
    pub sort: SortBy,
    pub filter: Filter,
    pub selected: Option<SocketAddr>,
    entries: HashMap<SocketAddr, Entry>,
    probe: Option<InfoProbe<S>>,
    /// Running query of the master list (separate thread).
    master: Option<Receiver<anyhow::Result<Vec<SocketAddr>>>>,
    /// Hint for the UI (errors from the master, missing address …).
    /// Error for the list: language key and technical detail (E-352).
    pub status: Option<(&'static str, String)>,
}

impl<S: Socket> std::fmt::Debug for Browser<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Browser")
            .field("tab", &self.tab)
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}

impl Default for Browser<UdpSocket> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl<S: Socket> Browser<S> {
    pub fn new(probe: Option<InfoProbe<S>>) -> Self {
        Self {
            tab: Tab::default(),
            sort: SortBy::default(),
            filter: Filter::default(),
            selected: None,
            entries: HashMap::new(),
            probe,
            master: None,
            status: None,
        }
    }

    /// Query addresses (new entries "running …").
    pub fn query(&mut self, addrs: &[SocketAddr], now: Instant) {
        let Some(probe) = &mut self.probe else { return };
        for a in addrs {
            probe.query(*a, now);
            self.entries.insert(
                *a,
                Entry {
                    addr: *a,
                    state: State::Querying,
                },
            );
        }
    }

    /// LAN search at `targets` (broadcast addresses and the local machine).
    pub fn discover(&mut self, targets: &[SocketAddr], now: Instant) {
        if let Some(probe) = &mut self.probe {
            probe.discover(targets, now);
        }
    }

    /// Clear the list (before refreshing).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected = None;
    }

    /// Is the browser still waiting for answers?
    pub fn busy(&self) -> bool {
        self.master.is_some() || self.probe.as_ref().is_some_and(InfoProbe::busy)
    }

    /// Collect answers; call every frame.
    pub fn poll(&mut self, now: Instant) {
        if let Some(rx) = &self.master {
            match rx.try_recv() {
                Ok(Ok(addrs)) => {
                    self.master = None;
                    self.status = None;
                    self.query(&addrs, now);
                }
                Ok(Err(e)) => {
                    self.master = None;
                    self.status = Some(("browser.master_failed", format!("{e:#}")));
                }
                Err(TryRecvError::Disconnected) => self.master = None,
                Err(TryRecvError::Empty) => {}
            }
        }
        let Some(probe) = &mut self.probe else { return };
        let (replies, lost) = probe.poll(now);
        for r in replies {
            let state = match ServerInfo::decode(&r.data) {
                Ok(info) => State::Online { info, ping: r.ping },
                Err(_) => State::Unreachable,
            };
            self.entries.insert(
                r.addr,
                Entry {
                    addr: r.addr,
                    state,
                },
            );
        }
        for a in lost {
            if let Some(e) = self.entries.get_mut(&a) {
                e.state = State::Unreachable;
            }
        }
    }

    /// Visible entries after filter and sorting.
    pub fn visible(&self) -> Vec<&Entry> {
        let f = self.filter;
        let mut v: Vec<&Entry> = self
            .entries
            .values()
            .filter(|e| match &e.state {
                State::Online { info, .. } => {
                    (!f.hide_empty || info.clients > 0)
                        && (!f.hide_full || info.clients < info.max_clients)
                }
                _ => true,
            })
            .collect();
        let ping = |e: &Entry| match &e.state {
            State::Online { ping, .. } => *ping,
            State::Querying => Duration::from_secs(10),
            State::Unreachable => Duration::from_secs(20),
        };
        // the same server listed via IPv4 and IPv6: only show the faster route
        let same = |a: &Entry, b: &Entry| match (&a.state, &b.state) {
            (State::Online { info: x, .. }, State::Online { info: y, .. }) => {
                a.addr.port() == b.addr.port()
                    && a.addr.is_ipv4() != b.addr.is_ipv4()
                    && x.name == y.name
                    && x.map == y.map
            }
            _ => false,
        };
        let all = v.clone();
        v.retain(|e| {
            !all.iter()
                .any(|o| same(e, o) && (ping(o), o.addr) < (ping(e), e.addr))
        });
        let name = |e: &Entry| match &e.state {
            State::Online { info, .. } => info.name.to_lowercase(),
            _ => e.addr.to_string(),
        };
        match self.sort {
            SortBy::Ping => v.sort_by_key(|e| (ping(e), e.addr)),
            SortBy::Players => v.sort_by_key(|e| (std::cmp::Reverse(e.players()), ping(e), e.addr)),
            SortBy::Name => v.sort_by_key(|e| (name(e), e.addr)),
        }
        v
    }

    pub fn entry(&self, addr: SocketAddr) -> Option<&Entry> {
        self.entries.get(&addr)
    }

    /// Set an entry directly (tests and preview images).
    #[cfg(test)]
    pub fn insert(&mut self, entry: Entry) {
        self.entries.insert(entry.addr, entry);
    }
}

impl Browser<UdpSocket> {
    /// Open the socket if needed (broadcast allowed).
    fn ensure_probe(&mut self) {
        if self.probe.is_some() {
            return;
        }
        // IPv4 and IPv6: servers behind DS-Lite are often only reachable via IPv6
        match UdpSocket::bind_dual(0) {
            Ok(socket) => {
                if let Err(e) = socket.set_broadcast(true) {
                    tracing::warn!("broadcast not possible: {e}");
                }
                #[allow(clippy::cast_possible_truncation)]
                let seed = Instant::now().elapsed().as_nanos() as u64 ^ 0x9e37_79b9;
                self.probe = Some(InfoProbe::new(socket, seed));
            }
            Err(e) => self.status = Some(("browser.no_socket", e.to_string())),
        }
    }

    /// Reload the current tab.
    pub fn refresh(&mut self, master_url: &str, favorites: &[String], now: Instant) {
        self.ensure_probe();
        self.clear();
        self.status = None;
        match self.tab {
            Tab::Internet => {
                if master_url.trim().is_empty() {
                    return;
                }
                let url = master_url.trim().to_owned();
                let (tx, rx) = std::sync::mpsc::channel();
                let spawned = std::thread::Builder::new()
                    .name("master-list".into())
                    .spawn(move || {
                        let _ = tx.send(elora_master::client::fetch(&url));
                    });
                match spawned {
                    Ok(_) => self.master = Some(rx),
                    Err(e) => self.status = Some(("browser.master_failed", e.to_string())),
                }
            }
            Tab::Lan => {
                let mut targets: Vec<SocketAddr> = LAN_PORTS
                    .map(|p| SocketAddr::from(([255, 255, 255, 255], p)))
                    .collect();
                // the local machine does not always answer broadcasts
                targets.extend(LAN_PORTS.map(|p| SocketAddr::from(([127, 0, 0, 1], p))));
                self.discover(&targets, now);
            }
            Tab::Favorites => {
                let addrs = resolve(favorites);
                self.query(&addrs, now);
            }
        }
    }
}

/// Resolve addresses (also names like `server.example.org:8303`).
pub fn resolve(addresses: &[String]) -> Vec<SocketAddr> {
    addresses
        .iter()
        .filter_map(|a| {
            a.parse().ok().or_else(|| {
                std::net::ToSocketAddrs::to_socket_addrs(a.as_str())
                    .ok()?
                    .next()
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_net::{Conditions, Keypair, MemNetwork, MemSocket, ServerEndpoint};
    use elora_protocol::PROTOCOL_VERSION;

    fn addr(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    fn server(net: &MemNetwork, port: u16, name: &str, clients: u32) -> ServerEndpoint<MemSocket> {
        let mut s = ServerEndpoint::new(
            net.socket(addr(port), Conditions::default(), u64::from(port)),
            Keypair::generate(),
            8,
        );
        s.set_info(
            ServerInfo {
                version: PROTOCOL_VERSION,
                name: name.into(),
                map: "sandbox".into(),
                mode: "DM".into(),
                clients,
                max_clients: 8,
                players: vec![],
            }
            .encode(),
        );
        s
    }

    #[test]
    fn query_sort_filter_and_unreachable() {
        let net = MemNetwork::new();
        let mut a = server(&net, 8303, "Beta", 0);
        let mut b = server(&net, 8304, "Alpha", 8);
        let mut c = server(&net, 8305, "Gamma", 3);
        let probe = InfoProbe::new(net.socket(addr(9100), Conditions::default(), 9), 1);
        let mut br = Browser::new(Some(probe));
        let mut now = Instant::now();
        br.query(&[addr(8303), addr(8304), addr(8305), addr(8399)], now);
        assert!(br.busy());
        for _ in 0..60 {
            now += Duration::from_millis(50);
            a.poll(now);
            b.poll(now);
            c.poll(now);
            br.poll(now);
        }
        assert!(!br.busy());
        assert_eq!(br.entry(addr(8399)).unwrap().state, State::Unreachable);
        br.sort = SortBy::Name;
        let names: Vec<String> = br
            .visible()
            .iter()
            .filter_map(|e| match &e.state {
                State::Online { info, .. } => Some(info.name.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(names, ["Alpha", "Beta", "Gamma"]);
        br.sort = SortBy::Players;
        assert_eq!(br.visible()[0].addr, addr(8304), "most players first");
        br.filter = Filter {
            hide_empty: true,
            hide_full: true,
        };
        let shown: Vec<SocketAddr> = br.visible().iter().map(|e| e.addr).collect();
        assert!(
            shown.contains(&addr(8305))
                && !shown.contains(&addr(8303))
                && !shown.contains(&addr(8304))
        );
    }

    #[test]
    fn same_server_over_v4_and_v6_is_shown_once() {
        let net = MemNetwork::new();
        let v6 = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 8303));
        let mut a = server(&net, 8303, "Doppelt", 2);
        let mut b = ServerEndpoint::new(
            net.socket(v6, Conditions::default(), 77),
            Keypair::generate(),
            8,
        );
        b.set_info(
            ServerInfo {
                version: PROTOCOL_VERSION,
                name: "Doppelt".into(),
                map: "sandbox".into(),
                mode: "DM".into(),
                clients: 2,
                max_clients: 8,
                players: vec![],
            }
            .encode(),
        );
        let probe = InfoProbe::new(net.socket(addr(9101), Conditions::default(), 9), 1);
        let mut br = Browser::new(Some(probe));
        let mut now = Instant::now();
        br.query(&[addr(8303), v6], now);
        for _ in 0..60 {
            now += Duration::from_millis(50);
            a.poll(now);
            b.poll(now);
            br.poll(now);
        }
        assert_eq!(br.visible().len(), 1, "once instead of twice");
    }

    #[test]
    fn resolve_addresses() {
        assert_eq!(
            resolve(&["127.0.0.1:8303".into(), "none-port".into()]),
            vec![addr(8303)]
        );
    }
}
