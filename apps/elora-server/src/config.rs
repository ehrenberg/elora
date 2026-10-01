//! Server-Konfiguration (Datei `server.toml` und Kommandozeile) und Server-Schlüssel.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use elora_game::{Mode, RulesConfig};
use elora_net::Keypair;
use serde::{Deserialize, Serialize};

/// Standard-Port (wie im Original).
pub const DEFAULT_PORT: u16 = 8303;
/// Technisches Maximum an Spielern (E-059).
pub const MAX_CLIENTS: usize = 64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ServerConfig {
    /// Anzeigename des Servers.
    pub name: String,
    /// Adresse, an die gebunden wird (z. B. `0.0.0.0`).
    pub bind: String,
    pub port: u16,
    /// Karte im Textformat.
    pub map: PathBuf,
    /// Maximale Spielerzahl (1..=64, Standard wie Original: 8).
    pub max_clients: usize,
    /// Snapshots jeden Tick (50 Hz) statt jeden zweiten (25 Hz) – nur für LAN (E-059).
    pub high_bandwidth: bool,
    /// Datei mit dem dauerhaften Server-Schlüssel (E-062); wird bei Bedarf erzeugt.
    pub key_file: PathBuf,
    /// Optionale Tuning-Datei (Format wie `tuning.toml` des Clients, Abschnitt `[physics]`).
    pub tuning: Option<PathBuf>,
    /// Kartenrotation (Namen ohne `.emap` aus `maps_dir`); leer = nur `map` (E-074).
    pub rotation: Vec<String>,
    /// Verzeichnis mit Karten für Rotation und Abstimmungen.
    pub maps_dir: PathBuf,
    /// Abstimmungen erlauben (E-077).
    pub votes: bool,
    /// Spielregeln (E-066 bis E-078).
    pub rules: RulesConfig,
    /// Master-Server, bei denen sich der Server anmeldet (HTTPS-Adressen, E-112/E-127);
    /// leer = nicht in der Internet-Liste, nur LAN und Direkt-Verbinden.
    pub masters: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "Elora-Server".into(),
            bind: "0.0.0.0".into(),
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
            masters: Vec::new(),
        }
    }
}

impl ServerConfig {
    /// Lädt die Konfiguration aus einer TOML-Datei.
    ///
    /// # Errors
    /// Bei nicht lesbarer oder ungültiger Datei.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let src = std::fs::read_to_string(path)
            .with_context(|| format!("{} nicht lesbar", path.display()))?;
        toml::from_str(&src).with_context(|| format!("{} ist ungültig", path.display()))
    }

    /// # Errors
    /// Wenn die Datei nicht geschrieben werden kann.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        std::fs::write(path, toml::to_string_pretty(self)?)
            .with_context(|| format!("{} nicht schreibbar", path.display()))
    }

    /// Übernimmt Kommandozeilen-Argumente (`--port 8303`, `--map pfad`, …).
    ///
    /// # Errors
    /// Bei unbekannten Argumenten oder ungültigen Werten.
    pub fn apply_args(&mut self, args: &[String]) -> anyhow::Result<()> {
        let mut it = args.iter();
        while let Some(arg) = it.next() {
            let mut value = || {
                it.next()
                    .with_context(|| format!("{arg} erwartet einen Wert"))
            };
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
                    self.rules.mode = Mode::parse(v).with_context(|| {
                        format!("unbekannter Modus `{v}` (dm, tdm, ctf, lms, lts)")
                    })?;
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
                "--master" => self.masters.push(value()?.clone()),
                "--config" => {
                    value()?; // bereits in main ausgewertet
                }
                other => anyhow::bail!("unbekanntes Argument `{other}`"),
            }
        }
        self.validate()
    }

    /// # Errors
    /// Bei Werten außerhalb der erlaubten Bereiche.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            (1..=MAX_CLIENTS).contains(&self.max_clients),
            "max_clients muss 1..={MAX_CLIENTS} sein"
        );
        anyhow::ensure!(
            !self.name.trim().is_empty(),
            "Servername darf nicht leer sein"
        );
        Ok(())
    }

    /// # Errors
    /// Bei ungültiger Adresse.
    pub fn addr(&self) -> anyhow::Result<SocketAddr> {
        format!("{}:{}", self.bind, self.port)
            .parse()
            .context("ungültige Bind-Adresse")
    }
}

#[derive(Serialize, Deserialize)]
struct KeyFile {
    public: String,
    private: String,
}

fn unhex(s: &str) -> anyhow::Result<Vec<u8>> {
    anyhow::ensure!(s.len().is_multiple_of(2), "ungerade Hex-Länge");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).context("ungültiges Hex"))
        .collect()
}

/// Lädt den Server-Schlüssel oder erzeugt und speichert einen neuen.
///
/// # Errors
/// Bei ungültiger oder nicht schreibbarer Schlüsseldatei.
pub fn load_or_create_key(path: &Path) -> anyhow::Result<Keypair> {
    if path.exists() {
        let src = std::fs::read_to_string(path)?;
        let f: KeyFile =
            toml::from_str(&src).with_context(|| format!("{} ist ungültig", path.display()))?;
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
        "# Elora – dauerhafter Server-Schlüssel (E-062). Geheim halten!\n\
         # Clients merken sich den öffentlichen Teil; bei Änderung warnen sie.\n{}",
        toml::to_string(&f)?
    );
    std::fs::write(path, text).with_context(|| format!("{} nicht schreibbar", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    tracing::info!(key = %f.public, "neuer Server-Schlüssel erzeugt");
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
