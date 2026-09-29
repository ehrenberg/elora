# Entscheidungslog

Alle Entscheidungen trifft der Projektinhaber. Hier wird jede Entscheidung mit Datum und Begründung festgehalten. Offene Punkte bleiben offen, bis sie entschieden sind – es werden **keine Annahmen** getroffen.

## Getroffene Entscheidungen

| # | Datum | Thema | Entscheidung | Begründung |
|---|---|---|---|---|
| E-001 | 2026-09-25 | Ziel | Klon von Teeworlds mit identischem Spielgefühl | Vorgabe Projektinhaber |
| E-002 | 2026-09-25 | Dokumentation | Erkenntnisse im Ordner `docs/` | Vorgabe Projektinhaber |
| E-003 | 2026-09-25 | Ziel (O-01) | Veröffentlichung (öffentliches Release, eigene Identität) | Entscheidung Projektinhaber |
| E-004 | 2026-09-25 | Plattform (O-02) | Desktop: Linux, Windows, macOS | Entscheidung Projektinhaber |
| E-005 | 2026-09-25 | Referenzversion (O-03) | Teeworlds 0.7 | Entscheidung Projektinhaber |
| E-006 | 2026-09-25 | Assets (O-05) | Eigene Assets, eigener Stil | Entscheidung Projektinhaber |
| E-007 | 2026-09-25 | Code-Herkunft (O-23) | Komplett neu, Teeworlds-Quellcode dient als Referenz für Werte/Algorithmen | Entscheidung Projektinhaber |
| E-008 | 2026-09-25 | Protokoll (O-04) | Eigenes Netzwerkprotokoll, keine Kompatibilität zum Original | Entscheidung Projektinhaber |
| E-009 | 2026-09-25 | Sprache/Engine (O-07) | Rust mit eigener Engine (Server und Client teilen Code) | Entscheidung Projektinhaber |
| ~~E-010~~ | 2026-09-25 | Lizenz (O-22) | ~~Open Source, permissiv~~ → **ersetzt durch E-020** | Entscheidung Projektinhaber |
| E-011 | 2026-09-25 | Rendering (O-25) | wgpu + winit, eigener 2D-Renderer | Entscheidung Projektinhaber |
| E-012 | 2026-09-25 | Netzwerk (O-09) | Eigenes UDP-Protokoll (Snapshots, Delta-Kompression, eigene Zuverlässigkeitsschicht) | Entscheidung Projektinhaber |
| E-013 | 2026-09-25 | Erster Meilenstein (O-14/O-21) | Lokale Physik-Sandbox: ein Tee, Testkarte, Laufen/Springen/Hook – Feeling ohne Netzwerk abstimmen | Entscheidung Projektinhaber |
| E-014 | 2026-09-25 | Spielmodi Release 1 (O-12) | DM, TDM, CTF, LMS, LTS, Instagib | Entscheidung Projektinhaber |
| E-015 | 2026-09-25 | Physikwerte (O-30) | Original-Tuning **nicht** 1:1 übernehmen, sondern etwas abweichen (Umfang/Richtung → O-32) | Entscheidung Projektinhaber; eigene Identität |
| E-016 | 2026-09-25 | Waffen Release 1 (O-13) | Hammer, Laser, Granate | Entscheidung Projektinhaber |
| E-017 | 2026-09-25 | Testkarte (O-31) | Einfaches Textformat (Syntax → O-33) | Entscheidung Projektinhaber |
| E-018 | 2026-09-25 | Projektname (O-06) | **Elora** – zugleich Name der spielbaren Figur; Abgrenzung zu Teeworlds | Entscheidung Projektinhaber |
| E-019 | 2026-09-25 | Code-Struktur (O-29) | Aufteilung nach professionellem Rust-Standard gemäß [`03-architektur.md`](03-architektur.md) – **bestätigt** | Entscheidung Projektinhaber |
| E-020 | 2026-09-25 | Lizenz (O-24) | **GPL-3.0** (Copyleft), ersetzt E-010 | Entscheidung Projektinhaber |
| E-021 | 2026-09-25 | Physik-Arithmetik (O-28) | `f32` mit Quantisierung pro Tick (wie Original) | Entscheidung Projektinhaber |
| E-022 | 2026-09-25 | Vorgehen Physikwerte (O-32) | Claude schlägt pro Wert eine Abweichung mit Begründung vor, Projektinhaber entscheidet einzeln → [`04-tuning.md`](04-tuning.md) | Entscheidung Projektinhaber |

