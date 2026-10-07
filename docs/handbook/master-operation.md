# Running the master server

Status: M8.4 · Basis: E-112, E-127, E-162, E-166, E-170 · Files: [`deploy/master/`](../../deploy/master/)

The master keeps the internet list: game servers register every 20 s via HTTPS, the master checks them via UDP (info query) and removes them again after 60 s without a registration. Clients fetch the list via HTTPS. The master stores nothing on disk.

| What | Value |
|---|---|
| Address (default in client and server) | `https://elora.bastianswelt.de` (E-166) |
| Interfaces | `POST /register`, `GET /servers` (JSON) |
| Program | `elora-master` (included in the release package) |
| Listens on | `127.0.0.1:8300` (HTTP), HTTPS is handled by the reverse proxy |
| Outgoing | UDP to the game servers (check), any ports |
| Incoming (firewall) | only 80/443 for the proxy |

## Option 0: web hosting with PHP (E-171)

For regular web hosting without your own service: [`deploy/master-php/`](../../deploy/master-php/) – same interface, list as a JSON file, UDP check directly during registration. It includes the project page `index.php` with live server status (E-172). Upload instructions: [`deploy/master-php/LIESMICH.md`](../../deploy/master-php/LIESMICH.md). Tested with PHP 8.3 and a real `elora-server`.

## Option A: systemd

```sh
# as root on the server
useradd --system --no-create-home --shell /usr/sbin/nologin elora-master
install -m 755 elora-master /usr/local/bin/elora-master       # from the Linux package
install -m 644 deploy/master/elora-master.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now elora-master
journalctl -u elora-master -f                                   # log
```

## Option B: Docker

```sh
docker build -f deploy/master/Dockerfile -t elora-master .
docker run -d --name elora-master --restart unless-stopped -p 127.0.0.1:8300:8300 elora-master
docker logs -f elora-master
```

## HTTPS in front

**Caddy** (automatic certificate): [`deploy/master/Caddyfile`](../../deploy/master/Caddyfile) to `/etc/caddy/Caddyfile`, then `systemctl reload caddy`.

**nginx**: [`deploy/master/nginx.conf`](../../deploy/master/nginx.conf) as a server block, certificate e.g. with `certbot --nginx -d elora.bastianswelt.de`.

The proxy sets `X-Forwarded-For` to the real sender address (in the nginx example overwritten, not appended). The master uses it because of `--behind-proxy` – only this way does it check the right address and enforce the limit of 32 servers per address (at most 8192 in total). **Never** expose the master with `--behind-proxy` directly to the internet, otherwise anyone could spoof an address.

DNS: an `A`/`AAAA` record `elora.bastianswelt.de` pointing to the server.

## Checking

```sh
curl https://elora.bastianswelt.de/servers          # [] or list
elora-server --name "Test" --port 8303               # registers by itself (E-170)
curl https://elora.bastianswelt.de/servers          # “Test” appears after a few seconds
```

The game server must be reachable from outside via UDP (open the port), otherwise the master rejects it after the check. By default it listens on IPv4 and IPv6 (`bind = "::"`) and registers over both families; whatever the master reaches gets listed. Behind **DS-Lite** (e.g. Vodafone cable) there is no own IPv4 address – then only IPv6 works, and in the Fritzbox the port forwarding for UDP 8303 must also apply to IPv6.

## Behavior of game servers (E-170)

- **`elora-server`** registers with the default master by itself. Disable with `--no-master` or `masters = []` in the configuration; a custom master given with `--master https://…` replaces the default.
- **Hosted from the client** (“Create server”) a round is private unless “Show on the internet list” is checked.
- **Client:** The internet list comes from `master_url` in the settings (default as above; empty = no internet list).

## Updates

Install the new program and restart (`systemctl restart elora-master` or rebuild the container). Servers re-register within 20 s; afterwards clients see the complete list again.
