use crate::state::{
    FromState, Maybe, State, StateAssign, StateCollector, StateGetter, StateSetter,
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum MapMode {
    #[serde(alias = "custom")]
    Custom,
    #[serde(alias = "casual")]
    Casual,
    #[serde(alias = "skirmish")]
    Skirmish,
    #[serde(alias = "competitive")]
    Competitive,
    #[serde(alias = "scrimcomp2v2")]
    Wingman,
    #[serde(alias = "scrimcomp5v5")]
    WeaponsExpert,
    #[serde(alias = "cooperative")]
    Cooperative,
    #[serde(alias = "deathmatch")]
    Deathmatch,
    #[serde(alias = "training")]
    Training,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum Phase {
    #[serde(alias = "freezetime")]
    Freezetime,
    #[serde(alias = "live")]
    Live,
    #[serde(alias = "warmup")]
    Warmup,
    #[serde(alias = "paused")]
    Paused,
    #[serde(alias = "timeout_t")]
    TTimeout,
    #[serde(alias = "timeout_ct")]
    CtTimeout,
    #[serde(alias = "gameover")]
    Gameover,
    #[serde(alias = "intermission")]
    Intermission,
    #[serde(alias = "over")]
    Over,
    #[serde(alias = "bomb")]
    Bomb,
    #[serde(alias = "defuse")]
    Defuse,
}

#[derive(Debug, Deserialize)]
pub struct MapTeamStatus {
    score: usize,
    consecutive_round_losses: usize,
    timeouts_remaining: usize,
    matches_won_this_series: usize,
}

#[derive(Debug, Deserialize)]
pub enum MapRoundWins {
    #[serde(alias = "t_win_elimination")]
    TWinElimination,
    #[serde(alias = "ct_win_elimination")]
    CtWinElimination,
    #[serde(alias = "t_win_bomb")]
    TWinBomb,
    #[serde(alias = "t_win_time")]
    TWinTime,
    #[serde(alias = "ct_win_time")]
    CtWinTime,
    #[serde(alias = "ct_win_defuse")]
    CtWinDefuse,
    #[serde(alias = "ct_win_rescue")]
    CtWinRescue,
}

#[derive(Debug, Deserialize)]
pub struct Map {
    mode: MapMode,
    name: String,
    phase: Phase,
    round: usize,
    team_ct: MapTeamStatus,
    team_t: MapTeamStatus,
    num_matches_to_win_series: usize,
    round_wins: Option<BTreeMap<usize, MapRoundWins>>,
}

impl Map {
    pub fn mode(&self) -> MapMode {
        *&self.mode
    }
}

impl MapMode {
    pub fn time_limit(&self) -> u8 {
        match self {
            MapMode::Competitive => 115,
            MapMode::Wingman => 90,
            MapMode::Casual => 135,
            _ => 0,
        }
    }
}

impl StateCollector for Map {
    type Inner = Map;

    fn vtable() -> &'static [&'static dyn StateAssign<Inner = Self::Inner>] {
        &[&(
            State::MapMode as StateSetter<MapMode>,
            Map::mode as StateGetter<Map, MapMode>,
        )]
    }

    fn inner(&self) -> &Self::Inner {
        self
    }
}

impl FromState for MapMode {
    fn from_state(state: &State) -> Option<&Maybe<MapMode>> {
        match state {
            State::MapMode(m) => Some(m),
            _ => None,
        }
    }
}