| E-023 | 2026-09-25 | Tuning (O-32) | Alle Vorschläge T-01 bis T-30 aus [`04-tuning.md`](04-tuning.md) angenommen | Entscheidung Projektinhaber |
| E-024 | 2026-09-25 | Karten-Textformat (O-33) | Vorschlag aus [`05-kartenformat.md`](05-kartenformat.md) angenommen (TOML + ASCII-Raster, Legende, Endung `.emap.toml`) – **nur für Test- und Entwicklungskarten** | Entscheidung Projektinhaber; für Release 1 zu einfach (keine Grafik-Layer) |
| E-025 | 2026-09-25 | Startausrüstung (O-35) | Elora spawnt **nur mit Hammer**; Laser und Granate ausschließlich per Pickup | Entscheidung Projektinhaber; Pickups und Kartenkontrolle werden wichtig |
| E-026 | 2026-09-25 | Instagib-Regeln | Klassisch: nur Laser, unendliche Munition, ein Treffer tötet, keine Pickups | Entscheidung Projektinhaber |
| E-027 | 2026-09-25 | Asset-Lizenz (O-36) | **CC-BY-SA 4.0** für eigene Grafiken/Sounds | Entscheidung Projektinhaber; Copyleft passend zu GPL-3.0 |
| E-028 | 2026-09-25 | Release-Kartenformat + Editor (O-10/O-16) | **Eigenes Format + eigener, ins Spiel integrierter Editor** (wie Teeworlds) | Entscheidung Projektinhaber; maximale Kontrolle |
| E-029 | 2026-09-25 | Figur & Skins (O-34/O-19) | Elora ist die Basisfigur; **Skin-System aus Teilen** (Körper, Augen, Deko u. ä., wie 0.7) inkl. Community-Skins | Entscheidung Projektinhaber |
| E-030 | 2026-09-25 | Grafikstil | **Flat/Vektor**: klare Formen, moderne Palette, auflösungsunabhängig | Entscheidung Projektinhaber; Abgrenzung zu Teeworlds |
| E-031 | 2026-09-25 | UI (O-27) | **egui** für Editor, Konsole, Debug-Regler; **eigene Spiel-UI** für Hauptmenü, Server-Browser, HUD | Entscheidung Projektinhaber |
| E-032 | 2026-09-25 | Audio (O-26) | **kira** | Entscheidung Projektinhaber |
| E-033 | 2026-09-25 | Vektor-Pipeline (O-38) | **Laufzeit-Tessellierung** (z. B. lyon → Dreiecke → wgpu), echt auflösungsunabhängig, dynamische Verformung möglich | Entscheidung Projektinhaber |
| E-034 | 2026-09-25 | Hosting (O-11) | Zunächst **nur lokales Git**, Hosting später | Entscheidung Projektinhaber |
| E-035 | 2026-09-25 | Bots (O-15) | **Nach Release 1**; Architektur sieht sie vor (Bots liefern Inputs wie Spieler) | Entscheidung Projektinhaber |
| E-036 | 2026-09-25 | Meilensteine (O-21) | Claude schlägt Roadmap bis Release 1 vor, Projektinhaber entscheidet je Meilenstein → [`06-roadmap.md`](06-roadmap.md) | Entscheidung Projektinhaber |
| E-037 | 2026-09-25 | Roadmap (O-21) | Meilensteine M0–M8 aus [`06-roadmap.md`](06-roadmap.md) angenommen | Entscheidung Projektinhaber |
| E-038 | 2026-09-25 | Rust-Toolchain | Umstieg auf **rustup**; Version fixiert in `rust-toolchain.toml` | Entscheidung Projektinhaber; reproduzierbar für alle Entwickler |
| E-039 | 2026-09-25 | Lokale Prüfungen (O-41) | **`cargo xtask`** (Rust-Programm im Workspace, plattformunabhängig) | Entscheidung Projektinhaber |
| E-040 | 2026-09-25 | Zusatz-Tools | **cargo-deny** (Lizenzen, Advisories) und **cargo-nextest** (Tests) | Entscheidung Projektinhaber |
| E-041 | 2026-09-25 | Workspace-Ort | Direkt im Projektordner (`Cargo.toml`, `crates/`, `apps/`, `docs/` im Root) | Entscheidung Projektinhaber |
| E-042 | 2026-09-25 | M0 Abnahme | M0 Projekt-Setup abgenommen (Commit `5ab9055`) | Entscheidung Projektinhaber |
| E-043 | 2026-09-25 | M1-Plan | Plan aus [`07-m1-plan.md`](07-m1-plan.md) inkl. technischer Festlegungen angenommen | Entscheidung Projektinhaber |
| E-044 | 2026-09-25 | Kamera (D-01) | **Statisch** wie 0.7-Standard: Kamera exakt auf Elora (korrigiert, siehe Analyse §9) | Entscheidung Projektinhaber |
| E-045 | 2026-09-25 | Sichtbereich (D-02) | Start mit Original (1,15 Mio. Einheiten², max. 1500 × 1050), als **Tuning-Regler** – finaler Wert in M1-Abnahme | Entscheidung Projektinhaber |
| E-046 | 2026-09-25 | Tuning speichern (D-03) | Sandbox speichert Werte in **`tuning.toml`**, wird beim Start geladen; Defaults bleiben im Code | Entscheidung Projektinhaber |
| E-047 | 2026-09-25 | Schriften | egui-Standardschriften (`epaint_default_fonts`, OFL-1.1 + Ubuntu Font Licence) **nicht** verwenden; stattdessen **Inter** (UI) und **JetBrains Mono** (Monospace), beide OFL-1.1, als Assets in `assets/fonts/` | Entscheidung Projektinhaber; keine Sonderlizenzen in Crate-Abhängigkeiten |
| E-048 | 2026-09-25 | Advisories | „unmaintained“-Meldungen von cargo-deny nur als **Warnung** (`-W unmaintained`), Sicherheitslücken bleiben Fehler. Anlass: `ttf-parser` (RUSTSEC-2026-0192, indirekt über egui) | Entscheidung Projektinhaber |
| E-049 | 2026-09-25 | M1 Abnahme | Physik-Sandbox abgenommen: Bewegungsgefühl „perfekt“, Tuning-Startwerte (E-023) bleiben unverändert | Entscheidung Projektinhaber |
| E-050 | 2026-09-25 | M2-Plan | Plan aus [`08-m2-plan.md`](08-m2-plan.md) inkl. technischer Festlegungen umsetzen | Entscheidung Projektinhaber |
| E-051 | 2026-09-25 | Waffenwahl (D-M2-01/02) | Tasten **1 Hammer, 2 Granate, 3 Laser**; Mausrad blättert in dieser Reihenfolge | Entscheidung Projektinhaber |
| E-052 | 2026-09-25 | Laser-Knockback (D-M2-03) | **Ja, leicht**: Stoß in Schussrichtung, Startwert **2** (Tuning-Regler) – Abweichung vom Original (0) | Entscheidung Projektinhaber |
| E-053 | 2026-09-25 | Trainings-Dummies (D-M2-04) | Dummies **aus der Karte** mit **Bewegungsmustern**: Stehen, Hin-und-her-Laufen, Springen, Laufen + Springen | Entscheidung Projektinhaber |
| E-054 | 2026-09-25 | Dummy-Kartenzeichen | Ein Zeichen pro Muster: `D` steht, `W` läuft, `J` springt, `X` läuft + springt (Erweiterung von E-024) | Entscheidung Projektinhaber |
| E-055 | 2026-09-25 | Kill-Taste (D-M2-05) | Erst in **M4** mit den Spielregeln | Entscheidung Projektinhaber |
| E-056 | 2026-09-29 | M2 Abnahme | Kampf lokal freigegeben („weiter gehts“); Respawn-Verhalten wie im Original (frühestens 0,5 s per Klick, sonst 3 s) bleibt, da kein Einwand | Entscheidung Projektinhaber |
| E-057 | 2026-09-29 | Vorhersage (D-M3-03) | Eigene Bewegung/Hook **und eigene Waffen** (Schüsse, Laserstrahl, Hammer-Effekt, Rückstoß) werden vorhergesagt; Schaden/Tod nur auf dem Server; andere Spieler interpoliert | Entscheidung Projektinhaber |
| E-058 | 2026-09-29 | Lag-Kompensation (D-M3-04) | **Keine** – wie Original | Entscheidung Projektinhaber |
| E-059 | 2026-09-29 | Kapazität (D-M3-01/02) | **Bis 64 Spieler** pro Server; Snapshots **25 Hz**, **50 Hz als LAN-Option** | Entscheidung Projektinhaber |
| E-060 | 2026-09-29 | Lokal hosten (D-M3-06) | Server als **eigener Prozess**; **Einrichtung und Konfiguration des Servers aus dem Client heraus** (Dialog, startet den Prozess) | Entscheidung Projektinhaber |
| E-061 | 2026-09-29 | Schutz (D-M3-05) | **Token-Handshake + Verschlüsselung** | Entscheidung Projektinhaber |
| E-062 | 2026-09-29 | Server-Vertrauen | **Wie SSH (TOFU):** Client merkt sich den Server-Schlüssel beim ersten Verbinden und warnt bei Änderung; Umsetzung Noise-Protokoll `XX` (Crate `snow`, Apache-2.0/MIT) | Entscheidung Projektinhaber |
| E-063 | 2026-09-29 | Kompression (D-M3-07) | „Bessere Methode als Original, sonst Huffman“ → **feldweises Delta mit Änderungsmaske + kompakte Zahlen, danach statischer Huffman mit auf eigenem Verkehr trainierter Tabelle** (Begründung: siehe `09-m3-plan.md`) | Entscheidung Projektinhaber, Methode von Claude ausgearbeitet |
| E-064 | 2026-09-29 | M3-Plan | Plan aus [`09-m3-plan.md`](09-m3-plan.md) mit obigen Antworten umsetzen | Entscheidung Projektinhaber |
| E-065 | 2026-09-29 | M3 Abnahme | Netzwerk abgenommen („erst mal alles super“) | Entscheidung Projektinhaber |
| E-066 | 2026-09-29 | Siegbedingung (D-M4-01) | **Wie Original:** Score-Limit 20, kein Zeitlimit, Sudden Death bei Gleichstand (CTF siehe E-067) | Entscheidung Projektinhaber |
| E-067 | 2026-09-29 | CTF-Wertung (D-M4-02) | Einzelpunkte wie Original (Träger +5, Aufnehmen/Zurückbringen/Träger töten +1, Kills wie DM); **Teamwertung = Eroberungen, Standard-Limit 5** (statt Rohwert 100/Eroberung) | Entscheidung Projektinhaber |
| E-068 | 2026-09-29 | Aufwärmen (D-M4-03) | **10 s Aufwärmen nach Kartenwechsel** (Punkte zählen nicht), dann **3 s Countdown** (Welt eingefroren); Countdown auch vor jedem weiteren Match/jeder Runde | Entscheidung Projektinhaber |
| E-069 | 2026-09-29 | Friendly Fire (D-M4-04) | **An:** Schaden und Rückstoß an Teammitgliedern, Teamkill −1 (Server-Einstellung, abschaltbar) | Entscheidung Projektinhaber |
| E-070 | 2026-09-29 | TDM-Respawn (D-M4-05) | frühestens nach **3 s** (wie Original) | Entscheidung Projektinhaber (Teil der Antwort zu D-M4-04) |
| E-071 | 2026-09-29 | Startausrüstung LMS/LTS (D-M4-06) | **Nur Hammer + Pickups** wie in allen Modi (E-025) | Entscheidung Projektinhaber |
| E-072 | 2026-09-29 | Konsole (D-M4-07, O-20) | **Server-Konsole** (Terminal) **+ Abstimmungen**; Remote-Konsole später | Entscheidung Projektinhaber |
| E-073 | 2026-09-29 | Team-Wahl (D-M4-08) | **Wie Original:** Beitritt ins kleinere Team, Wechsel und Zuschauen möglich, automatischer Ausgleich nach 1 min | Entscheidung Projektinhaber |
| E-074 | 2026-09-29 | Rotation (D-M4-09) | **Wie Original:** Kartenliste, Matches pro Karte, Teamtausch nach jedem Match | Entscheidung Projektinhaber |
| E-075 | 2026-09-29 | Sandbox-Modi (D-M4-10) | Spielmodi **in der Sandbox wählbar**, Dummies bekommen Teams | Entscheidung Projektinhaber |
| E-076 | 2026-09-29 | Instagib (D-M4-11) | **Schalter für alle Modi** (iDM, iTDM, iCTF, iLMS, iLTS) | Entscheidung Projektinhaber |
| E-077 | 2026-09-29 | Abstimmungen | Über **Karte, Modus, Kick (5 min Sperre), Zuschauer**; Ablauf wie Original (25 s) | Entscheidung Projektinhaber |
| E-078 | 2026-09-29 | Tasten (D-M4-12) | **T** Chat, **Y** Team-Chat, **Tab** Scoreboard, **F3/F4** Ja/Nein, **K** kill; Abstimmungen und Team-Wahl im Panel | Entscheidung Projektinhaber |
| E-079 | 2026-09-29 | M4-Plan | Plan aus [`10-m4-plan.md`](10-m4-plan.md) mit obigen Antworten umsetzen | Entscheidung Projektinhaber |
| E-080 | 2026-09-29 | Grafik-Erstellung (D-M5-01) | **Claude erzeugt SVG** (mit Selbstprüfung am gerenderten Bild) **+ externe Vektor-Bild-API** für aufwendige Motive; Freigabe per Screenshot durch den Projektinhaber | Entscheidung Projektinhaber |
| E-081 | 2026-09-29 | Sounds (D-M5-05) | **Prozedural generiert + CC0 gemischt** (generiert für UI/einfache Effekte, CC0 z. B. für Explosion/Treffer; Quellenliste) | Entscheidung Projektinhaber |
| E-082 | 2026-09-29 | Musik (D-M5-06) | **Nur im Menü**, kommt mit M7 | Entscheidung Projektinhaber |
| E-083 | 2026-09-29 | Reihenfolge M5 (D-M5-11) | **Erst Look, dann Sound** | Entscheidung Projektinhaber |
| E-084 | 2026-09-29 | Bild-API | **Recraft** (SVG-Ausgabe); API-Schlüssel nur als Umgebungsvariable, nie im Repo; Nutzungsbedingungen vor dem ersten Einsatz prüfen | Entscheidung Projektinhaber |
| E-085 | 2026-09-29 | Elora-Form (D-M5-03) | **Tropfenform** – nach oben spitz, starkes Squash & Stretch beim Springen/Landen | Entscheidung Projektinhaber |
| E-086 | 2026-09-29 | Skins (D-M5-04, O-39) | **Nur Farben + wenige Teile** aus fester Auswahl (z. B. Körperfarbe, Muster, Augenform), **keine Community-Skins** – ersetzt den Community-Teil von E-029 | Entscheidung Projektinhaber |
| E-087 | 2026-09-29 | Darstellungsgröße | **Etwas kleiner als Original:** sichtbarer Körper ≈ 36 Einheiten (Hitbox 28) | Entscheidung Projektinhaber |
| E-088 | 2026-09-29 | Zusatz-Effekte (D-M5-07) | **Kamera-Wackeln** (nahe Explosionen, eigener Schaden) und **Treffer-Marker** beim Treffen anderer – beides abschaltbar; genaue Form im Entwurf | Entscheidung Projektinhaber |
| E-089 | 2026-09-29 | Welt-Optik Textkarten (D-M5-08) | **Schlicht:** einfarbige Tiles mit Kontur, Verlaufs-Hintergrund; Aufwand in M6 | Entscheidung Projektinhaber |
| E-090 | 2026-09-29 | HUD (D-M5-09) | **Modern, am Fadenkreuz:** Leben/Rüstung/Munition als Balken bzw. Ringe am Fadenkreuz oder unten mittig; genaue Form im Entwurf | Entscheidung Projektinhaber |
| E-091 | 2026-09-29 | Emotes (D-M5-10) | **Emote-Rad mit 8 eigenen Emoticons** (Taste E halten, Maus wählt) + automatische Augen-Ausdrücke | Entscheidung Projektinhaber |
| E-092 | 2026-09-29 | M4 Abnahme | Spielmodi „vorerst abgeschlossen“ | Entscheidung Projektinhaber |
| E-093 | 2026-09-29 | M5-Plan | Plan aus [`11-m5-plan.md`](11-m5-plan.md) umsetzen | Entscheidung Projektinhaber |
| E-094 | 2026-09-29 | Elora-Entwurf (M5.3) | **Entwurf B „Wirbel“:** Tropfen mit zur Seite geneigter Spitze, heller Bauchfleck, kleine Augen mit Lächeln ([`design/elora-entwuerfe.png`](design/elora-entwuerfe.png)) | Entscheidung Projektinhaber |
| E-095 | 2026-09-29 | Skin-Teile (O-46) | Färbbar sind **Augen, Körper, Füße** – **nur Farben**, keine Form-Varianten je Teil | Entscheidung Projektinhaber |
| E-096 | 2026-09-29 | Farbwahl (O-46) | **Feste Palette** je Teil, keine freien Regler; Farben der Palette als Entwurf zur Freigabe | Entscheidung Projektinhaber |
| E-097 | 2026-09-29 | Bauchfleck (O-46) | Kein eigenes Teil – **hellere Abstufung der Körperfarbe** | Entscheidung Projektinhaber |
| E-098 | 2026-09-29 | Skin-Palette | Entwurf [`design/elora-palette.png`](design/elora-palette.png) freigegeben: 16 Farben für Körper und Füße, 8 für Augen | Entscheidung Projektinhaber |
| E-099 | 2026-09-29 | Skins in Team-Modi | Körper in **Teamfarbe**, Füße und Augen behalten die Farben des Spielers | Entscheidung Projektinhaber |
| E-100 | 2026-09-29 | Huffman-Tabelle | Neu trainierte Tabelle (Commit `3a550bc`) wird behalten | Entscheidung Projektinhaber |
| E-101 | 2026-09-29 | Pickups, Waffen, Flaggen (M5.5) | **Stil A „Rund“** aus [`design/elora-items.png`](design/elora-items.png) | Entscheidung Projektinhaber |
| E-102 | 2026-09-29 | HUD (M5.8) | **Entwurf B „Leiste unten mittig“** aus [`design/elora-hud.png`](design/elora-hud.png); zusätzlich färbt sich das **Fadenkreuz nach dem Leben: Weiß → Gelb → Rot** (fließend) | Entscheidung Projektinhaber |
| E-103 | 2026-09-29 | Emotes (M5.9) | Vorschlag aus [`design/elora-emotes.png`](design/elora-emotes.png) freigegeben: Herz, Lachen, Wut, Traurig, Staunen, Frage, GG, Schlaf; Rad mit Taste E, Anzeige ca. 2 s | Entscheidung Projektinhaber |
| E-104 | 2026-09-29 | Augen-Ausdrücke (M5.9) | Augen reagieren **zusätzlich automatisch**: zusammengekniffen bei Schaden, fröhlich nach einem Kill | Entscheidung Projektinhaber |
| E-105 | 2026-09-29 | Look (M5) und Vorgehen Sound (M5.7) | Look im Playtest abgenommen („super“); Sound wie vorgeschlagen: Crate `elora-audio` mit kira, prozeduraler Generator (sfxr-Prinzip, Parameter in `assets/sounds/`), WAV-Hörproben zur Freigabe, CC0 nur wo nötig | Entscheidung Projektinhaber |
| E-106 | 2026-09-30 | Hörprobe prozedurale Sounds | Behalten: `spawn`, `death`, `weapon_switch`, `pickup_weapon`. Alle anderen klingen „zu sehr nach Computersound“ → durch CC0-Sounds ersetzen, Ziel ist Atmosphäre | Entscheidung Projektinhaber |
| E-107 | 2026-09-30 | CC0-Sounds | Quelle **Kenney** (Grundstock) **+ Freesound nur CC0** (Lücken; Freesound verworfen → E-108); Klangstil **organisch / weich** (Plopps, Glibber, Holz, Stoff, natürliche Schläge) | Entscheidung Projektinhaber |
| E-108 | 2026-09-30 | Freesound | **Ohne Freesound** – nur Kenney-Pakete, Sounds dürfen bearbeitet und kombiniert werden. Hörprobe 2: Hammer soll nach Hammer-Schwung klingen, Granate nach Explosion, Laser organischer, Sprung passender; Rest in Ordnung | Entscheidung Projektinhaber |
| E-109 | 2026-09-30 | Sounds (M5.7) | Stand „erst einmal in Ordnung“ – M5.7 abgeschlossen. **Neue Sounds besorgt der Projektinhaber künftig selbst** (Ablauf in README „Sounds austauschen“) | Entscheidung Projektinhaber |
| E-110 | 2026-09-30 | M5 Abnahme | **M5 abgeschlossen** (Look im Playtest abgenommen, Sounds vorerst in Ordnung). Playtest-Aufzeichnung `rec-1790721458` als Golden-Regressionstest übernommen | Entscheidung Projektinhaber |
| E-111 | 2026-09-30 | Reihenfolge M6/M7 | **M7 (Menüs & Infrastruktur) vor M6 (Karten & Editor)**; Nummern bleiben, Reihenfolge M5 → M7 → M6 → M8 | Entscheidung Projektinhaber |

