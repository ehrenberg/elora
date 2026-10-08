//! Server console (E-072): commands in the server's terminal.

use std::fmt::Write as _;
use std::time::Instant;

use elora_game::Mode;
use elora_net::Socket;
use elora_sim::Team;

use crate::GameServer;

pub const HELP: &str = "\
Commands:
  status                      players, map, mode
  mode <dm|tdm|ctf|lms|lts>   change mode (new match)
  instagib <on|off>           instagib for all modes (E-076)
  scorelimit <n>              score limit (0 = off)
  timelimit <min>             time limit in minutes (0 = off)
  friendlyfire <on|off>       damage to team mates
  restart                     restart the match
  map <name>                  change map
  maps                        known maps
  kick <slot>                 disconnect a player
  ban <slot>                  disconnect a player and ban for 5 min
  spectate <slot>             move a player to the spectators
  say <text>                  message to everyone
  vote cancel                 cancel the running vote
  quit                        stop the server
  help                        this help";

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
                    format!("Mode: {}", self.rules.cfg.title())
                }
                None => "Usage: mode <dm|tdm|ctf|lms|lts>".into(),
            },
            "instagib" => match on_off(arg) {
                Some(on) => {
                    let mut cfg = self.rules.cfg.clone();
                    cfg.instagib = on;
                    self.set_rules(cfg);
                    format!("Mode: {}", self.rules.cfg.title())
                }
                None => "Usage: instagib <on|off>".into(),
            },
            "scorelimit" => match arg.and_then(|a| a.parse().ok()) {
                Some(n) => {
                    self.rules.cfg.score_limit = Some(n);
                    format!("Score limit: {n}")
                }
                None => "Usage: scorelimit <n>".into(),
            },
            "timelimit" => match arg.and_then(|a| a.parse().ok()) {
                Some(n) => {
                    self.rules.cfg.time_limit = n;
                    format!("Time limit: {n} min")
                }
                None => "Usage: timelimit <min>".into(),
            },
            "friendlyfire" => match on_off(arg) {
                Some(on) => {
                    self.rules.cfg.friendly_fire = on;
                    self.world.friendly_fire = on;
                    format!("Friendly Fire: {}", if on { "on" } else { "off" })
                }
                None => "Usage: friendlyfire <on|off>".into(),
            },
            "restart" => {
                let cfg = self.rules.cfg.clone();
                self.set_rules(cfg);
                "Match restarted".into()
            }
            "map" => match arg {
                Some(m) => match self.change_map(m, now) {
                    Ok(()) => format!("Map: {m}"),
                    Err(e) => format!("Error: {e:#}"),
                },
                None => "Usage: map <name>".into(),
            },
            "maps" => self.map_names().join(", "),
            "kick" | "ban" => match slot() {
                Some(s) if self.kick(s, elora_protocol::reason::KICKED, cmd == "ban", now) => {
                    format!("Slot {s} disconnected")
                }
                _ => "Usage: kick <slot> (see status)".into(),
            },
            "spectate" => match slot() {
                Some(s) if self.world.player(s).is_some() => {
                    self.rules.set_team(&mut self.world, s, Team::Spectator);
                    format!("Slot {s} is spectating")
                }
                _ => "Usage: spectate <slot>".into(),
            },
            "say" => match arg {
                Some(text) => {
                    self.chat(None, false, text);
                    String::new()
                }
                None => "Usage: say <text>".into(),
            },
            "vote" if arg == Some("cancel") => {
                self.cancel_vote(now);
                "Vote cancelled".into()
            }
            other => format!("Unknown command `{other}` – `help` lists all commands"),
        }
    }

    fn status(&self) -> String {
        let r = &self.rules;
        let mut out = format!(
            "Map {} · {} · phase {:?} · teams {}:{} · players {}\n",
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
                "  [{i:2}] {:<16} {:?} score {} ({}/{})",
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
