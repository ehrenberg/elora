//! Abstimmungen (E-077): Karte, Modus, Kick, Zuschauer.
//!
//! Dauer 25 s. Angenommen sofort bei mehr als der Hälfte Ja, abgelehnt sofort bei
//! mindestens der Hälfte Nein, sonst nach Ablauf angenommen, wenn mehr Ja als Nein.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use elora_net::Socket;
use elora_protocol::{Message, ServerMsg, VoteInfo, VoteKind, VoteSubject, reason};
use elora_sim::Team;

use crate::GameServer;

pub const VOTE_DURATION: Duration = Duration::from_secs(25);

#[derive(Debug, Clone)]
pub struct Vote {
    pub kind: VoteKind,
    pub subject: VoteSubject,
    yes: HashSet<u32>,
    no: HashSet<u32>,
    ends: Instant,
}

/// Ergebnis einer Auswertung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    Passed,
    Failed,
}

impl Vote {
    pub fn new(kind: VoteKind, subject: VoteSubject, caller: u32, now: Instant) -> Self {
        let mut yes = HashSet::new();
        yes.insert(caller);
        Self {
            kind,
            subject,
            yes,
            no: HashSet::new(),
            ends: now + VOTE_DURATION,
        }
    }

    pub fn cast(&mut self, id: u32, yes: bool) {
        self.yes.remove(&id);
        self.no.remove(&id);
        if yes {
            self.yes.insert(id);
        } else {
            self.no.insert(id);
        }
    }

    pub fn forget(&mut self, id: u32) {
        self.yes.remove(&id);
        self.no.remove(&id);
    }

    pub fn outcome(&self, voters: usize, now: Instant) -> Outcome {
        let (yes, no) = (self.yes.len(), self.no.len());
        if yes * 2 > voters {
            Outcome::Passed
        } else if no * 2 >= voters {
            Outcome::Failed
        } else if now >= self.ends {
            if yes > no {
                Outcome::Passed
            } else {
                Outcome::Failed
            }
        } else {
            Outcome::Pending
        }
    }

    pub fn info(&self, voters: usize, now: Instant) -> VoteInfo {
        VoteInfo {
            subject: self.subject.clone(),
            yes: u32::try_from(self.yes.len()).unwrap_or(u32::MAX),
            no: u32::try_from(self.no.len()).unwrap_or(u32::MAX),
            voters: u32::try_from(voters).unwrap_or(u32::MAX),
            seconds_left: u32::try_from(self.ends.saturating_duration_since(now).as_secs())
                .unwrap_or(0),
        }
    }
}

impl<S: Socket> GameServer<S> {
    /// Stimmberechtigte: alle beigetretenen Spieler.
    pub(crate) fn voters(&self) -> usize {
        self.player_count().max(1)
    }

    pub(crate) fn send_vote_status(&mut self, now: Instant) {
        let info = self.vote.as_ref().map(|v| v.info(self.voters(), now));
        self.broadcast(&ServerMsg::Vote(info));
    }

    fn reply(&mut self, id: u32, message: Message) {
        self.endpoint
            .send(id, &ServerMsg::Notice(message).encode(), true);
    }

