//! HTTPS-Client zum Master (E-127): Spielserver melden sich an, Spiel-Clients holen
//! die Liste. Blockierend – im eigenen Thread aufrufen.

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context as _;

use crate::{RegisterReply, RegisterRequest, ServerList};

/// Antworten größer als das werden abgelehnt.
const MAX_BODY: u64 = 1024 * 1024;

fn agent() -> ureq::Agent {
    agent_for(ureq::config::IpFamily::Any)
}

fn agent_for(family: ureq::config::IpFamily) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .ip_family(family)
        // auch bei 4xx/5xx die Antwort lesen: der Master nennt dort den Grund
        .http_status_as_error(false)
        .build()
        .into()
}

fn endpoint(base: &str, path: &str) -> String {
    format!("{}/{path}", base.trim_end_matches('/'))
}

/// Spielserver beim Master anmelden (bzw. die Anmeldung erneuern).
///
/// Der Master prüft die Adresse, von der die Anmeldung kommt. Lauscht der Server nur auf
/// IPv4 (`ipv4_only`), geht die Anmeldung deshalb auch nur über IPv4 – sonst prüfte der
/// Master bei Hosts mit IPv6 die falsche Adresse.
///
/// # Errors
/// Bei Netzwerkfehlern oder ungültiger Antwort.
pub fn register(
    base: &str,
    port: u16,
    version: u32,
    ipv4_only: bool,
) -> anyhow::Result<RegisterReply> {
    let body = serde_json::to_string(&RegisterRequest { port, version })?;
    let family = if ipv4_only {
        ureq::config::IpFamily::Ipv4Only
    } else {
        ureq::config::IpFamily::Any
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

/// Adressen aller gelisteten Server.
///
/// # Errors
/// Bei Netzwerkfehlern oder ungültiger Antwort.
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
