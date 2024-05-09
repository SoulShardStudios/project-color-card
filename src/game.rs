use crate::assets::LoadState;
use crate::cards::{Card, CardAssetPlugin, CardBack, CardBackAssetPlugin, CardBackType, CardType};
use bevy::ecs::query::{QueryData, QueryFilter, QueryIter};
use bevy::prelude::*;
use bevy::utils::hashbrown::Equivalent;
use bevy_rand::prelude::WyRand;
use bevy_rand::resource::GlobalEntropy;
use itertools::Itertools;
use num_traits::FromPrimitive;
use rand::Rng;
use std::collections::BTreeMap;
use std::default;

trait Itertools2: Itertools {}

impl<'world, 'state, D: QueryData, F: QueryFilter> Itertools2 for QueryIter<'world, 'state, D, F> {}

const CARD_SLOT_COUNT: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect, States, Default, Debug, Hash)]
#[repr(u32)]
pub enum Team {
    #[default]
    Red,
    Blue,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect, States, Default, Debug, Hash)]
pub struct CurrentTurnTeam(Team);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect, Debug)]
#[repr(u32)]
pub enum CardSlotType {
    Hand,
    Play,
}

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub struct CardSlot {
    pub id: usize,
    pub team: Team,
    pub slot_type: CardSlotType,
}

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub struct CardDeckMarker;

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub struct DiscardMarker;

pub fn setup_game_ui(mut commands: Commands, card_backs: Res<Assets<CardBack>>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(10.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| spawn_card_piles(parent, &card_backs));
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(90.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    spawn_slots_for_team(
                        parent,
                        Team::Blue,
                        CardSlotType::Hand,
                        &Color::rgb(0.5, 0.5, 1.0),
                    );
                    spawn_slots_for_team(
                        parent,
                        Team::Blue,
                        CardSlotType::Play,
                        &Color::rgb(0.7, 0.7, 1.0),
                    );
                    spawn_slots_for_team(
                        parent,
                        Team::Red,
                        CardSlotType::Play,
                        &Color::rgb(1.0, 0.7, 0.7),
                    );
                    spawn_slots_for_team(
                        parent,
                        Team::Red,
                        CardSlotType::Hand,
                        &Color::rgb(1.0, 0.5, 0.5),
                    );
                });
        });
}

fn spawn_slots_for_team<'a>(
    parent: &mut ChildBuilder<'a>,
    team: Team,
    slot_type: CardSlotType,
    color: &Color,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(25.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(*color),
            ..default()
        })
        .with_children(|parent| {
            for id in 0..CARD_SLOT_COUNT {
                parent
                    .spawn(ImageBundle {
                        style: Style {
                            height: Val::Percent(100.0),
                            aspect_ratio: Some(72.0 / 102.0),
                            ..default()
                        },
                        ..default()
                    })
                    .insert(CardSlot {
                        id: id,
                        team: team,
                        slot_type: slot_type,
                    });
            }
        });
}

fn spawn_card_piles<'a>(parent: &mut ChildBuilder<'a>, card_backs: &Res<Assets<CardBack>>) {
    let discard_back = card_backs
        .iter()
        .filter(|(_, back)| back.card_type == CardBackType::Discard)
        .nth(0)
        .map(|x| x.1.image_handle.clone())
        .unwrap_or(Handle::default());
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Percent(100.0),
                aspect_ratio: Some(72.0 / 102.0),
                ..default()
            },
            ..default()
        })
        .insert(CardDeckMarker);
    parent
        .spawn(ImageBundle {
            style: Style {
                width: Val::Percent(100.0),
                aspect_ratio: Some(72.0 / 102.0),

                ..default()
            },
            image: UiImage {
                texture: discard_back,
                ..default()
            },
            ..default()
        })
        .insert(DiscardMarker);
}

pub fn draw_card(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut game_ui_controller: Query<&mut GameUIController>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    cards: Res<Assets<Card>>,
    mut next_card_type_state: ResMut<NextState<NextTurnCardType>>,
    card_type_state: Res<State<NextTurnCardType>>,
    current_turn_team: Res<State<CurrentTurnTeam>>,
    mut turn_state: ResMut<NextState<TurnState>>,
    current_turn_state: Res<State<TurnState>>,
) {
    if *current_turn_state.get() != TurnState::DrawCards {
        return;
    }
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mut controller = game_ui_controller.iter_mut().nth(0).unwrap();
                let random_card_of_type = controller.get_random_card_of_type(
                    &mut rng,
                    &cards,
                    card_type_state.get().0.clone(),
                );
                controller.push_card(
                    CardSlotType::Hand,
                    current_turn_team.0,
                    random_card_of_type.unwrap(),
                );
                next_card_type_state.set(NextTurnCardType(
                    CardType::from_i8(rng.gen_range(0..4)).unwrap(),
                ));
                turn_state.set(TurnState::PlayCards);
            }
            _ => {}
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash, States, Reflect)]
pub struct NextTurnCardType(CardType);

#[derive(Component)]
pub struct GameUIController {
    current_cards: BTreeMap<CardSlot, Option<AssetId<Card>>>,
    valid_new_cards: Vec<AssetId<Card>>,
    push_card_actions: Vec<(Team, CardSlotType, AssetId<Card>)>,
    take_card_actions: Vec<CardSlot>,
}

