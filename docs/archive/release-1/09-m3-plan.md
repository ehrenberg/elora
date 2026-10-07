# M3 – Networking: Implementation Plan

Status: **completed** (E-065) · accepted (E-057–E-064) · Basis: [`06-roadmap.md`](06-roadmap.md) M3, E-008, E-012, analysis §10

## Goal

A dedicated server, multiple clients over UDP. Your own movement feels the same at 100 ms ping as offline. Acceptance: 2 to 8 players on LAN, plus a test with simulated ping, jitter and packet loss.

## Work Steps

| # | Step | Crate | Content | Check |
|---|---|---|---|---|
| M3.1 ✅ | Serialization | `elora-protocol` | Bit/byte packer with variable-length integers, messages (input, snapshot, events, connection), version number | Unit tests: round trip, malformed packets are rejected |
| M3.2 ✅ | Transport | `elora-net` | UDP socket, connection setup with token (against spoofed senders), keepalive, timeout, disconnect with reason; reliable and unreliable channels (sequence, ack, resend), splitting of large messages | Tests over a simulated link with loss and reordering |
| M3.3 ✅ | Network simulator | `elora-net` | Adjustable ping, jitter, packet loss, reordering, inserted between socket and transport | Tests; sliders in the debug panel |
| M3.4 ✅ | Snapshots | `elora-protocol` | Full world snapshot (characters, projectiles, lasers, pickups, player infos), **delta against the last acknowledged snapshot**, CRC | Tests: delta round trip bit-identical, size measurement |
| M3.5 ✅ | Server | `apps/elora-server` | Dedicated server without graphics: load map, add/remove players, apply inputs per tick, send snapshots, report input timing back; configuration via file/command line | Headless integration test: server + 2 test clients |
| M3.6 ✅ | Client: connection & interpolation | `elora-client` | Connect via IP:port, receive snapshots, interpolate other characters and objects between snapshots | Manual on LAN |
| M3.7 ✅ | Client: prediction | `elora-client` | Buffer inputs and send them with a target tick, regulate prediction time (like `INPUTTIMING`), simulate forward from the snapshot, correction on divergence (see D-M3-03) | Test: prediction = server on a lossless link |
| M3.8 ✅ | Local hosting | `elora-client` | "Start server" from the client (see D-M3-06), then connect automatically | Manual |
| M3.9 ✅ | Debug & metrics | `elora-client` | Panel: ping, packet loss, bandwidth, prediction ticks, corrections; network simulator sliders | Visual check |
| M3.10 ✅ | Acceptance | – | 2–8 players on LAN, test with 100 ms simulated ping | Your acceptance |

## Technical Specifications (Proposal)

- **No async runtime:** `std::net::UdpSocket` (non-blocking) in a dedicated network thread, channels to the game logic. Simple, easy to test and sufficient for a 50 TPS game.
- **Transport and protocol are separate:** `elora-net` knows nothing about the game (bytes in, bytes out). `elora-protocol` knows the messages and snapshots, but no sockets.
- **Transport testable without a real network:** The socket layer is swappable. Tests run over a simulated in-memory link.
- **Simulation stays unchanged:** Server and client prediction use the same `World::step`. For prediction, `elora-sim` gains the ability to adopt a world state from a snapshot.
- **Sandbox stays:** Without a connection, the client keeps running locally as before (for tuning, dummies and recordings).
- **Game rules:** In M3 the server plays "free for all" without scoring; the rules follow in M4.

## Decisions for M3

| # | Question | Original behavior (0.7) |
|---|---|---|
| D-M3-01 → E-059 | Maximum players per server | Default 8, technically up to 64 |
| D-M3-02 → E-059 | Snapshot rate | 25 Hz (every 2nd tick), 50 Hz as a LAN option |
| D-M3-03 → E-057 | Scope of prediction | own movement/hook only; others interpolated; weapons not predicted |
| D-M3-04 → E-058 | Lag compensation for hits | none |
| D-M3-05 → E-061/E-062 | Encryption / protection | Token handshake, no encryption |
| D-M3-06 → E-060 | Local hosting | Server as a separate process started by the client |
| D-M3-07 → E-063 | Compression | Huffman with a fixed table + delta |

## Elaboration of the Decisions

### Prediction of Own Weapons (E-057)

