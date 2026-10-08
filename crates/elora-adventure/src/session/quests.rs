//! Glue to the quests: outcomes of quest updates become session events.

use super::{Session, SessionEvent};
use crate::quest::Outcome;

impl Session {
    /// Show the outcomes with the events of the next tick.
    pub(super) fn push_outcomes(&mut self, outs: Vec<Outcome>) {
        self.pending.extend(outcomes(outs));
    }
}

/// Outcomes of quest updates as session events.
pub(super) fn outcomes(v: Vec<Outcome>) -> Vec<SessionEvent> {
    v.into_iter()
        .map(|o| match o {
            Outcome::Notice(n) => SessionEvent::Notice(n),
            Outcome::Open(o) => SessionEvent::Open(o),
        })
        .collect()
}
