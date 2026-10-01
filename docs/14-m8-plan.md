# M8 – Release 1: Umsetzungsplan

Status: **angenommen** (E-169), in Umsetzung · Grundlage: [`06-roadmap.md`](06-roadmap.md) M8, E-003, E-027, E-161 bis E-164, O-44, O-47, O-48

## Ziel

Release 1 ist veröffentlicht: Pakete für Linux, Windows und macOS auf GitHub (E-161), ein laufender Master-Server für die Internet-Liste (E-162), übersetzte Server-Meldungen (E-164), Lizenz- und Credits-Seite, Balancing und Playtests abgeschlossen.

**Abnahme (Roadmap):** Release 1 ist veröffentlicht.

## Stand

- Spiel, Server, Master und Editor laufen aus dem Projektordner (`cargo run`).
- Mehrere Dateien werden **relativ zum Arbeitsverzeichnis** gesucht: `maps/`, `tuning.toml`, `known_servers.toml`, `assets/music/menu.wav`, beim Hosten `elora-server` im Build-Ordner. Ein installiertes Paket findet sie so nicht.
- Version `0.1.0`, Lizenz GPL-3.0 (Code), CC-BY-SA 4.0 für eigene Assets (E-027), `THIRD_PARTY_LICENSES` vorhanden, Quellen fremder Assets in `assets/SOURCES.md`.
- Repository auf GitHub (`ehrenberg/elora`), noch ohne CI.

## Arbeitsschritte

| # | Schritt | Inhalt | Prüfung |
|---|---|---|---|
| M8.1 | Übersetzte Server-Meldungen (O-48, E-164) | Server schickt Meldungs-Codes mit Werten statt deutscher Texte (Beitritt, Verlassen, Kartenwechsel, Abstimmungen, Kick, Modus, Runde …); Client übersetzt (DE/EN); Konsole/Log bleiben lesbar; Protokollversion 6 | Tests (alle Codes in beiden Sprachen, Integrationstest) |
| M8.2 | Paketfähigkeit | Daten neben dem Programm finden (Ordner `data/` bzw. `Resources` im macOS-Bundle), Arbeitsverzeichnis egal; schreibbare Dateien (`known_servers.toml`, eigenes Tuning, `server_key.toml` beim Hosten) ins Benutzerverzeichnis; `elora-server` neben dem Client finden | Tests + Start aus fremdem Ordner |
| M8.3 | Release-Builds (E-163) | GitHub-Actions-Workflow: bei Versions-Tag `v*` bauen und an ein GitHub-Release hängen – Linux (AppImage + tar.gz), Windows (ZIP), macOS (.app im DMG, unsigniert, Hinweis zum Öffnen); dazu `cargo xtask package` für lokale Pakete; Prüf-Workflow (fmt, clippy, test, deny) bei jedem Push | Workflow-Lauf auf GitHub, Pakete starten |
| M8.4 | Master-Server-Betrieb (O-47, E-162) | Fertige Betriebsdateien für `elora-master`: systemd-Dienst und Dockerfile, Anleitung (HTTPS über Reverse-Proxy, Firewall, Updates); Standard-Adresse im Client und im Server | Probelauf lokal; danach dein Betrieb |
| M8.5 | Lizenzen & Credits | Seite im Hauptmenü: Lizenz (GPL-3.0, CC-BY-SA 4.0), Mitwirkende, fremde Assets (aus `assets/SOURCES.md`), Bibliotheken (aus `THIRD_PARTY_LICENSES`); Lizenzdateien in jedem Paket | Sichtprüfung |
| M8.6 | Balancing & Playtests | Spielrunden mit mehreren Leuten (UAT): Tuning, Karten, Modi; Rückmeldungen sammeln, Änderungen als Entscheidungen festhalten | Dein Playtest |
| M8.7 | Release-Kandidat | Fehlerbehebung, Version setzen, Änderungsliste, Release-Notizen, Probe-Release (Pre-Release auf GitHub) | Deine Abnahme |
| M8.8 | Veröffentlichung | Tag setzen → GitHub-Release öffentlich (nur nach deinem ausdrücklichen OK) | Release 1 veröffentlicht |

## Entscheidungen zu M8

| # | Frage | Optionen | Entscheidung |
|---|---|---|---|
| D-M8-01 | Vertrieb (O-44) | GitHub Releases / itch.io / Flathub / Steam | E-161: **erst GitHub Releases**; weitere Kanäle als Möglichkeit (O-49) |
| D-M8-02 | Master-Server (O-47) | du betreibst ihn / keiner / später | E-162: **du betreibst ihn**, ich liefere Betriebsdateien und Anleitung |
| D-M8-03 | Builds | GitHub Actions / lokal | E-163: **GitHub Actions** bei Versions-Tag |
| D-M8-04 | Server-Meldungen (O-48) | vor Release 1 übersetzbar / später | E-164: **vor Release 1** |
| D-M8-05 | Versionsnummer von Release 1 | z. B. `1.0.0` / `0.9.0` (Beta) | E-165: **0.9.0 Beta** |
| D-M8-06 | Adresse des Master-Servers | deine Domain, z. B. `https://master.example.org` | E-166: **https://elora.bastianswelt.de** (voraussichtlich) |
| D-M8-07 | macOS-Signierung | unsigniert (Hinweis „Rechtsklick → Öffnen“) / signiert und notarisiert (Apple-Entwicklerkonto, 99 $/Jahr) | E-167: **unsigniert** |
| D-M8-08 | Playtest-Runden | Anzahl, Teilnehmer, Ablauf | E-168: Projektinhaber fragt Bekannte |