> **Hinweis zu E-020:** GPL-3.0 ist kompatibel mit der Teeworlds-Lizenz (zlib-artig) und mit MIT/Apache-lizenzierten Rust-Crates (wgpu, winit, …). Nicht kompatibel wären Abhängigkeiten unter GPL-2.0-only – `cargo-deny` prüft das. Die eigenen Assets (E-006) brauchen eine eigene Lizenz (→ O-36).

> **Hinweis zu E-007:** Die Teeworlds-Lizenz (zlib-artig) erlaubt Übernahme/Anpassung, verlangt aber, dass veränderte Versionen als solche gekennzeichnet sind und der Lizenzhinweis erhalten bleibt. Werden Algorithmen 1:1 nach Rust portiert, sollte der Teeworlds-Lizenzhinweis vorsorglich im Projekt mitgeführt werden (Datei `THIRD_PARTY_LICENSES`). Reine Zahlenwerte (Tuning) sind unkritisch.

## Offene Entscheidungen

### Grundlagen
- [x] ~~O-01 Ziel/Umfang~~ → E-003
- [x] ~~O-02 Plattform~~ → E-004
- [x] ~~O-03 Referenzversion~~ → E-005
- [x] ~~O-04 Kompatibilität~~ → E-008
- [x] ~~O-05 Assets~~ → E-006
- [x] ~~O-06 Projektname~~ → E-018
- [x] ~~O-22 Lizenz~~ → E-010
- [x] ~~O-23 Code-Herkunft~~ → E-007
- [x] ~~O-24 Konkrete Lizenz~~ → E-020
- [x] ~~O-36 Lizenz der Assets~~ → E-027

