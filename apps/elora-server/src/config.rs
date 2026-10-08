//! Server configuration (file `server.toml` and command line) and server key.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use elora_game::{Mode, RulesConfig};
use elora_net::Keypair;
use serde::{Deserialize, Serialize};

/// Default port (as in the original).
pub const DEFAULT_PORT: u16 = 8303;
/// Technical maximum of players (E-059).
pub const MAX_CLIENTS: usize = 64;

/// Default master for the internet list (E-166); dedicated servers register there (E-170).
pub const DEFAULT_MASTER: &str = "https://elora.bastianswelt.de";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ServerConfig {
    /// Display name of the server.
    pub name: String,
    /// Address to bind to: `::` = IPv4 and IPv6 (default), `0.0.0.0` = IPv4 only,
    /// otherwise a specific address.
    pub bind: String,
    pub port: u16,
    /// Map in text format.
    pub map: PathBuf,
    /// Maximum number of players (1..=64, default as in the original: 8).
    pub max_clients: usize,
    /// Snapshots every tick (50 Hz) instead of every second one (25 Hz) – LAN only (E-059).
    pub high_bandwidth: bool,
    /// File with the persistent server key (E-062); created if needed.
    pub key_file: PathBuf,
    /// Optional tuning file (format like the client's `tuning.toml`, section `[physics]`).
    pub tuning: Option<PathBuf>,
    /// Map rotation (names without `.emap` from `maps_dir`); empty = only `map` (E-074).
    pub rotation: Vec<String>,
    /// Directory with maps for rotation and votes.
    pub maps_dir: PathBuf,
    /// Allow votes (E-077).
    pub votes: bool,
    /// Game rules (E-066 to E-078).
    pub rules: RulesConfig,
    /// Master servers the server registers with (HTTPS addresses, E-112/E-127);
    /// empty = not in the internet list, only LAN and direct connect.
    pub masters: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "Elora-Server".into(),
            bind: "::".into(),
            port: DEFAULT_PORT,
            map: PathBuf::from("maps/dm-wiese.emap"),
            max_clients: 8,
            high_bandwidth: false,
            key_file: PathBuf::from("server_key.toml"),
            tuning: None,
            rotation: Vec::new(),
            maps_dir: PathBuf::from("maps"),
            votes: true,
            rules: RulesConfig::default(),
            masters: vec![DEFAULT_MASTER.into()],
        }
    }
}

