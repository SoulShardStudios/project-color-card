use std::ops::Not;

use crate::cards::CardType;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect, States, Default, Debug, Hash)]
#[repr(u32)]
pub enum Team {
    #[default]
    Red,
    Blue,
}

impl Not for Team {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Blue => Self::Red,
            Self::Red => Self::Blue,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect, States, Default, Debug, Hash)]
pub struct CurrentTurnTeam(pub Team);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash, States, Reflect)]
pub struct NextTurnCardType(pub CardType);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect, Debug)]
#[repr(u32)]
pub enum CardSlotType {
    Hand,
    Play,
}

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect, Debug)]
pub struct CardSlot {
    pub id: usize,
    pub team: Team,
    pub slot_type: CardSlotType,
}

// TODO: rework systems to not have this hack
#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect, Debug)]
pub struct ButtonCardSlot(pub CardSlot);

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect, Debug)]
pub struct CardStats {
    pub hp: Option<u32>,
}

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub struct CardDeckMarker;

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub struct DiscardMarker;

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub struct RedHealthMarker;

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub struct BlueHealthMarker;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States, Reflect)]
pub enum TurnState {
    #[default]
    DrawCards,
    PlayCards,
    ApplyMoves,
}
