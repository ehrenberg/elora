# Elora-Master und Projektseite für Webspace (PHP)

Gleiche Schnittstelle wie das Programm `elora-master`, aber als PHP-Skript für normales
Webhosting ohne eigenen Dienst. Client und Server brauchen keine Änderung.

## Voraussetzungen

- PHP **8.0** oder neuer (keine Erweiterungen nötig, kein Datenbankserver)
- HTTPS für die Domain (bei den meisten Hostern per Let's Encrypt im Kundenmenü)
- Apache mit `mod_rewrite` (Standard bei fast allen Webspaces) – sonst siehe unten
- **ausgehendes UDP** muss erlaubt sein (für die Prüfung der Spiel-Server)

## Hochladen

1. Subdomain `elora.bastianswelt.de` im Kundenmenü anlegen, Zielordner z. B. `elora/`.
2. Den **Inhalt** dieses Ordners dorthin hochladen (FTP/SFTP), also:
   `index.php` (Projektseite), `master.php` (Master), `common.php`, `config.php`,
   `.htaccess`, Ordner `assets/` (Grafiken, Schrift) und Ordner `data/` mit seiner `.htaccess`.
   Dateien mit Punkt am Anfang sind in manchen FTP-Programmen versteckt – mit hochladen!
3. Ordner `data/` für PHP beschreibbar machen (meist schon so; sonst Rechte 755 oder 775).
4. HTTPS für die Subdomain einschalten.

## Projektseite

`index.php` ist die Startseite: Vorstellung des Spiels, Links zu GitHub und den Downloads und
ein **Live-Status** der Server. Die Seite fragt die gelisteten Server selbst per UDP nach Name,
Karte, Modus und Spielern (alle gleichzeitig, höchstens 48) und merkt sich das 15 Sekunden in
`data/status.json`. Texte der Server werden maskiert ausgegeben. Grafiken stammen aus dem
Projekt (`assets/*.svg`, CC-BY-SA 4.0), die Schrift ist Inter (OFL, `assets/Inter-OFL.txt`).

## Prüfen

Im Browser oder mit curl:

- `https://elora.bastianswelt.de/master.php` → `{"service":"elora-master"}`
- `https://elora.bastianswelt.de/servers` → `{"servers":[]}`

Dann einen Spiel-Server starten (er meldet sich von selbst an) und nach ein paar Sekunden
erneut `/servers` aufrufen – seine Adresse steht in der Liste. Im Log des Servers steht
`beim Master angemeldet`, sonst die Fehlermeldung des Masters.

## Wenn etwas nicht geht

| Meldung des Servers | Ursache |
|---|---|
| `Server per UDP nicht erreichbar` | Spiel-Server-Port (UDP) nicht von außen erreichbar – oder der Hoster sperrt ausgehendes UDP. Im zweiten Fall in `config.php` `'verify_udp' => false` setzen: Server werden dann ungeprüft gelistet (nur mit ihrer eigenen Adresse, höchstens 32 je Adresse). |
| `falsche Protokollversion` | Spiel und `config.php` (`protocol_version`) passen nicht zusammen – nach einem Update der Spielversion anpassen. |
| `Datenablage nicht beschreibbar` | Ordner `data/` beschreibbar machen. |
| 404 bei `/servers` | `mod_rewrite`/`.htaccess` greift nicht. Ohne Rewrite geht auch die Adresse `https://elora.bastianswelt.de/master.php` als Master-Adresse (in den Einstellungen bzw. `--master`), dann wird `/master.php/servers` aufgerufen. |

## Hinweise

- Steht ein CDN/Proxy (z. B. Cloudflare) vor der Seite, `trust_forwarded_for` auf `true`
  setzen – sonst sieht der Master nur die Adresse des Proxys.
- Die Liste liegt in `data/servers.json` und räumt sich selbst auf (60 s ohne Anmeldung).
