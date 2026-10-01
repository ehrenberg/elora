# Elora

Ein 2D-Multiplayer-Arena-Shooter nach dem Vorbild von [Teeworlds](https://teeworlds.com) – eigene Figur, eigener Stil, gleiches Spielgefühl. Elora ist zugleich der Name der spielbaren Figur.

**Stand:** M0–M7 abgeschlossen (Bewegung, Hook, Waffen, Dummies, Netzwerk, Spielmodi, Look & Sound, Menüs & Server-Browser, Karten & Editor). Als Nächstes M8 (Release 1). Siehe [Roadmap](docs/06-roadmap.md).

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

```sh
cargo run --bin elora                               # startet ins Hauptmenü
```

Im Hauptmenü: **Spielen** (Server-Browser mit Internet über `https://elora.bastianswelt.de`, LAN und Favoriten; Direkt-Verbinden; „Schnell spielen“ = letzter Server), **Training** (Sandbox), **Server erstellen** (startet einen eigenen Server und verbindet), **Einstellungen**, **Beenden**. Im Spiel öffnet Esc das Pause-Menü.

### Sandbox direkt starten (Entwicklung)

Mit einer Karte, `--mode` oder `--connect` auf der Kommandozeile geht es ohne Menü direkt ins Spiel, mit Debug-Panel:

```sh
cargo run --bin elora -- maps/sandbox.emap          # Standardkarte
cargo run --bin elora -- maps/eigene.emap           # andere Karte
cargo run --bin elora -- maps/ctf-test.emap --mode ctf   # Spielmodus gegen Dummies
cargo run --bin elora -- maps/look-test.emap         # Kartenlook: Materialien, Hintergründe, Deko, Animationen
cargo run --bin elora -- maps/tiles-test.emap        # Plattform, Eis, Sprungfeld, Beschleuniger (S = Runter)
```

**Pakete:** `cargo xtask package --archive` baut ein Release-Paket für das eigene System unter `dist/` (Programme, Release-Karten, Lizenzen, Symbol; macOS zusätzlich `Elora.app`). Bei einem Tag `v<version>` baut GitHub Actions Linux (tar.gz, AppImage), Windows (ZIP) und macOS (DMG) und legt ein Entwurfs-Release an; jeder Push wird mit `cargo xtask check` geprüft.

**Dateien:** Mitgelieferte Daten (`maps/`, Menümusik) sucht das Spiel erst im Arbeitsverzeichnis, dann in `ELORA_DATA`, neben dem Programm, in `../share/elora` oder `../Resources` (macOS). Einstellungen, `tuning.toml`, `known_servers.toml` und die Dateien des gehosteten Servers (`server.toml`, `server_key.toml`) liegen im Einstellungsordner (`~/.config/elora`, Windows `%APPDATA%\Elora`, macOS `~/Library/Application Support/Elora`); eigene und heruntergeladene Karten unter `~/.local/share/elora`.

**Release-Karten** (M6.10): `dm-wiese` (64×36, 4–8 Spieler), `dm-wueste` (96×48, 8–12), `dm-winter` (128×64, 12–16), `ctf-wald` (150×48, 8–12), `ctf-nacht` (190×64, 12–16). Layouts werden in `tools/design/release_maps/` gebaut und geprüft (`python3 tools/design/release_maps/export.py`), die Dateien schreibt `cargo test -p elora-client --bin elora write_release_maps -- --ignored`.

Spielmodi in der Sandbox: `--mode dm|tdm|ctf|lms|lts` (optional `--instagib`) oder im Panel unter *Spiel → Modus*. CTF braucht eine Karte mit Flaggen, z. B. `maps/ctf-test.emap`.

Die Sandbox dient zum Tunen und Testen: Dummies, Pickups, Live-Regler für alle Werte, Hot-Reload der Karte.

### Online

**Server starten:**

```sh
cargo run --bin elora-server
cargo run --bin elora-server -- --port 8303 --map maps/dm-wiese.emap --max-clients 16 --name "Mein Server"
cargo run --bin elora-server -- --config server.toml
```

| Option | Standard | Bedeutung |
|---|---|---|
| `--port` | 8303 | UDP-Port |
| `--bind` | 0.0.0.0 | Adresse, an die gebunden wird |
| `--map` | `maps/dm-wiese.emap` | Karte (relativ: erst Arbeitsverzeichnis, dann Datenordner der Installation) |
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
| `--master` | `https://elora.bastianswelt.de` | beim Master-Server anmelden (Internet-Liste); ein eigener ersetzt den Standard, mehrfach möglich |
| `--no-master` | – | nicht in die Internet-Liste eintragen |
| `--config` | – | alle Optionen aus einer TOML-Datei; weitere Optionen überschreiben sie |

Weitere Einstellungen nur in der Konfigurationsdatei: `rotation = ["sandbox", "ctf-test"]` (Kartenrotation), `maps_dir`, und ein Abschnitt `[rules]` (`warmup_secs`, `countdown_secs`, `tdm_respawn_secs`, `team_balance_secs`, `match_swap`, `matches_per_map`).

**Konsole:** Im Terminal des Servers Befehle eintippen – `help` zeigt alle: `status`, `mode ctf`, `instagib on`, `scorelimit 10`, `timelimit 5`, `friendlyfire off`, `restart`, `map <name>`, `maps`, `kick <slot>`, `ban <slot>`, `spectate <slot>`, `say <text>`, `vote cancel`, `quit`.

Beenden: `quit` + Enter oder Strg+C. Log-Ausgabe steuern: `RUST_LOG=debug cargo run --bin elora-server`.

**Verbinden:**

```sh
cargo run --bin elora -- --connect 127.0.0.1:8303
```

oder im Hauptmenü unter *Spielen* (Adresse, Favoriten). Für Entwickler zusätzlich im Debug-Panel (F1) → *Netzwerk*.

**Karten-Editor:** Hauptmenü → *Editor*. Seitenleiste rechts: Datei (Neu, Öffnen, Speichern), Rückgängig/Wiederholen, Werkzeuge, Ebenen ein-/ausblenden, Karteneigenschaften (Name, Autor, Größe, Himmel). Werkzeuge (Tasten 1–8): Pinsel, Rechteck, Füllen, Radierer, Auswahl (Strg+C/X/V, Entf), Entities, Material (Erde, Sand, Schnee), Deko (setzen, wählen, ziehen; Größe, Drehung, Färbung, Animationen; eigene SVGs einbetten); `[`/`]` ändern die Pinselgröße. Abschnitte *Hintergrund-Ebenen* (Vorlagen Tag/Nacht, Parallax, Wiederholung, Reihenfolge) und *Animationen* (Bewegung/Farbe, Punkte mit Kurven, Kurvenbild). Kartenfläche: mittlere Maustaste oder Leertaste + Ziehen verschiebt, Mausrad zoomt, links anwenden, rechts löschen. Kürzel: Strg+Z/Strg+Y, Strg+S, Strg+N, Strg+O, F5 testspielen (ohne Speichern; Esc im Spiel führt zurück in den Editor), Esc zurück. Eigene Karten liegen im Benutzerverzeichnis (`~/.local/share/elora/maps`, Windows/macOS im Einstellungsordner) und erscheinen danach in Training und *Server erstellen*.

**Aus dem Client hosten:** Hauptmenü → *Server erstellen* (Name, Karte, Modus, Instagib, Spieler) → *Server starten*. Der Client schreibt `server.toml` und startet `elora-server` als eigenen Prozess (vorher einmal `cargo build`). Mehr Optionen im Debug-Panel → *Netzwerk* → *Server einrichten …*.

**Master-Server (Internet-Liste):**

```sh
cargo run --bin elora-master -- --bind 0.0.0.0:8300                # HTTP auf Port 8300
cargo run --bin elora-master -- --bind 127.0.0.1:8300 --behind-proxy   # hinter Reverse-Proxy
```

Spielserver melden sich beim Standard-Master an (abschaltbar mit `--no-master`, eigener mit `--master <URL>` oder `masters = ["…"]` in `server.toml`; aus dem Client gehostete nur mit „Im Internet anzeigen“), alle 20 s; der Master listet sie erst, wenn er sie selbst per UDP erreicht, und entfernt sie nach 60 s ohne Anmeldung. Öffentlich den Master hinter einem Reverse-Proxy mit HTTPS betreiben (z. B. Caddy: `reverse_proxy 127.0.0.1:8300`) und `--behind-proxy` setzen. Schnittstelle: `GET /servers`, `POST /register` (JSON). Betrieb mit systemd oder Docker: [`docs/15-master-betrieb.md`](docs/15-master-betrieb.md).

**Firewall:** Für Spieler aus dem LAN/Internet muss der UDP-Port (Standard 8303) freigegeben sein, z. B. `sudo ufw allow 8303/udp` bzw. `sudo firewall-cmd --add-port=8303/udp`.

**Sicherheit:** Die Verbindung ist verschlüsselt (Noise-Protokoll). Der Client merkt sich den Schlüssel jedes Servers in `known_servers.toml` und warnt, wenn er sich ändert (wie SSH).

### Steuerung

Standardbelegung; alles außer Esc und F1 lässt sich unter *Einstellungen → Steuerung* umbelegen.

| Taste | Aktion |
|---|---|
| A / D | laufen |
| Leertaste | springen, in der Luft Doppelsprung |
| Rechte Maustaste (halten) | Hook |
| Linke Maustaste | schießen (Granate/Laser: gedrückt halten = Dauerfeuer) |
| 1 / 2 / 3, Mausrad | Hammer / Granate / Laser |
| Esc | Pause-Menü: Team, Abstimmungen, Einstellungen, Hauptmenü, Beenden |
| F1 | Debug-Panel ein/aus (auch im Menü) |
| R | Respawn (nur Sandbox ohne Modus) |
| K | Selbstmord (`kill`) |
| T / Y | Chat / Team-Chat (Enter senden, Esc abbrechen; online) |
| Tab (halten) | Scoreboard |
| E (halten) | Emote-Rad: Maus in Richtung des Emotes, loslassen zeigt es |
| F3 / F4 | Ja / Nein bei Abstimmungen |
| F5 | Aufzeichnung starten/beenden (nur Sandbox, siehe unten) |

Nach dem Tod: Feuertaste = Respawn (frühestens nach 0,5 s, TDM 3 s), sonst automatisch nach 3 s. In LMS/LTS kein Respawn bis zur nächsten Runde.

Team wählen, zuschauen und Abstimmungen (Karte, Modus, Kick, Zuschauer) starten: `Esc` → Pause-Menü.

## Dateien im Arbeitsverzeichnis

| Datei | Zweck | im Git |
|---|---|---|
| `settings.toml` | Spieler-Einstellungen: Name, Skin, Sprache, Grafik, Ton, Effekte, Maus, Favoriten – **im Benutzerverzeichnis** (Linux `~/.config/elora/`, Windows `%APPDATA%\Elora\`, macOS `~/Library/Application Support/Elora/`), beim Beenden gespeichert | nein, lokal |
| `tuning.toml` | Entwickler-Tuning der Sandbox: Physik und Sichtbereich (Panel → *Speichern*) | nein, lokal |
| `server.toml` | vom Client geschriebene Server-Konfiguration | nein |
| `server_key.toml` | geheimer Server-Schlüssel – nicht weitergeben | nein |
| `known_servers.toml` | bekannte Server-Schlüssel des Clients | nein |
| `maps/*.emap` | Karten ([Format](docs/05-kartenformat.md), ansehen mit `cargo xtask map-dump`) | ja |

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