impl GameUIController {
    pub fn new(cards: &Res<Assets<Card>>) -> Self {
        let mut card_names: BTreeMap<CardSlot, Option<AssetId<Card>>> = BTreeMap::new();
        for team in [Team::Blue, Team::Red] {
            for slot_type in [CardSlotType::Hand, CardSlotType::Play] {
                for slot_id in 0..CARD_SLOT_COUNT {
                    card_names.insert(
                        CardSlot {
                            id: slot_id,
                            slot_type: slot_type,
                            team: team,
                        },
                        None,
                    );
                }
            }
        }
        let valid_new_cards = cards
            .iter()
            .filter(|(_id, card)| -> bool { card.colors.len() < 3 })
            .map(|x| -> AssetId<Card> { x.0 })
            .collect();
        GameUIController {
            current_cards: card_names,
            valid_new_cards,
            push_card_actions: vec![],
            take_card_actions: vec![],
        }
    }
    pub fn get_card_id(&self, slot: &CardSlot) -> Option<AssetId<Card>> {
        self.current_cards[slot].clone()
    }

    pub fn push_card<'a>(&mut self, slot_type: CardSlotType, team: Team, card: AssetId<Card>) {
        self.push_card_actions.push((team, slot_type, card));
    }

    pub fn take_card(&mut self, slot: &CardSlot) {
        self.take_card_actions.push(slot.clone());
    }

    pub fn get_random_card(
        &self,
        rng: &mut ResMut<GlobalEntropy<WyRand>>,
    ) -> Option<AssetId<Card>> {
        if self.valid_new_cards.len() == 0 {
            return None;
        }
        return Some(self.valid_new_cards[rng.gen_range(0usize..self.valid_new_cards.len() - 1)]);
    }

    pub fn get_random_card_of_type(
        &self,
        rng: &mut ResMut<GlobalEntropy<WyRand>>,
        cards: &Res<Assets<Card>>,
        card_type: CardType,
    ) -> Option<AssetId<Card>> {
        loop {
            if let Some(card_id) = self.get_random_card(rng) {
                if let Some(card) = cards.get(card_id) {
                    if card.card_type == card_type {
                        return Some(card_id);
                    }
                }
            } else {
                return None;
            }
        }
    }
}

fn set_cards(
    mut game_ui_controller_query: Query<&mut GameUIController>,
    cards: Res<Assets<Card>>,
    mut query: Query<(&CardSlot, &mut UiImage)>,
) {
    let mut game_ui_controller = match game_ui_controller_query.iter_mut().nth(0) {
        None => {
            return;
        }
        Some(x) => x,
    };
    println!("{:#?}", game_ui_controller.push_card_actions.clone());
    for (team, slot_type, card) in game_ui_controller.push_card_actions.clone() {
        let card_asset = cards.get(card).unwrap();
        for (slot, mut ui) in query
            .iter_mut()
            .filter(|(x, _)| x.team == team && x.slot_type == slot_type)
        {
            if game_ui_controller.get_card_id(slot).is_some() {
                continue;
            }
            ui.texture = card_asset.image_handle.clone();
            game_ui_controller.current_cards.remove(&slot);
            game_ui_controller
                .current_cards
                .insert(slot.clone(), Some(card));
            break;
        }
    }
    game_ui_controller.push_card_actions.clear();
}

fn take_cards(
    mut game_ui_controller_query: Query<&mut GameUIController>,
    mut query: Query<(&CardSlot, &mut UiImage)>,
) {
    let mut game_ui_controller = match game_ui_controller_query.iter_mut().nth(0) {
        None => {
            return;
        }
        Some(x) => x,
    };
    // apply take actions
    for slot in game_ui_controller.take_card_actions.clone() {
        query
            .iter_mut()
            .filter(|(x, _)| **x == slot)
            .nth(0)
            .unwrap()
            .1
            .texture = Handle::default();
        game_ui_controller.current_cards.remove(&slot);
        game_ui_controller.current_cards.insert(slot.clone(), None);
        // reset the stack
        let textures: Vec<_> = query
            .iter()
            .map(|(_, image)| image.texture.clone())
            .collect();
        for (slot, mut ui) in query
            .iter_mut()
            .filter(|(x, _)| x.team == slot.team && x.slot_type == slot.slot_type)
        {
            if game_ui_controller.get_card_id(slot).is_some() {
                continue;
            }
            let next_slot = CardSlot {
                id: slot.id + 1,
                slot_type: slot.slot_type,
                team: slot.team,
            };
            let next_slot_card = game_ui_controller.get_card_id(&next_slot);
            ui.texture = textures[next_slot.id].clone();
            game_ui_controller.current_cards.remove(&slot);
            game_ui_controller
                .current_cards
                .insert(slot.clone(), next_slot_card);
        }
    }

    game_ui_controller.take_card_actions.clear();
}

pub struct GameUIPlugin;

fn spawn_game_ui_controller(mut commands: Commands, cards: Res<Assets<Card>>) {
    commands.spawn(GameUIController::new(&cards));
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States, Reflect)]
pub enum TurnState {
    #[default]
    DrawCards,
    PlayCards,
    ApplyMoves,
}

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CardSlot>()
            .register_type::<CardSlotType>()
            .register_type::<Team>()
            .init_state::<TurnState>()
            .init_state::<NextTurnCardType>()
            .init_state::<CurrentTurnTeam>()
            .add_plugins(CardAssetPlugin)
            .add_plugins(CardBackAssetPlugin)
            .add_systems(
                OnEnter(LoadState::Loaded),
                (setup_game_ui, spawn_game_ui_controller),
            )
            .add_systems(Update, (set_cards, take_cards, draw_card));
    }
}
