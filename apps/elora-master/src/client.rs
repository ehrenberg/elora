//! HTTPS-Client zum Master (E-127): Spielserver melden sich an, Spiel-Clients holen
//! die Liste. Blockierend – im eigenen Thread aufrufen.

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context as _;

use crate::{RegisterReply, RegisterRequest, ServerList};

/// Antworten größer als das werden abgelehnt.
const MAX_BODY: u64 = 1024 * 1024;

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build()
        .into()
}

fn endpoint(base: &str, path: &str) -> String {
    format!("{}/{path}", base.trim_end_matches('/'))
}

/// Spielserver beim Master anmelden (bzw. die Anmeldung erneuern).
///
/// # Errors
/// Bei Netzwerkfehlern oder ungültiger Antwort.
pub fn register(base: &str, port: u16, version: u32) -> anyhow::Result<RegisterReply> {
    let body = serde_json::to_string(&RegisterRequest { port, version })?;
    let mut resp = agent()
        .post(&endpoint(base, "register"))
        .header("Content-Type", "application/json")
        .send(&body)
        .with_context(|| format!("Master {base} nicht erreichbar"))?;
    let text = resp
        .body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_string()?;
    Ok(serde_json::from_str(&text)?)
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
    let text = resp
        .body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_string()?;
    let list: ServerList = serde_json::from_str(&text)?;
    Ok(list.servers.iter().filter_map(|s| s.parse().ok()).collect())
}