    pub(crate) fn call_vote(&mut self, id: u32, kind: VoteKind, now: Instant) {
        if !self.votes_enabled {
            self.reply(id, Message::VotesDisabled);
            return;
        }
        if self.vote.is_some() {
            self.reply(id, Message::VoteRunning);
            return;
        }
        let caller_slot = self.clients.get(&id).and_then(|c| c.slot);
        let subject = match &kind {
            VoteKind::Map(m) => {
                if !self.maps.iter().any(|e| &e.name == m) {
                    self.reply(id, Message::UnknownMap { map: m.clone() });
                    return;
                }
                VoteSubject::Map(m.clone())
            }
            VoteKind::Mode { mode, instagib } => VoteSubject::Mode {
                mode: *mode,
                instagib: *instagib,
            },
            VoteKind::Kick(s) | VoteKind::Spectate(s) => {
                let slot = *s as usize;
                if self.client_by_slot(slot).is_none() || caller_slot == Some(slot) {
                    self.reply(id, Message::InvalidPlayer);
                    return;
                }
                let name = self.name_of(slot);
                if matches!(kind, VoteKind::Kick(_)) {
                    VoteSubject::Kick(name)
                } else {
                    VoteSubject::Spectate(name)
                }
            }
        };
        let who = caller_slot.map_or_else(|| "?".into(), |s| self.name_of(s));
        self.notice(Message::VoteStarted {
            who,
            subject: subject.clone(),
        });
        self.vote = Some(Vote::new(kind, subject, id, now));
        self.update_vote(now);
        self.send_vote_status(now);
    }

    pub(crate) fn update_vote(&mut self, now: Instant) {
        let Some(vote) = &self.vote else { return };
        match vote.outcome(self.voters(), now) {
            Outcome::Pending => {}
            Outcome::Failed => {
                let subject = vote.subject.clone();
                self.vote = None;
                self.notice(Message::VoteFailed(subject));
                self.send_vote_status(now);
            }
            Outcome::Passed => {
                let vote = self.vote.take().expect("vorhanden");
                self.notice(Message::VotePassed(vote.subject.clone()));
                self.send_vote_status(now);
                self.execute(vote.kind, now);
            }
        }
    }

    fn execute(&mut self, kind: VoteKind, now: Instant) {
        match kind {
            VoteKind::Map(m) => {
                if let Err(e) = self.change_map(&m, now) {
                    tracing::warn!("Kartenwechsel fehlgeschlagen: {e:#}");
                    self.notice(Message::MapChangeFailed { map: m });
                }
            }
            VoteKind::Mode { mode, instagib } => {
                let mut cfg = self.rules.cfg.clone();
                cfg.mode = mode;
                cfg.instagib = instagib;
                cfg.score_limit = None;
                self.set_rules(cfg);
            }
            VoteKind::Kick(s) => {
                self.kick(s as usize, reason::KICKED_BY_VOTE, true, now);
            }
            VoteKind::Spectate(s) => {
                self.rules
                    .set_team(&mut self.world, s as usize, Team::Spectator);
            }
        }
    }

    /// Laufende Abstimmung abbrechen (Konsole).
    pub fn cancel_vote(&mut self, now: Instant) {
        if self.vote.take().is_some() {
            self.notice(Message::VoteCancelled);
            self.send_vote_status(now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_rules() {
        let now = Instant::now();
        let mut v = Vote::new(
            VoteKind::Map("x".into()),
            VoteSubject::Map("x".into()),
            1,
            now,
        );
        assert_eq!(v.outcome(4, now), Outcome::Pending, "1 von 4 Ja");
        v.cast(2, true);
        assert_eq!(
            v.outcome(4, now),
            Outcome::Pending,
            "2 von 4 ist nicht mehr als die Hälfte"
        );
        v.cast(3, true);
        assert_eq!(v.outcome(4, now), Outcome::Passed);
        let mut v = Vote::new(
            VoteKind::Map("x".into()),
            VoteSubject::Map("x".into()),
            1,
            now,
        );
        v.cast(2, false);
        v.cast(3, false);
        assert_eq!(v.outcome(4, now), Outcome::Failed, "Hälfte Nein");
        let mut v = Vote::new(
            VoteKind::Map("x".into()),
            VoteSubject::Map("x".into()),
            1,
            now,
        );
        v.cast(2, true);
        v.cast(3, false);
        assert_eq!(
            v.outcome(6, now + VOTE_DURATION),
            Outcome::Passed,
            "nach Ablauf mehr Ja als Nein"
        );
        v.cast(4, false);
        assert_eq!(v.outcome(6, now + VOTE_DURATION), Outcome::Failed);
    }
}