impl ServerConfig {
    /// Loads the configuration from a TOML file.
    ///
    /// # Errors
    /// If the file is unreadable or invalid.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let src = std::fs::read_to_string(path)
            .with_context(|| format!("{} not readable", path.display()))?;
        toml::from_str(&src).with_context(|| format!("{} is invalid", path.display()))
    }

    /// # Errors
    /// If the file cannot be written.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        std::fs::write(path, toml::to_string_pretty(self)?)
            .with_context(|| format!("{} not writable", path.display()))
    }

    /// Applies command-line arguments (`--port 8303`, `--map path`, …).
    ///
    /// # Errors
    /// On unknown arguments or invalid values.
    pub fn apply_args(&mut self, args: &[String]) -> anyhow::Result<()> {
        let mut it = args.iter();
        while let Some(arg) = it.next() {
            let mut value = || it.next().with_context(|| format!("{arg} expects a value"));
            match arg.as_str() {
                "--name" => self.name.clone_from(value()?),
                "--bind" => self.bind.clone_from(value()?),
                "--port" => self.port = value()?.parse().context("--port")?,
                "--map" => self.map = PathBuf::from(value()?),
                "--max-clients" => self.max_clients = value()?.parse().context("--max-clients")?,
                "--high-bandwidth" => self.high_bandwidth = true,
                "--key-file" => self.key_file = PathBuf::from(value()?),
                "--tuning" => self.tuning = Some(PathBuf::from(value()?)),
                "--mode" => {
                    let v = value()?;
                    self.rules.mode = Mode::parse(v)
                        .with_context(|| format!("unknown mode `{v}` (dm, tdm, ctf, lms, lts)"))?;
                }
                "--instagib" => self.rules.instagib = true,
                "--score-limit" => {
                    self.rules.score_limit = Some(value()?.parse().context("--score-limit")?);
                }
                "--time-limit" => {
                    self.rules.time_limit = value()?.parse().context("--time-limit")?;
                }
                "--no-friendly-fire" => self.rules.friendly_fire = false,
                "--no-votes" => self.votes = false,
                // a custom master replaces the default; multiple `--master` are possible
                "--master" => {
                    if self.masters == [DEFAULT_MASTER] {
                        self.masters.clear();
                    }
                    self.masters.push(value()?.clone());
                }
                "--no-master" => self.masters.clear(),
                "--config" => {
                    value()?; // already evaluated in main
                }
                other => anyhow::bail!("unknown argument `{other}`"),
            }
        }
        self.validate()
    }

    /// # Errors
    /// On values outside the allowed ranges.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            (1..=MAX_CLIENTS).contains(&self.max_clients),
            "max_clients must be 1..={MAX_CLIENTS}"
        );
        anyhow::ensure!(
            !self.name.trim().is_empty(),
            "server name must not be empty"
        );
        Ok(())
    }

    /// Does the server listen on IPv4 and IPv6 (`bind = "::"`)?
    pub fn dual_stack(&self) -> bool {
        matches!(self.bind.trim(), "::" | "[::]" | "")
    }

    /// # Errors
    /// On an invalid address.
    pub fn addr(&self) -> anyhow::Result<SocketAddr> {
        let host = self
            .bind
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']');
        let host = if host.contains(':') {
            format!("[{host}]")
        } else {
            host.to_owned()
        };
        format!("{host}:{}", self.port)
            .parse()
            .context("invalid bind address")
    }
}

#[derive(Serialize, Deserialize)]
struct KeyFile {
    public: String,
    private: String,
}

fn unhex(s: &str) -> anyhow::Result<Vec<u8>> {
    anyhow::ensure!(s.len().is_multiple_of(2), "odd hex length");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).context("invalid hex"))
        .collect()
}

/// Loads the server key or creates and stores a new one.
///
/// # Errors
/// If the key file is invalid or not writable.
pub fn load_or_create_key(path: &Path) -> anyhow::Result<Keypair> {
    if path.exists() {
        let src = std::fs::read_to_string(path)?;
        let f: KeyFile =
            toml::from_str(&src).with_context(|| format!("{} is invalid", path.display()))?;
        return Ok(Keypair {
            public: unhex(&f.public)?,
            private: unhex(&f.private)?,
        });
    }
    let kp = Keypair::generate();
    let f = KeyFile {
        public: elora_net::hex(&kp.public),
        private: elora_net::hex(&kp.private),
    };
    let text = format!(
        "# Elora – persistent server key (E-062). Keep it secret!\n\
         # Clients remember the public part and warn if it changes.\n{}",
        toml::to_string(&f)?
    );
    std::fs::write(path, text).with_context(|| format!("{} not writable", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    tracing::info!(key = %f.public, "new server key created");
    Ok(kp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_override_defaults() {
        let mut c = ServerConfig::default();
        let args: Vec<String> = ["--port", "9000", "--max-clients", "64", "--high-bandwidth"]
            .map(String::from)
            .to_vec();
        c.apply_args(&args).unwrap();
        assert_eq!((c.port, c.max_clients, c.high_bandwidth), (9000, 64, true));
        let bad: Vec<String> = ["--max-clients", "65"].map(String::from).to_vec();
        assert!(ServerConfig::default().apply_args(&bad).is_err());
        assert!(
            ServerConfig::default()
                .apply_args(&["--foo".into()])
                .is_err()
        );
    }

    #[test]
    fn config_roundtrip() {
        let c = ServerConfig {
            name: "Test".into(),
            max_clients: 16,
            ..ServerConfig::default()
        };
        let text = toml::to_string_pretty(&c).unwrap();
        assert_eq!(toml::from_str::<ServerConfig>(&text).unwrap(), c);
    }
}
