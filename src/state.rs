use crate::models::map::{MapMode, Phase};
use crate::models::player::{Ammo, PlayerActivity, PlayerState};
use crate::models::round::BombStatus;

pub type StateSetter<T> = fn(Maybe<T>) -> State;
pub type StateGetter<I, T> = fn(&I) -> T;

pub trait StateCollector {
    type Inner;

    fn vtable() -> &'static [&'static dyn StateAssign<Inner = Self::Inner>] {
        &[]
    }

    fn next(_: Option<&Self>) -> Vec<&dyn StateCollectorDyn> {
        vec![]
    }

    fn states(&self, states: &mut States)
    where
        <Self as StateCollector>::Inner: 'static,
    {
        Self::vtable()
            .iter()
            .for_each(|s| states.add(s.get(self.inner())));

        for next in Self::next(Some(self)) {
            next.collect(states)
        }
    }

    fn inner(&self) -> &Self::Inner;
}

pub trait StateCollectorDyn {
    fn collect(&self, states: &mut States);
}

pub trait StateAssign {
    type Inner;
    fn get(&self, inner: &Self::Inner) -> State;

    fn unknown(&self) -> State;
}

pub trait StateEvents {
    fn compare(&self, previous: &Self, states: &States) -> Vec<Event>;
}

#[derive(Default, Debug)]
pub struct States {
    inner: Vec<State>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    RoundPhase(Maybe<Phase>),
    PlayerActivity(Maybe<PlayerActivity>),
    PlayerState(Maybe<PlayerState>),
    BombStatus(Maybe<BombStatus>),
    Ammo(Maybe<Ammo>),
    MapMode(Maybe<MapMode>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Maybe<T> {
    Unknown,
    Set(T),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    RoundStarted(u8),
    RoundOver,
    BombPlanted,
    AmmoLow,
    Ammo(Ammo),
    PlayingStopped,
    PlayingStarted,
    HealthArmorChanged((u8, u8)),
    Unknown,
}

pub trait FromState: Sized {
    fn from_state(state: &State) -> Option<&Maybe<Self>>;
}

impl States {
    pub fn inner(&self) -> &[State] {
        &self.inner
    }

    pub fn add(&mut self, event: State) {
        self.inner.push(event);
    }

    pub fn get<T: FromState>(&self) -> &Maybe<T> {
        self.inner
            .iter()
            .rev()
            .find_map(T::from_state)
            .unwrap_or(&Maybe::Unknown)
    }

    pub fn events(&self, old: &Self) -> Vec<Event> {
        let mut events = vec![];

        for state in &self.inner {
            let e = match state {
                State::BombStatus(i) => i.compare(old.get::<BombStatus>(), &self),
                State::RoundPhase(i) => i.compare(old.get::<Phase>(), &self),
                State::Ammo(i) => i.compare(old.get::<Ammo>(), &self),
                State::PlayerActivity(i) => i.compare(old.get::<PlayerActivity>(), &self),
                State::PlayerState(i) => i.compare(old.get::<PlayerState>(), &self),
                // no events implemented for the other states
                _ => continue,
            };

            events.extend(e);
        }

        events
    }
}

impl<T> StateCollector for Option<T>
where
    T: StateCollector<Inner = T> + 'static,
{
    type Inner = T;

    fn vtable() -> &'static [&'static dyn StateAssign<Inner = Self::Inner>] {
        T::vtable()
    }

    fn next(s: Option<&Self>) -> Vec<&dyn StateCollectorDyn> {
        T::next(Option::from(s.unwrap()))
    }

    fn states(&self, states: &mut States) {
        Self::vtable().iter().for_each(|s| match self {
            None => states.add(s.unknown()),
            Some(i) => states.add(s.get(i)),
        });

        for next in Self::next(Option::from(self)) {
            next.collect(states)
        }
    }

    fn inner(&self) -> &T {
        self.as_ref().unwrap()
    }
}

impl<T> StateCollectorDyn for T
where
    T: StateCollector + 'static,
{
    fn collect(&self, states: &mut States) {
        self.states(states)
    }
}

impl<T, I> StateAssign for (StateSetter<T>, StateGetter<I, T>) {
    type Inner = I;

    fn get(&self, inner: &Self::Inner) -> State {
        self.0(Maybe::Set(self.1(inner)))
    }

    fn unknown(&self) -> State {
        self.0(Maybe::Unknown)
    }
}
