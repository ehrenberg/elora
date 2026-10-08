//! Server console (E-072): commands in the server's terminal.

use std::fmt::Write as _;
use std::time::Instant;

use elora_game::Mode;
use elora_net::Socket;
use elora_sim::Team;

use crate::GameServer;

pub const HELP: &str = "\
Befehle:
  status                      Spieler, Karte, Modus
  mode <dm|tdm|ctf|lms|lts>   Modus wechseln (neues Match)
  instagib <on|off>           Instagib für alle Modi (E-076)
  scorelimit <n>              Siegpunkte (0 = aus)
  timelimit <min>             Zeitlimit in Minuten (0 = aus)
  friendlyfire <on|off>       Schaden an Teammitgliedern
  restart                     Match neu starten
  map <name>                  Karte wechseln
  maps                        bekannte Karten
  kick <slot>                 Spieler trennen
  ban <slot>                  Spieler trennen und 5 min sperren
  spectate <slot>             Spieler zu den Zuschauern
  say <text>                  Nachricht an alle
  vote cancel                 laufende Abstimmung abbrechen
  quit                        Server beenden
  help                        diese Hilfe";

fn on_off(arg: Option<&str>) -> Option<bool> {
    match arg? {
        "on" | "an" | "1" | "true" => Some(true),
        "off" | "aus" | "0" | "false" => Some(false),
        _ => None,
    }
}

impl<S: Socket> GameServer<S> {
    /// Runs a console command and returns the output.
    pub fn command(&mut self, line: &str, now: Instant) -> String {
        let line = line.trim();
        let (cmd, rest) = line
            .split_once(' ')
            .map_or((line, ""), |(c, r)| (c, r.trim()));
        let arg = (!rest.is_empty()).then_some(rest);
        let slot = || arg.and_then(|a| a.parse::<usize>().ok());
        match cmd {
            "" => String::new(),
            "help" => HELP.to_owned(),
            "status" => self.status(),
            "mode" => match arg.and_then(Mode::parse) {
                Some(m) => {
                    let mut cfg = self.rules.cfg.clone();
                    cfg.mode = m;
                    cfg.score_limit = None;
                    self.set_rules(cfg);
                    format!("Modus: {}", self.rules.cfg.title())
                }
                None => "Verwendung: mode <dm|tdm|ctf|lms|lts>".into(),
            },
            "instagib" => match on_off(arg) {
                Some(on) => {
                    let mut cfg = self.rules.cfg.clone();
                    cfg.instagib = on;
                    self.set_rules(cfg);
                    format!("Modus: {}", self.rules.cfg.title())
                }
                None => "Verwendung: instagib <on|off>".into(),
            },
            "scorelimit" => match arg.and_then(|a| a.parse().ok()) {
                Some(n) => {
                    self.rules.cfg.score_limit = Some(n);
                    format!("Siegpunkte: {n}")
                }
                None => "Verwendung: scorelimit <n>".into(),
            },
            "timelimit" => match arg.and_then(|a| a.parse().ok()) {
                Some(n) => {
                    self.rules.cfg.time_limit = n;
                    format!("Zeitlimit: {n} min")
                }
                None => "Verwendung: timelimit <min>".into(),
            },
            "friendlyfire" => match on_off(arg) {
                Some(on) => {
                    self.rules.cfg.friendly_fire = on;
                    self.world.friendly_fire = on;
                    format!("Friendly Fire: {}", if on { "an" } else { "aus" })
                }
                None => "Verwendung: friendlyfire <on|off>".into(),
            },
            "restart" => {
                let cfg = self.rules.cfg.clone();
                self.set_rules(cfg);
                "Match neu gestartet".into()
            }
            "map" => match arg {
                Some(m) => match self.change_map(m, now) {
                    Ok(()) => format!("Karte: {m}"),
                    Err(e) => format!("Fehler: {e:#}"),
                },
                None => "Verwendung: map <name>".into(),
            },
            "maps" => self.map_names().join(", "),
            "kick" | "ban" => match slot() {
                Some(s) if self.kick(s, elora_protocol::reason::KICKED, cmd == "ban", now) => {
                    format!("Slot {s} getrennt")
                }
                _ => "Verwendung: kick <slot> (siehe status)".into(),
            },
            "spectate" => match slot() {
                Some(s) if self.world.player(s).is_some() => {
                    self.rules.set_team(&mut self.world, s, Team::Spectator);
                    format!("Slot {s} schaut zu")
                }
                _ => "Verwendung: spectate <slot>".into(),
            },
            "say" => match arg {
                Some(text) => {
                    self.chat(None, false, text);
                    String::new()
                }
                None => "Verwendung: say <text>".into(),
            },
            "vote" if arg == Some("cancel") => {
                self.cancel_vote(now);
                "Abstimmung abgebrochen".into()
            }
            other => format!("Unbekannter Befehl `{other}` – `help` zeigt alle Befehle"),
        }
    }

    fn status(&self) -> String {
        let r = &self.rules;
        let mut out = format!(
            "Karte {} · {} · Phase {:?} · Teams {}:{} · Spieler {}\n",
            self.map_name(),
            r.cfg.title(),
            r.phase,
            r.team_score[0],
            r.team_score[1],
            self.player_count()
        );
        for (i, p) in self.world.players.iter().enumerate() {
            let Some(p) = p else { continue };
            let stats = r.stats.get(&i).copied().unwrap_or_default();
            let _ = writeln!(
                out,
                "  [{i:2}] {:<16} {:?} Punkte {} ({}/{})",
                self.name_of(i),
                p.team,
                stats.score,
                stats.kills,
                stats.deaths
            );
        }
        out
    }
}
