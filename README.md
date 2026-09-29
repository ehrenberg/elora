# Elora

Ein 2D-Multiplayer-Arena-Shooter nach dem Vorbild von [Teeworlds](https://teeworlds.com) – eigene Figur, eigener Stil, gleiches Spielgefühl. Elora ist zugleich der Name der spielbaren Figur.

**Stand:** M0–M4 abgeschlossen (Bewegung, Hook, Waffen, Dummies, Netzwerk, Spielmodi), M5 (Look & Sound) in Arbeit: Figur, Skins, Welt, Effekte, HUD und Emotes stehen, Sound folgt. Siehe [Roadmap](docs/06-roadmap.md).

## Einrichtung unter Arch Linux

### 1. Pakete

```sh
sudo pacman -S --needed git base-devel rustup cargo-deny cargo-nextest
rustup default stable
```

- `rustup` ersetzt das Paket `rust` (pacman fragt nach – mit Ja bestätigen). Die genaue Rust-Version (siehe `rust-toolchain.toml`) lädt rustup beim ersten `cargo`-Aufruf im Projekt automatisch.
- `cargo-deny` (Lizenz- und Sicherheitsprüfung) und `cargo-nextest` (Test-Runner) braucht `cargo xtask check`.

### 2. Grafik (Vulkan) und Fenster

Der Renderer nutzt wgpu, unter Linux standardmäßig Vulkan. Nötig sind der Vulkan-Loader und der Treiber zur Grafikkarte:

```sh
sudo pacman -S --needed vulkan-icd-loader
sudo pacman -S --needed vulkan-radeon    # AMD
# oder: vulkan-intel                     # Intel
# oder: nvidia-utils                     # NVIDIA (proprietär)
```

Fenster laufen unter Wayland und X11 (auf Desktop-Installationen meist schon vorhanden, sonst: `sudo pacman -S --needed wayland libxkbcommon libx11 libxcursor libxrandr libxi`).

Prüfen: `vulkaninfo --summary` (Paket `vulkan-tools`) sollte die Grafikkarte zeigen.

### 3. Bauen

```sh
git clone <repo-url> elora && cd elora
cargo build            # baut Client (elora) und Server (elora-server)
```

Alle Befehle werden aus dem Projektordner ausgeführt (Karten liegen in `maps/`).

## Spielen

### Sandbox (lokal, ohne Netzwerk)

```sh
cargo run --bin elora                               # Standardkarte maps/sandbox.emap.toml
cargo run --bin elora -- maps/eigene.emap.toml      # andere Karte
cargo run --bin elora -- maps/ctf-test.emap.toml --mode ctf   # Spielmodus gegen Dummies
```

Spielmodi in der Sandbox: `--mode dm|tdm|ctf|lms|lts` (optional `--instagib`) oder im Panel unter *Spiel → Modus*. CTF braucht eine Karte mit Flaggen, z. B. `maps/ctf-test.emap.toml`.

Die Sandbox dient zum Tunen und Testen: Dummies, Pickups, Live-Regler für alle Werte, Hot-Reload der Karte.

### Online

**Server starten:**

```sh
cargo run --bin elora-server
cargo run --bin elora-server -- --port 8303 --map maps/sandbox.emap.toml --max-clients 16 --name "Mein Server"
cargo run --bin elora-server -- --config server.toml
```

| Option | Standard | Bedeutung |
|---|---|---|
| `--port` | 8303 | UDP-Port |
| `--bind` | 0.0.0.0 | Adresse, an die gebunden wird |
| `--map` | `maps/sandbox.emap.toml` | Karte (Textformat) |
| `--max-clients` | 8 | Spieler, 1–64 |
| `--name` | Elora-Server | Anzeigename |
| `--high-bandwidth` | aus | Snapshots mit 50 statt 25 Hz (nur LAN) |
| `--key-file` | `server_key.toml` | dauerhafter Server-Schlüssel (wird beim ersten Start erzeugt) |
| `--tuning` | – | Tuning-Datei (Abschnitt `[physics]` wie `tuning.toml`) |
| `--mode` | dm | Spielmodus: `dm`, `tdm`, `ctf`, `lms`, `lts` |
| `--instagib` | aus | nur Laser, ein Treffer tötet (für alle Modi) |
| `--score-limit` | 20 (CTF: 5 Eroberungen) | Siegpunkte, 0 = aus |
| `--time-limit` | 0 | Zeitlimit in Minuten, 0 = aus |
| `--no-friendly-fire` | an | kein Schaden an Teammitgliedern (Rückstoß bleibt) |
| `--no-votes` | an | Abstimmungen abschalten |
| `--config` | – | alle Optionen aus einer TOML-Datei; weitere Optionen überschreiben sie |

Weitere Einstellungen nur in der Konfigurationsdatei: `rotation = ["sandbox", "ctf-test"]` (Kartenrotation), `maps_dir`, und ein Abschnitt `[rules]` (`warmup_secs`, `countdown_secs`, `tdm_respawn_secs`, `team_balance_secs`, `match_swap`, `matches_per_map`).

**Konsole:** Im Terminal des Servers Befehle eintippen – `help` zeigt alle: `status`, `mode ctf`, `instagib on`, `scorelimit 10`, `timelimit 5`, `friendlyfire off`, `restart`, `map <name>`, `maps`, `kick <slot>`, `ban <slot>`, `spectate <slot>`, `say <text>`, `vote cancel`, `quit`.

Beenden: `quit` + Enter oder Strg+C. Log-Ausgabe steuern: `RUST_LOG=debug cargo run --bin elora-server`.

**Verbinden:**

```sh
cargo run --bin elora -- --connect 127.0.0.1:8303
```

oder im Client: `Esc` → Panel *Netzwerk* → Adresse eintragen → *Verbinden*. *Trennen* führt zurück in die Sandbox.

**Aus dem Client hosten:** `Esc` → *Netzwerk* → *Server einrichten …* → Name, Port, Karte, Spielerzahl, 50 Hz, „weiterlaufen lassen“ → *Starten und verbinden*. Der Client schreibt `server.toml` und startet `elora-server` als eigenen Prozess (vorher einmal `cargo build`).

**Firewall:** Für Spieler aus dem LAN/Internet muss der UDP-Port (Standard 8303) freigegeben sein, z. B. `sudo ufw allow 8303/udp` bzw. `sudo firewall-cmd --add-port=8303/udp`.

**Sicherheit:** Die Verbindung ist verschlüsselt (Noise-Protokoll). Der Client merkt sich den Schlüssel jedes Servers in `known_servers.toml` und warnt, wenn er sich ändert (wie SSH).

### Steuerung

| Taste | Aktion |
|---|---|
| A / D | laufen |
| Leertaste | springen, in der Luft Doppelsprung |
| Rechte Maustaste (halten) | Hook |
| Linke Maustaste | schießen (Granate/Laser: gedrückt halten = Dauerfeuer) |
| 1 / 2 / 3, Mausrad | Hammer / Granate / Laser |
| Esc | Maus freigeben (Panel bedienen); erneut Esc = beenden |
| F1 | Debug-Panel ein/aus |
| R | Respawn (nur Sandbox ohne Modus) |
| K | Selbstmord (`kill`) |
| T / Y | Chat / Team-Chat (Enter senden, Esc abbrechen; online) |
| Tab (halten) | Scoreboard |
| E (halten) | Emote-Rad: Maus in Richtung des Emotes, loslassen zeigt es |
| F3 / F4 | Ja / Nein bei Abstimmungen |
| F5 | Aufzeichnung starten/beenden (nur Sandbox, siehe unten) |

Nach dem Tod: Feuertaste = Respawn (frühestens nach 0,5 s, TDM 3 s), sonst automatisch nach 3 s. In LMS/LTS kein Respawn bis zur nächsten Runde.

Team wählen, zuschauen und Abstimmungen (Karte, Modus, Kick, Zuschauer) starten: `Esc` → Panel *Spiel*.

## Dateien im Arbeitsverzeichnis

| Datei | Zweck | im Git |
|---|---|---|
| `tuning.toml` | gespeicherte Regler-Werte der Sandbox und Effekt-Schalter (Panel → *Speichern*) | nein, lokal |
| `server.toml` | vom Client geschriebene Server-Konfiguration | nein |
| `server_key.toml` | geheimer Server-Schlüssel – nicht weitergeben | nein |
| `known_servers.toml` | bekannte Server-Schlüssel des Clients | nein |
| `maps/*.emap.toml` | Test-/Entwicklungskarten ([Format](docs/05-kartenformat.md)) | ja |

## Entwicklung

```sh
cargo xtask check          # alles: Formatierung, clippy, Tests, Lizenzen & Advisories
cargo xtask fmt            # Code formatieren
cargo nextest run --workspace --all-features   # nur Tests
```

Nach jeder Änderung muss `cargo xtask check` grün sein, dann wird committet.

**Determinismus-Tests (Golden-Dateien):** Die Simulation muss bit-genau reproduzierbar bleiben. In der Sandbox zeichnet `F5` die eigenen Eingaben auf; die Datei landet in `crates/elora-sim/tests/recordings/` und wird zum Regressionstest. Golden-Dateien (neu) erzeugen – nur nach bewusster Physik-Änderung:

```sh
ELORA_BLESS=1 cargo nextest run -p elora-sim --all-features
```

**Netzwerk-Kompression:**

```sh
cargo xtaska net-stats        # Nachrichtengrößen für 8/16/64 Spieler messen
cargo xtask train-huffman    # Huffman-Tabelle neu trainieren (nach Änderungen am Snapshot-Format)
```

### Projektstruktur

```
crates/elora-sim       deterministische Simulation (Physik, Hook, Waffen, Dummies)
crates/elora-map       Kartenformat und Aufbau der Welt aus einer Karte
crates/elora-render    wgpu-2D-Renderer (lyon-Tessellierung), Kamera
crates/elora-protocol  Nachrichten, Delta-Snapshots, Huffman
crates/elora-net       UDP-Transport: Handshake, Verschlüsselung, Zuverlässigkeit, Simulator
crates/elora-game      Spielregeln: Modi, Punkte, Runden, Teams, Flaggen
apps/elora-client      das Spiel (Sandbox + Online)
apps/elora-server      dedizierter Server
xtask                  Entwicklungsbefehle (cargo xtask …)
maps/                  Textkarten
assets/                Schriften (später Grafik und Sound)
docs/                  Analyse, Entscheidungen, Architektur, Pläne
```

Details: [Architektur](docs/03-architektur.md).

## Fehlerbehebung

| Problem | Lösung |
|---|---|
| „kein passender Grafikadapter“ / Fenster startet nicht | Vulkan-Treiber installieren (siehe oben); notfalls OpenGL erzwingen: `WGPU_BACKEND=gl cargo run --bin elora` |
| `elora-server … nicht gefunden` beim Hosten aus dem Client | einmal `cargo build` (baut beide Programme) |
| `Port … nicht verfügbar` | anderer Server läuft bereits auf dem Port → `--port` ändern |
| Warnung „Server-Schlüssel geändert“ | Server wurde neu eingerichtet (neuer `server_key.toml`) – oder ein Angriff. Nur vertrauen, wenn du den Grund kennst. |
| Maus lässt sich nicht fangen | ins Spielfeld klicken; unter manchen Wayland-Umgebungen wird statt „gesperrt“ „begrenzt“ genutzt |
| `cargo xtask check` meldet fehlende Befehle | `cargo-deny` und `cargo-nextest` installieren (siehe Einrichtung) |

## Dokumentation

Siehe [`docs/`](docs/README.md) – Analyse des Originals, Entscheidungslog, Architektur, Tuning, Kartenformat, Roadmap und die Umsetzungspläne der Meilensteine.

## Lizenz

- Code: [GPL-3.0](LICENSE)
- Grafiken & Sounds: CC-BY-SA 4.0
- Schriften (Inter, JetBrains Mono): SIL Open Font License 1.1
- Drittanbieter: [THIRD_PARTY_LICENSES](THIRD_PARTY_LICENSES)
