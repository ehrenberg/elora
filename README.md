# Elora

Ein 2D-Multiplayer-Arena-Shooter nach dem Vorbild von Teeworlds. Elora ist zugleich der Name der spielbaren Figur.

## Entwicklung

Voraussetzungen: [rustup](https://rustup.rs) (die Version wird über `rust-toolchain.toml` automatisch installiert), `cargo-deny` und `cargo-nextest`.

```sh
cargo run --bin elora     # Client / Sandbox starten (Steuerung: docs/07-m1-plan.md)
cargo xtask check         # alle Prüfungen: fmt, clippy, Tests, Lizenzen
cargo xtask fmt           # Code formatieren
```

## Dokumentation

Siehe [`docs/`](docs/README.md) – Analyse, Entscheidungslog, Architektur, Tuning, Kartenformat, Roadmap.

## Lizenz

- Code: [GPL-3.0](LICENSE)
- Grafiken & Sounds: CC-BY-SA 4.0
- Drittanbieter: [THIRD_PARTY_LICENSES](THIRD_PARTY_LICENSES)
