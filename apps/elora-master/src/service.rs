//! The master service: HTTP interface, UDP check of the servers, list.

use std::io::Read as _;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use elora_net::{InfoProbe, Socket};
use elora_protocol::ServerInfo;

use crate::registry::Registry;
use crate::{RegisterReply, RegisterRequest, ServerList};

/// Largest accepted request.
const MAX_REQUEST: u64 = 4096;

/// Logic of the service without HTTP (testable with the in-memory network).
#[derive(Debug)]
pub struct Master<S: Socket> {
    pub registry: Registry,
    probe: InfoProbe<S>,
}

impl<S: Socket> Master<S> {
    pub fn new(probe_socket: S) -> Self {
        Self {
            registry: Registry::default(),
            probe: InfoProbe::new(probe_socket, 0x5eed),
        }
    }

    /// `POST /register` from `ip` with JSON `body`. Returns HTTP status and reply.
    pub fn register(&mut self, ip: IpAddr, body: &str, now: Instant) -> (u16, RegisterReply) {
        let reply = |ok, msg: &str| RegisterReply {
            ok,
            message: msg.to_owned(),
        };
        let Ok(req) = serde_json::from_str::<RegisterRequest>(body) else {
            return (400, reply(false, "invalid request"));
        };
        if req.version != elora_protocol::PROTOCOL_VERSION {
            return (400, reply(false, "wrong protocol version"));
        }
        if req.port == 0 {
            return (400, reply(false, "invalid port"));
        }
        let addr = SocketAddr::new(ip, req.port);
        match self.registry.request(addr, now) {
            Ok(()) => {
                self.probe.query(addr, now);
                (202, reply(true, "check running"))
            }
            Err(e) => (429, reply(false, e.message())),
        }
    }

    /// `GET /servers` as JSON.
    pub fn list_json(&self) -> String {
        let list = ServerList {
            servers: self
                .registry
                .list()
                .iter()
                .map(ToString::to_string)
                .collect(),
        };
        serde_json::to_string(&list).unwrap_or_else(|_| "{\"servers\":[]}".into())
    }

    /// Evaluate UDP replies, remove expired entries.
    pub fn tick(&mut self, now: Instant) {
        let (replies, lost) = self.probe.poll(now);
        for r in replies {
            let ok = ServerInfo::decode(&r.data).is_ok_and(|i| i.compatible());
            if !ok {
                tracing::info!(addr = %r.addr, "server answers with wrong version");
            }
            self.registry.verified(r.addr, ok, now);
        }
        for addr in lost {
            tracing::info!(%addr, "server not reachable – not listed");
        }
        self.registry.expire(now);
    }
}

/// Sender of a request: direct or – behind a reverse proxy – from `X-Forwarded-For`.
fn client_ip(req: &tiny_http::Request, behind_proxy: bool) -> Option<IpAddr> {
    if behind_proxy {
        let header = req
            .headers()
            .iter()
            .find(|h| h.field.equiv("X-Forwarded-For"))?;
        // first address = original sender
        return header.value.as_str().split(',').next()?.trim().parse().ok();
    }
    req.remote_addr().map(SocketAddr::ip)
}

fn json(status: u16, body: String) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
        .expect("valid header");
    tiny_http::Response::from_string(body)
        .with_status_code(status)
        .with_header(header)
}

/// Start the service and keep it running until shutdown.
///
/// # Errors
/// If the port or UDP socket cannot be opened.
pub fn run(bind: SocketAddr, behind_proxy: bool) -> anyhow::Result<()> {
    let http = tiny_http::Server::http(bind).map_err(|e| anyhow::anyhow!("{bind}: {e}"))?;
    let udp = elora_net::UdpSocket::bind_dual(0)?;
    let mut master = Master::new(udp);
    tracing::info!(%bind, behind_proxy, "master running");
    loop {
        if let Some(mut req) = http.recv_timeout(Duration::from_millis(50))? {
            let now = Instant::now();
            let method = req.method().clone();
            let url = req.url().to_owned();
            let response = match (method, url.as_str()) {
                (tiny_http::Method::Get, "/servers") => json(200, master.list_json()),
                (tiny_http::Method::Post, "/register") => {
                    let mut body = String::new();
                    let read = req.as_reader().take(MAX_REQUEST).read_to_string(&mut body);
                    match (read, client_ip(&req, behind_proxy)) {
                        (Ok(_), Some(ip)) => {
                            let (status, reply) = master.register(ip, &body, now);
                            json(status, serde_json::to_string(&reply)?)
                        }
                        _ => json(400, "{\"ok\":false,\"message\":\"invalid request\"}".into()),
                    }
                }
                (tiny_http::Method::Get, "/") => json(200, "{\"service\":\"elora-master\"}".into()),
                _ => json(404, "{\"ok\":false,\"message\":\"unknown\"}".into()),
            };
            if let Err(e) = req.respond(response) {
                tracing::debug!("response not sent: {e}");
            }
        }
        master.tick(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_net::{Conditions, Keypair, MemNetwork, ServerEndpoint};
    use elora_protocol::PROTOCOL_VERSION;

    fn addr(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    fn info(version: u32) -> Vec<u8> {
        ServerInfo {
            version,
            name: "Test".into(),
            map: "sandbox".into(),
            mode: "DM".into(),
            clients: 0,
            max_clients: 8,
            players: vec![],
        }
        .encode()
    }

    #[test]
    fn reachable_servers_are_listed_others_not() {
        let net = MemNetwork::new();
        let mut good = ServerEndpoint::new(
            net.socket(addr(8303), Conditions::default(), 1),
            Keypair::generate(),
            8,
        );
        let mut old = ServerEndpoint::new(
            net.socket(addr(8304), Conditions::default(), 2),
            Keypair::generate(),
            8,
        );
        good.set_info(info(PROTOCOL_VERSION));
        old.set_info(info(PROTOCOL_VERSION + 99));
        let mut m = Master::new(net.socket(addr(9999), Conditions::default(), 3));
        let ip = addr(0).ip();
        let mut now = Instant::now();
        let body = |port| format!("{{\"port\":{port},\"version\":{PROTOCOL_VERSION}}}");
        assert_eq!(m.register(ip, &body(8303), now).0, 202);
        assert_eq!(m.register(ip, &body(8304), now).0, 202);
        assert_eq!(
            m.register(ip, &body(8305), now).0,
            202,
            "nobody listens on 8305"
        );
        assert_eq!(m.register(ip, &body(8303), now).0, 429, "too often");
        assert_eq!(m.register(ip, "no json", now).0, 400);
        assert_eq!(
            m.register(ip, "{\"port\":1,\"version\":1}", now).0,
            400,
            "Version"
        );
        for _ in 0..50 {
            now += Duration::from_millis(50);
            good.poll(now);
            old.poll(now);
            m.tick(now);
        }
        assert_eq!(m.list_json(), "{\"servers\":[\"127.0.0.1:8303\"]}");
    }
}
