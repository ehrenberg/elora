//! Entwicklungsaufgaben für Elora (E-039).
//!
//! Aufruf: `cargo xtask <befehl>`

mod traffic;

use std::fmt::Write as _;
use std::process::{Command, ExitCode};

const HELP: &str = "\
Verwendung: cargo xtask <befehl>

Befehle:
  check          Alle Prüfungen: fmt, clippy, test, deny
  fmt            Code formatieren
  train-huffman  Huffman-Tabelle aus synthetischem Verkehr erzeugen (E-063)
  net-stats      Nachrichtengrößen für 8/16/64 Spieler messen
  help           Diese Hilfe
";

fn main() -> ExitCode {
    let task = std::env::args().nth(1);
    let result = match task.as_deref() {
        Some("check") => check(),
        Some("fmt") => cargo(&["fmt", "--all"]),
        Some("train-huffman") => train_huffman(),
        Some("net-stats") => {
            net_stats();
            Ok(())
        }
        Some("help") | None => {
            print!("{HELP}");
            Ok(())
        }
        Some(other) => Err(format!("Unbekannter Befehl `{other}`\n\n{HELP}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("xtask: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> Result<(), String> {
    let steps: [(&str, &[&str]); 4] = [
        ("Formatierung", &["fmt", "--all", "--check"]),
        (
            "Clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
        ),
        (
            "Tests",
            &[
                "nextest",
                "run",
                "--workspace",
                "--all-features",
                "--no-tests=pass",
            ],
        ),
        (
            "Lizenzen & Advisories",
            // „unmaintained“ nur als Warnung, echte Sicherheitslücken bleiben Fehler (E-048)
            &["deny", "check", "-W", "unmaintained"],
        ),
    ];
    for (name, args) in steps {
        println!("==> {name}");
        cargo(args)?;
    }
    println!("==> Alle Prüfungen bestanden");
    Ok(())
}

fn cargo(args: &[&str]) -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(&cargo)
        .args(args)
        .status()
        .map_err(|e| format!("`{cargo}` konnte nicht gestartet werden: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo {}` fehlgeschlagen", args.join(" ")))
    }
}

const TABLE_FILE: &str = "crates/elora-protocol/src/huffman_table.rs";

/// Zählt Byte-Häufigkeiten über Verkehr mit 8, 16 und 64 Spielern.
fn train_huffman() -> Result<(), String> {
    let mut freq = [0u64; 256];
    for (players, seed) in [(8, 1), (16, 2), (64, 3)] {
        let t = traffic::generate(players, seed, 3000, 3);
        for msg in t.server.iter().chain(&t.client) {
            for &b in msg {
                freq[b as usize] += 1;
            }
        }
    }
    // auf u32 skalieren
    let max = *freq.iter().max().unwrap_or(&1);
    let scale = (max / u64::from(u32::MAX / 2)).max(1);
    let values: Vec<String> = freq
        .iter()
        .map(|f| (f / scale).max(1).to_string())
        .collect();
    let mut body = String::new();
    for row in values.chunks(12) {
        let _ = writeln!(body, "    {},", row.join(", "));
    }
    let text = format!(
        "//! Byte-Häufigkeiten für den statischen Huffman-Code (E-063).\n\
         //!\n\
         //! Erzeugt mit `cargo xtask train-huffman` – nicht von Hand ändern.\n\n\
         /// Häufigkeiten der Byte-Werte 0..=255.\n\
         pub const FREQUENCIES: [u32; 256] = [\n{body}];\n"
    );
    std::fs::write(TABLE_FILE, text).map_err(|e| format!("{TABLE_FILE}: {e}"))?;
    println!("Tabelle geschrieben: {TABLE_FILE} – neu bauen, dann `cargo xtask net-stats`");
    Ok(())
}

/// Misst durchschnittliche Nachrichtengrößen roh und komprimiert (anderer Seed als beim Training).
fn net_stats() {
    println!(
        "| Spieler | Snapshot wie Original (Delta aller Felder + Huffman) | Elora roh | Elora + Huffman | vs. Original | Eingabe roh | Eingabe + Huffman | Server→Client kB/s (25 Hz) |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    for players in [8, 16, 64] {
        let t = traffic::generate(players, 99, 3000, 3);
        let avg = |msgs: &[Vec<u8>], packed: bool| {
            let total: usize = msgs
                .iter()
                .map(|m| {
                    if packed {
                        elora_protocol::msg::pack(m.clone()).len()
                    } else {
                        m.len()
                    }
                })
                .sum();
            total as f64 / msgs.len().max(1) as f64
        };
        let (sr, sp) = (avg(&t.server, false), avg(&t.server, true));
        let orig = avg(&t.original, true);
        let (cr, cp) = (avg(&t.client, false), avg(&t.client, true));
        // Pro Client: ein Snapshot alle 2 Ticks, zzgl. ca. 40 Byte Kopf/Verschlüsselung
        let kbps = (sp + 40.0) * 25.0 / 1000.0;
        println!(
            "| {players} | {orig:.0} B | {sr:.0} B | {sp:.0} B | {:+.0} % | {cr:.0} B | {cp:.0} B | {kbps:.1} |",
            (sp / orig - 1.0) * 100.0
        );
    }
}