### Technik
- [x] ~~O-07 Programmiersprache / Engine~~ → E-009
- [x] ~~O-25 Grafik-/Fenster-Bibliothek~~ → E-011
- [x] ~~O-26 Audio-Bibliothek~~ → E-032
- [x] ~~O-27 UI-Lösung~~ → E-031
- [x] ~~O-38 Vektor-Pipeline~~ → E-033
- [x] ~~O-40 Vektor-Quellformat~~ → SVG (E-080, E-084)
- [x] ~~O-09 Netzwerk-Transport~~ → E-012
- [x] ~~O-28 Physik-Arithmetik~~ → E-021
- [x] ~~O-29 Rust-Workspace-Struktur~~ → E-019
- [x] ~~O-10 Release-Kartenformat~~ → E-028
- [ ] **O-37 Details Release-Kartenformat** (Layer-Modell, Binär/Text, Kompression, Asset-Einbettung) – später, vor den ersten echten Karten
- [x] ~~O-11 Versionskontrolle/Hosting~~ → E-034 (Repo-Struktur → E-019)
- [x] ~~O-41 CI ohne Hosting~~ → E-039

### Gameplay-Umfang
- [x] ~~O-12 Spielmodi~~ → E-014
- [x] ~~O-13 Waffen~~ → E-016
- [x] ~~O-14 Multiplayer vs. Sandbox zuerst~~ → E-013
- [x] ~~O-30 Physikwerte~~ → E-015
- [x] ~~O-31 Sandbox-Testkarte~~ → E-017
- [x] ~~O-32 Physikwerte im Einzelnen~~ → E-023
- [x] ~~O-33 Syntax des Karten-Textformats~~ → E-024
- [x] ~~O-34 Elora als Figur~~ → E-029
- [x] ~~O-39 Skin-Aufbau~~ → E-086 (Details → O-46)
- [x] ~~O-46 Skin-Auswahl im Detail~~ → E-095, E-096, E-097
- [x] ~~O-35 Startausrüstung~~ → E-025
- [x] ~~O-15 Bots~~ → E-035
- [x] ~~O-16 Map-Editor~~ → E-028 (Zeitpunkt → Meilensteine O-21)

### Features / Infrastruktur
- [ ] **O-17 Server-Browser / Master-Server**
- [ ] **O-18 Demos / Replays**
- [x] ~~O-19 Skins-System~~ → E-029
- [x] ~~O-20 Konsole~~ → E-072 (Remote-Konsole später → O-45)
- [ ] **O-45 Remote-Konsole** (Admin-Befehle aus dem Client mit Passwort)
- [x] ~~O-42 Netzwerk-Zielwerte~~ → E-059
- [ ] **O-43 Release-Karten** (Anzahl, Modi)
- [ ] **O-44 Vertrieb** (itch.io, Steam, Flathub, Website …)
- [x] ~~O-21 Meilensteine~~ → E-037
