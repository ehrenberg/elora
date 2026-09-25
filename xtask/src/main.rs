//! Entwicklungsaufgaben für Elora (E-039).
//!
//! Aufruf: `cargo xtask <befehl>`

use std::process::{Command, ExitCode};

const HELP: &str = "\
Verwendung: cargo xtask <befehl>

Befehle:
  check   Alle Prüfungen: fmt, clippy, test, deny
  fmt     Code formatieren
  help    Diese Hilfe
";

fn main() -> ExitCode {
    let task = std::env::args().nth(1);
    let result = match task.as_deref() {
        Some("check") => check(),
        Some("fmt") => cargo(&["fmt", "--all"]),
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
                "--",
                "-D",
                "warnings",
            ],
        ),
        (
            "Tests",
            &["nextest", "run", "--workspace", "--no-tests=pass"],
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