From each snapshot, the client builds a local world and simulates it forward with its own not-yet-acknowledged inputs up to the prediction tick – using the same `World::step` as the server. In this **prediction world**:
- forces apply (knockback, rocket jump), but **no damage, no death, no pickups** – those are decided by the server alone;
- other characters get **no inputs** (like `Tick(false)` in the original: walking direction is kept, no new jump/hook);
- own projectiles and laser beams are drawn from the prediction, other players' from the interpolated snapshots.

### Encryption (E-061, E-062)

- Flow: token request (padded to 512 bytes, against amplification attacks) → token (bound to the sender address) → Noise handshake `Noise_XX_25519_ChaChaPoly_BLAKE2s` → encrypted packets with an explicit packet number as nonce (UDP-friendly, replay protection via a window).
- The server has a persistent key (file next to the server configuration). The client stores known server keys (`known_servers.toml`) and warns when one changes.

### Compression (E-063)

The original computes, per object, the difference of all integers to the previous one, writes them with variable length and compresses the packet with Huffman (fixed table). Better is:
1. **Per-field delta with a change mask:** per object one bit per field "changed?"; unchanged fields cost 1 bit instead of ≥ 1 byte. Most fields (health, weapon, hook state) rarely change.
2. **Compact numbers:** changed fields as a difference, ZigZag + variable length.
3. **Static Huffman** on top, with a table trained on **our** traffic (tool in `xtask`) instead of the original's table.

In experience, stages 1 + 2 already beat the original's "delta + Huffman"; stage 3 gets the rest. Measured and documented in M3.9 (bytes per snapshot at 8/16/64 players).

## Implementation Notes (as of 2026-09-29)

### Usage

- **Start the server:** `cargo run --bin elora-server -- [--port 8303] [--map maps/sandbox.emap.toml] [--max-clients 8] [--high-bandwidth] [--name "…"]` or with `--config server.toml`. Quit: type `quit` or Ctrl+C.
- **Connect:** in the panel under *Network*, enter the address → *Connect*, or `cargo run --bin elora -- --connect 127.0.0.1:8303`. *Disconnect* returns to the sandbox.
- **Local hosting (E-060):** *Set up server …* → name, port, map, max. players, 50 Hz, "keep running" → *Start and connect*. The settings are stored in `server.toml`; the client starts `elora-server` as a separate process.
- **TOFU (E-062):** On the first connection, the client stores the server key in `known_servers.toml`. If it changes, a warning appears with both fingerprints and the choice "Trust new key".
- **Simulator (M3.3/M3.9):** In the online panel, set delay, jitter and loss for outgoing packets.

### Measurements

Prediction in the integration test (server + client in a simulated network, 100 ms ping):

| Quantity | Value |
|---|---|
| Predicted ticks | 7 |
| Input lead | ≈ 125 ms |
| Remaining input time at the server | ≈ 15–25 ms (target: > 10 ms) |
| Deviation prediction ↔ server (lossless) | **0** units |
| Deviation at 10 % loss + 30 ms jitter | > 0 in 1 of 40 samples (redundant inputs catch almost everything) |

Sizes (`cargo xtask net-stats`, synthetic traffic, delta against a base 3 snapshots older, all players run/jump/hook/shoot):

| Players | Snapshot like original (delta of all fields + Huffman) | Elora raw | Elora + Huffman | vs. original | Input raw | Input + Huffman | Server→client per client (25 Hz) |
|---|---|---|---|---|---|---|---|
| 8 | 198 B | 209 B | 188 B | −5 % | 48 B | 31 B | 5.7 kB/s |
| 16 | 341 B | 357 B | 322 B | −5 % | 48 B | 31 B | 9.1 kB/s |
| 64 | 1169 B | 1224 B | 1106 B | −5 % | 48 B | 31 B | 28.6 kB/s |

**Assessment regarding E-063:** The per-field delta with change mask plus trained Huffman is **measurably, but only slightly (≈ 5 %) better** than the original's method with the same table – Huffman already makes the original's many zero differences cheap, and with many active players almost every object changes in every snapshot. A delta against a **predicted** base (position + velocity extrapolated) would gain considerably more; this is noted as a possible optimization but not needed for Release 1: even 64 players need < 30 kB/s per client. Real games in sandbox idle are at ≈ 30–50 B per snapshot.

### Limitations and Open Points

- **Weapon prediction (E-057):** own shots, laser beams, hammer effects and recoil appear immediately; damage, death and pickups come from the server.
- **Setup dialog and key warning** are built, but I have not clicked through them with the mouse (only starting via `--connect` and starting the server from the command line were tested live).
- **Server rules:** free for all without scoring (M4).
- **Map:** the server transmits the text format; the release format follows in M6.
