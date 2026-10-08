//! HTTPS client for the master (E-127): game servers register, game clients fetch the
//! list. Blocking – call it in a separate thread.

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context as _;

use crate::{RegisterReply, RegisterRequest, ServerList};

/// Replies larger than this are rejected.
const MAX_BODY: u64 = 1024 * 1024;

fn agent() -> ureq::Agent {
    agent_for(ureq::config::IpFamily::Any)
}

/// Over which address family a game server registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Any,
    V4,
    V6,
}

fn agent_for(family: ureq::config::IpFamily) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .ip_family(family)
        // read the reply even on 4xx/5xx: the master states the reason there
        .http_status_as_error(false)
        .build()
        .into()
}

fn endpoint(base: &str, path: &str) -> String {
    format!("{}/{path}", base.trim_end_matches('/'))
}

/// Register a game server with the master (or renew the registration).
///
/// The master checks the address the registration comes from. A server therefore registers
/// over the families it listens on (once each for IPv4 and IPv6) – otherwise the master
/// would check the wrong address.
///
/// # Errors
/// On network errors or an invalid reply.
pub fn register(
    base: &str,
    port: u16,
    version: u32,
    family: Family,
) -> anyhow::Result<RegisterReply> {
    let body = serde_json::to_string(&RegisterRequest { port, version })?;
    let family = match family {
        Family::Any => ureq::config::IpFamily::Any,
        Family::V4 => ureq::config::IpFamily::Ipv4Only,
        Family::V6 => ureq::config::IpFamily::Ipv6Only,
    };
    let mut resp = agent_for(family)
        .post(&endpoint(base, "register"))
        .header("Content-Type", "application/json")
        .send(&body)
        .with_context(|| format!("Master {base} nicht erreichbar"))?;
    let status = resp.status();
    let text = resp
        .body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_string()?;
    serde_json::from_str(&text).with_context(|| format!("Master {base}: HTTP {status}"))
}

/// Addresses of all listed servers.
///
/// # Errors
/// On network errors or an invalid reply.
pub fn fetch(base: &str) -> anyhow::Result<Vec<SocketAddr>> {
    let mut resp = agent()
        .get(&endpoint(base, "servers"))
        .call()
        .with_context(|| format!("Master {base} nicht erreichbar"))?;
    anyhow::ensure!(
        resp.status().is_success(),
        "Master {base}: HTTP {}",
        resp.status()
    );
    let text = resp
        .body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_string()?;
    let list: ServerList = serde_json::from_str(&text)?;
    Ok(list.servers.iter().filter_map(|s| s.parse().ok()).collect())
}
