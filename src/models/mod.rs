pub mod map;
pub mod player;
pub mod provider;
pub mod round;

use crate::models::map::Map;
use crate::models::player::Player;
use crate::models::provider::Provider;
use crate::models::round::Round;
use crate::state::{StateAssign, StateCollector, StateCollectorDyn, States};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GameState {
    provider: Provider,
    player: Option<Player>,
    map: Option<Map>,
    round: Option<Round>,
}

impl GameState {
    pub fn get(&self) -> States {
        let mut states = States::default();
        self.states(&mut states);

        states
    }
}

impl StateCollector for GameState {
    type Inner = GameState;

    fn vtable() -> &'static [&'static dyn StateAssign<Inner = Self::Inner>] {
        &[]
    }

    fn next(s: Option<&Self>) -> Vec<&dyn StateCollectorDyn> {
        let r = s.as_ref();
        vec![
            r.map(|p| &p.player).unwrap_or(&None),
            r.map(|p| &p.round).unwrap_or(&None),
            r.map(|p| &p.map).unwrap_or(&None),
        ]
    }

    fn inner(&self) -> &GameState {
        self
    }
}
