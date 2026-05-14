use crate::models::map::{MapMode, Phase};
use crate::state::Event::{BombPlanted, RoundOver, RoundStarted};
use crate::state::{
    Event, FromState, Maybe, State, StateAssign, StateCollector, StateCollectorDyn, StateEvents,
    StateGetter, StateSetter, States,
};
use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum BombStatus {
    #[serde(alias = "carried")]
    Carried,
    #[serde(alias = "dropped")]
    Dropped,
    #[serde(alias = "planted")]
    Planted,
    #[serde(alias = "planting")]
    Planting,
    #[serde(alias = "defusing")]
    Defusing,
    #[serde(alias = "exploded")]
    Exploded,
    #[serde(alias = "defused")]
    Defused,
}

#[derive(Debug, Deserialize)]
pub enum WinTeam {
    #[serde(alias = "CT")]
    Ct,
    #[serde(alias = "T")]
    T,
}

#[derive(Debug, Deserialize)]
pub struct Round {
    phase: Phase,
    win_team: Option<WinTeam>,
    bomb: Option<BombStatus>,
}

impl BombStatus {
    fn status(&self) -> BombStatus {
        *self
    }
}

impl Round {
    fn phase(&self) -> Phase {
        self.phase
    }
}

impl StateCollector for Round {
    type Inner = Round;

    fn vtable() -> &'static [&'static dyn StateAssign<Inner = Self::Inner>] {
        &[&(
            State::RoundPhase as StateSetter<Phase>,
            Round::phase as StateGetter<Self::Inner, Phase>,
        )]
    }

    fn next(s: Option<&Self>) -> Vec<&dyn StateCollectorDyn> {
        let r = s.as_ref();
        vec![r.map(|p| &p.bomb).unwrap_or(&None)]
    }

    fn inner(&self) -> &Self::Inner {
        self
    }
}

impl StateCollector for BombStatus {
    type Inner = BombStatus;

    fn vtable() -> &'static [&'static dyn StateAssign<Inner = Self::Inner>] {
        &[&(
            State::BombStatus as StateSetter<BombStatus>,
            BombStatus::status as StateGetter<Self::Inner, BombStatus>,
        )]
    }

    fn inner(&self) -> &Self::Inner {
        self
    }
}

impl FromState for Phase {
    fn from_state(state: &State) -> Option<&Maybe<Phase>> {
        match state {
            State::RoundPhase(r) => Some(r),
            _ => None,
        }
    }
}

impl FromState for BombStatus {
    fn from_state(state: &State) -> Option<&Maybe<BombStatus>> {
        match state {
            State::BombStatus(b) => Some(b),
            _ => None,
        }
    }
}

impl StateEvents for Maybe<Phase> {
    fn compare(&self, previous: &Self, states: &States) -> Vec<Event> {
        if matches!(previous, Maybe::Set(Phase::Live)) && !matches!(self, Maybe::Set(Phase::Live)) {
            return vec![RoundOver];
        }

        if !matches!(previous, Maybe::Set(Phase::Live)) && matches!(self, Maybe::Set(Phase::Live)) {
            let limit = match states.get::<MapMode>() {
                Maybe::Unknown => 0,
                Maybe::Set(mode) => mode.time_limit(),
            };

            return vec![RoundStarted(limit)];
        }

        [].into()
    }
}

impl StateEvents for Maybe<BombStatus> {
    fn compare(&self, previous: &Self, _: &States) -> Vec<Event> {
        if matches!(self, Maybe::Set(BombStatus::Planted))
            && !matches!(previous, Maybe::Set(BombStatus::Planted))
        {
            return vec![BombPlanted];
        }

        [].into()
    }
}
