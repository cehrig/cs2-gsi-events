use crate::state::Event::{AmmoLow, PlayingStarted, PlayingStopped};
use crate::state::{
    Event, FromState, Maybe, State, StateAssign, StateCollector, StateCollectorDyn, StateEvents,
    StateGetter, StateSetter, States,
};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum PlayerActivity {
    #[serde(alias = "menu")]
    Menu,
    #[serde(alias = "textinput")]
    Console,
    #[serde(alias = "playing")]
    Playing,
}

#[derive(Debug, Deserialize)]
pub enum PlayerTeam {
    #[serde(alias = "CT")]
    Ct,
    #[serde(alias = "T")]
    T,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WeaponType {
    #[serde(alias = "Knife")]
    Knife,
    #[serde(alias = "Pistol")]
    Pistol {
        #[serde(flatten)]
        ammo: Ammo,
    },
    #[serde(alias = "Machine Gun")]
    MachineGun {
        #[serde(flatten)]
        ammo: Ammo,
    },
    #[serde(alias = "Submachine Gun")]
    SubmachineGun {
        #[serde(flatten)]
        ammo: Ammo,
    },
    #[serde(alias = "Rifle")]
    Rifle {
        #[serde(flatten)]
        ammo: Ammo,
    },
    #[serde(alias = "SniperRifle")]
    SniperRifle {
        #[serde(flatten)]
        ammo: Ammo,
    },
    #[serde(alias = "Shotgun")]
    Shotgun {
        #[serde(flatten)]
        ammo: Ammo,
    },
    #[serde(alias = "Grenade")]
    Grenade,
    #[serde(alias = "C4")]
    C4,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub enum WeaponState {
    #[serde(alias = "holstered")]
    Holstered,
    #[serde(alias = "active")]
    Active,
    #[serde(alias = "reloading")]
    Reloading,
}

#[derive(Copy, Clone, Default, Debug, Deserialize, PartialEq, Eq)]
pub struct Ammo {
    pub ammo_clip: u32,
    pub ammo_clip_max: u32,
    pub ammo_reserve: u32,
}

#[derive(Debug, Deserialize)]
pub struct Weapon {
    name: String,
    paintkit: String,
    state: WeaponState,
    #[serde(flatten, deserialize_with = "deserialize_weapon_type")]
    kind: WeaponType,
}

#[derive(Debug, Deserialize)]
pub struct PlayerState {
    health: u8,
    armor: u8,
    helmet: bool,
    flashed: u8,
    smoked: u8,
    burning: u8,
    money: u16,
    round_kills: u8,
    round_killhs: u8,
    equip_value: u16,
}

#[derive(Debug, Deserialize)]
pub struct Weapons {
    #[serde(flatten)]
    weapons: BTreeMap<String, Weapon>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    steamid: String,
    name: String,
    activity: Option<PlayerActivity>,
    xpoverload: Option<usize>,
    observer_slot: Option<usize>,
    team: Option<PlayerTeam>,
    state: Option<PlayerState>,
    weapons: Option<Weapons>,
}

impl Weapon {
    fn ammo(&self) -> Ammo {
        match self.kind {
            WeaponType::Knife => Ammo::default(),
            WeaponType::Pistol { ref ammo } => *ammo,
            WeaponType::MachineGun { ref ammo } => *ammo,
            WeaponType::SubmachineGun { ref ammo } => *ammo,
            WeaponType::Rifle { ref ammo } => *ammo,
            WeaponType::SniperRifle { ref ammo } => *ammo,
            WeaponType::Shotgun { ref ammo } => *ammo,
            WeaponType::Grenade => Ammo::default(),
            WeaponType::C4 => Ammo::default(),
            WeaponType::Unknown => Ammo::default(),
        }
    }

    fn is_active(&self) -> bool {
        matches!(self.state, WeaponState::Active)
    }
}

impl Weapons {
    fn ammo(&self) -> Ammo {
        for weapon in self.weapons.values() {
            if weapon.is_active() {
                return weapon.ammo();
            }
        }

        Ammo::default()
    }
}

impl PlayerActivity {
    pub fn activity(&self) -> PlayerActivity {
        *self
    }
}

impl StateCollector for Player {
    type Inner = Player;

    fn next(s: Option<&Self>) -> Vec<&dyn StateCollectorDyn> {
        let r = s.as_ref();
        vec![
            r.map(|p| &p.activity).unwrap_or(&None),
            r.map(|p| &p.weapons).unwrap_or(&None),
        ]
    }

    fn inner(&self) -> &Self::Inner {
        self
    }
}

impl StateCollector for PlayerActivity {
    type Inner = PlayerActivity;

    fn vtable() -> &'static [&'static dyn StateAssign<Inner = Self::Inner>] {
        &[&(
            State::PlayerActivity as StateSetter<PlayerActivity>,
            PlayerActivity::activity as StateGetter<Self::Inner, PlayerActivity>,
        )]
    }

    fn inner(&self) -> &Self::Inner {
        self
    }
}

impl StateCollector for Weapons {
    type Inner = Weapons;

    fn vtable() -> &'static [&'static dyn StateAssign<Inner = Self::Inner>] {
        &[&(
            State::Ammo as StateSetter<Ammo>,
            Weapons::ammo as StateGetter<Self::Inner, Ammo>,
        )]
    }

    fn inner(&self) -> &Self::Inner {
        self
    }
}

impl FromState for PlayerActivity {
    fn from_state(state: &State) -> Option<&Maybe<PlayerActivity>> {
        match state {
            State::PlayerActivity(a) => Some(a),
            _ => None,
        }
    }
}

impl FromState for Ammo {
    fn from_state(state: &State) -> Option<&Maybe<Ammo>> {
        match state {
            State::Ammo(a) => Some(a),
            _ => None,
        }
    }
}

impl StateEvents for Maybe<PlayerActivity> {
    fn compare(&self, _: &Self, _: &States) -> Vec<Event> {
        if !matches!(self, Maybe::Set(PlayerActivity::Playing)) {
            return vec![PlayingStopped];
        }

        if matches!(self, Maybe::Set(PlayerActivity::Playing)) {
            return vec![PlayingStarted];
        }

        [].into()
    }
}

impl StateEvents for Maybe<Ammo> {
    fn compare(&self, previous: &Self, _: &States) -> Vec<Event> {
        let mut events = vec![];

        if let Maybe::Set(current) = self {
            events.push(Event::Ammo(*current));
        };

        if let Maybe::Set(current) = self
            && let Maybe::Set(previous) = previous
        {
            if current != previous
                && (current.ammo_clip as f64 <= current.ammo_clip_max as f64 * 0.20
                    || current.ammo_clip == 3)
                && current.ammo_clip != 0
            {
                events.push(AmmoLow);
            }
        }

        events
    }
}

fn deserialize_weapon_type<'de, D>(deserializer: D) -> Result<WeaponType, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<WeaponType> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or(WeaponType::Unknown))
}
