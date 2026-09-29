# Elora

Ein 2D-Multiplayer-Arena-Shooter nach dem Vorbild von [Teeworlds](https://teeworlds.com) – eigene Figur, eigener Stil, gleiches Spielgefühl. Elora ist zugleich der Name der spielbaren Figur.

**Stand:** M0–M5 abgeschlossen (Bewegung, Hook, Waffen, Dummies, Netzwerk, Spielmodi, Look & Sound). Als Nächstes M7 (Menüs & Infrastruktur, vor M6 laut E-111). Siehe [Roadmap](docs/06-roadmap.md).

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

### 3. Ton

Die Tonausgabe läuft über ALSA (`alsa-lib`, auf Desktop-Installationen vorhanden; mit PipeWire zusätzlich `pipewire-alsa`):

```sh
sudo pacman -S --needed alsa-lib pipewire-alsa
```

Ohne Audiogerät startet das Spiel trotzdem – stumm, mit Hinweis im Panel unter *Ton*.

### 4. Bauen

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
| Esc | Pause-Menü (Fortsetzen, Hauptmenü, Beenden) |
| F1 | Debug-Panel ein/aus (auch im Menü) |
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
| `settings.toml` | Spieler-Einstellungen: Name, Skin, Sprache, Grafik, Ton, Effekte, Maus, Favoriten – **im Benutzerverzeichnis** (Linux `~/.config/elora/`, Windows `%APPDATA%\Elora\`, macOS `~/Library/Application Support/Elora/`), beim Beenden gespeichert | nein, lokal |
| `tuning.toml` | Entwickler-Tuning der Sandbox: Physik und Sichtbereich (Panel → *Speichern*) | nein, lokal |
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
cargo xtask net-stats         # Nachrichtengrößen für 8/16/64 Spieler messen
cargo xtask train-huffman    # Huffman-Tabelle neu trainieren (nach Änderungen am Snapshot-Format)
cargo xtask sound-preview    # alle Sounds als WAV nach target/sounds/ (Hörprobe; Parameter in assets/sounds/sounds.toml)
cargo xtask svg-preview a.svg a.png 1200   # SVG rastern (Entwürfe, Assets)
```

### Sounds austauschen

Jeder Sound hat einen festen Namen (Liste: `cargo xtask sound-preview` oder `crates/elora-audio/src/cues.rs`, z. B. `jump`, `hammer_fire`, `grenade_explode`). Eine Datei `assets/sounds/files/<name>.wav` ersetzt den Klang gleichen Namens.

1. Datei importieren – jedes Format, das ffmpeg liest; optional Start und Länge in Sekunden:
   ```sh
   cargo xtask sound-import jump ~/Downloads/boing.ogg          # ganze Datei
   cargo xtask sound-import jump ~/Downloads/boing.ogg 0.1 0.3  # ab 0,1 s, 0,3 s lang
   ```
   Der Import wandelt nach Mono/44,1 kHz/16 Bit, entfernt Stille am Anfang, blendet das Ende aus und bringt die Spitze auf −1 dBFS. Längstens 2,5 s (Test).
2. Lautstärke im Spiel: Abschnitt `[gain]` in `assets/sounds/sounds.toml` (1 = unverändert).
3. Quelle und Lizenz in `assets/SOURCES.md` eintragen – nur CC0 oder mit GPL-3.0 verträgliche Lizenzen.
4. Anhören: `cargo xtask sound-preview jump` (nach `target/sounds/`) oder im Spiel (`cargo run --bin elora`, eingebettet beim Bauen).
5. `cargo xtask check`, dann committen.

Zurück zum prozeduralen Klang: die Datei löschen (für die vier prozeduralen Sounds `spawn`, `death`, `weapon_switch`, `pickup_weapon` steht er in `sounds.toml`; für die anderen braucht es dann dort wieder einen Eintrag, sonst schlägt der Test fehl).

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
