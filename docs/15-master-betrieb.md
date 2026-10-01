# Master-Server betreiben

Stand: M8.4 · Grundlage: E-112, E-127, E-162, E-166, E-170 · Dateien: [`deploy/master/`](../deploy/master/)

Der Master führt die Internet-Liste: Spiel-Server melden sich alle 20 s per HTTPS an, der Master prüft sie per UDP (Info-Abfrage) und nimmt sie nach 60 s ohne Meldung wieder heraus. Clients holen die Liste per HTTPS. Der Master speichert nichts auf der Platte.

| Was | Wert |
|---|---|
| Adresse (Standard in Client und Server) | `https://elora.bastianswelt.de` (E-166) |
| Schnittstellen | `POST /register`, `GET /servers` (JSON) |
| Programm | `elora-master` (im Release-Paket enthalten) |
| Lauscht | `127.0.0.1:8300` (HTTP), HTTPS macht der Reverse-Proxy |
| Ausgehend | UDP zu den Spiel-Servern (Prüfung), beliebige Ports |
| Eingehend (Firewall) | nur 80/443 für den Proxy |

## Variante 0: Webspace mit PHP (E-171)

Für normales Webhosting ohne eigenen Dienst: [`deploy/master-php/`](../deploy/master-php/) – gleiche Schnittstelle, Liste als JSON-Datei, UDP-Prüfung direkt in der Anmeldung. Dazu gehört die Projektseite `index.php` mit Live-Status der Server (E-172). Anleitung zum Hochladen: [`deploy/master-php/LIESMICH.md`](../deploy/master-php/LIESMICH.md). Getestet mit PHP 8.3 und einem echten `elora-server`.

## Variante A: systemd

```sh
# als root auf dem Server
useradd --system --no-create-home --shell /usr/sbin/nologin elora-master
install -m 755 elora-master /usr/local/bin/elora-master       # aus dem Linux-Paket
install -m 644 deploy/master/elora-master.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now elora-master
journalctl -u elora-master -f                                   # Log
```

## Variante B: Docker

```sh
docker build -f deploy/master/Dockerfile -t elora-master .
docker run -d --name elora-master --restart unless-stopped -p 127.0.0.1:8300:8300 elora-master
docker logs -f elora-master
```

## HTTPS davor

**Caddy** (Zertifikat automatisch): [`deploy/master/Caddyfile`](../deploy/master/Caddyfile) nach `/etc/caddy/Caddyfile`, dann `systemctl reload caddy`.

**nginx**: [`deploy/master/nginx.conf`](../deploy/master/nginx.conf) als Server-Block, Zertifikat z. B. mit `certbot --nginx -d elora.bastianswelt.de`.

Der Proxy setzt `X-Forwarded-For` auf die echte Absenderadresse (beim nginx-Beispiel überschrieben, nicht angehängt). Der Master nutzt sie wegen `--behind-proxy` – nur so prüft er die richtige Adresse und hält die Grenze von 32 Servern je Adresse (insgesamt höchstens 8192) ein. Den Master **nie** mit `--behind-proxy` direkt ins Internet stellen, sonst könnte jeder eine Adresse vortäuschen.

DNS: ein `A`/`AAAA`-Eintrag `elora.bastianswelt.de` auf den Server.

## Prüfen

```sh
curl https://elora.bastianswelt.de/servers          # [] oder Liste
elora-server --name "Test" --port 8303               # trägt sich von selbst ein (E-170)
curl https://elora.bastianswelt.de/servers          # „Test“ erscheint nach wenigen Sekunden
```

Der Spiel-Server muss von außen per UDP erreichbar sein (Port freigeben), sonst lehnt der Master ihn nach der Prüfung ab.

## Verhalten der Spiel-Server (E-170)

- **`elora-server`** trägt sich von selbst beim Standard-Master ein. Abschalten mit `--no-master` oder `masters = []` in der Konfiguration; ein eigener Master mit `--master https://…` ersetzt den Standard.
- **Aus dem Client gehostet** („Server erstellen“) ist eine Runde privat, außer „Im Internet anzeigen“ ist angehakt.
- **Client:** Die Internet-Liste kommt aus `master_url` in den Einstellungen (Standard wie oben; leer = keine Internet-Liste).

## Updates

Neues Programm einspielen und neu starten (`systemctl restart elora-master` bzw. Container neu bauen). Server melden sich innerhalb von 20 s wieder an; Clients sehen die Liste danach wieder vollständig.
