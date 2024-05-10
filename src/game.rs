use crate::assets::LoadState;
use crate::cards::{Card, CardAssetPlugin, CardBack, CardBackAssetPlugin, CardBackType, CardType};
use crate::custom_cursor::{CustomCursor, CustomCursorPlugin};
use bevy::ecs::query::{QueryData, QueryFilter, QueryIter};
use bevy::prelude::*;
use bevy_rand::prelude::WyRand;
use bevy_rand::resource::GlobalEntropy;
use itertools::Itertools;
use num_traits::FromPrimitive;
use rand::Rng;
use std::collections::BTreeMap;

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

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect, Debug)]
pub struct CardSlot {
    pub id: usize,
    pub team: Team,
    pub slot_type: CardSlotType,
}

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub struct CardDeckMarker;

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub struct DiscardMarker;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States, Reflect)]
pub enum TurnState {
    #[default]
    DrawCards,
    PlayCards,
    ApplyMoves,
}

pub fn spawn_game_ui(
    mut commands: Commands,
    card_backs: Res<Assets<CardBack>>,
    card_type_state: Res<State<NextTurnCardType>>,
) {
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
                .with_children(|parent| spawn_card_piles(parent, &card_backs, &card_type_state));
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
                    .spawn(ButtonBundle {
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

fn spawn_card_piles<'a>(
    parent: &mut ChildBuilder<'a>,
    card_backs: &Res<Assets<CardBack>>,
    card_type_state: &Res<State<NextTurnCardType>>,
) {
    let discard_back = get_card_back_image(card_backs, CardBackType::Discard);
    let current_back = get_card_back_image(
        card_backs,
        CardBackType::CardType(card_type_state.get().0.clone()),
    );
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Percent(100.0),
                aspect_ratio: Some(72.0 / 102.0),
                ..default()
            },
            image: UiImage {
                texture: current_back,
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

fn get_card_back_image(
    card_backs: &Res<Assets<CardBack>>,
    back_type: CardBackType,
) -> Handle<Image> {
    return card_backs
        .iter()
        .filter(|(_, back)| back.card_type == back_type)
        .nth(0)
        .map(|x| x.1.image_handle.clone())
        .unwrap_or(Handle::default());
}

pub fn draw_card(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<CardDeckMarker>),
    >,
    mut game_ui_controller_query: Query<&mut GameUIController>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    cards: Res<Assets<Card>>,
    current_card_type_state: Res<State<NextTurnCardType>>,
    mut card_type_state: ResMut<NextState<NextTurnCardType>>,
    current_turn_team: Res<State<CurrentTurnTeam>>,
    mut turn_state: ResMut<NextState<TurnState>>,
    current_turn_state: Res<State<TurnState>>,
    mut draw_image_query: Query<&mut UiImage, With<CardDeckMarker>>,
    card_backs: Res<Assets<CardBack>>,
) {
    if *current_turn_state.get() != TurnState::DrawCards {
        return;
    }
    let mut game_ui_controller = match game_ui_controller_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let random_card_of_type = game_ui_controller.get_random_card_of_type(
                    &mut rng,
                    &cards,
                    current_card_type_state.get().0.clone(),
                );
                game_ui_controller.push_card_at(
                    CardSlotType::Hand,
                    current_turn_team.0,
                    random_card_of_type.unwrap(),
                    None,
                );
                let new_card_type = CardType::from_i8(rng.gen_range(0..4)).unwrap();
                card_type_state.set(NextTurnCardType(new_card_type));
                draw_image_query.iter_mut().nth(0).unwrap().texture =
                    get_card_back_image(&card_backs, CardBackType::CardType(new_card_type));
                turn_state.set(TurnState::PlayCards);
            }
            _ => {}
        }
    }
}

fn play_card(
    mut game_ui_controller_query: Query<&mut GameUIController>,
    mut custom_cursor_query: Query<&mut CustomCursor>,
    mut interaction_query: Query<
        (&Interaction, &CardSlot),
        (Changed<Interaction>, With<Button>, With<CardSlot>),
    >,
    current_turn_state: Res<State<TurnState>>,
    mut turn_state: ResMut<NextState<TurnState>>,

    current_turn_team: Res<State<CurrentTurnTeam>>,
) {
    if *current_turn_state.get() != TurnState::PlayCards {
        return;
    }
    let mut game_ui_controller = match game_ui_controller_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    if game_ui_controller.card_stack_full(current_turn_team.get().0, CardSlotType::Play) {
        turn_state.set(TurnState::ApplyMoves);
        return;
    }
    let mut custom_cursor = match custom_cursor_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    match custom_cursor.get_current_card() {
        // pick up card and set custom cursor
        None => {
            for (interaction, slot) in &mut interaction_query {
                if !(slot.team == current_turn_team.get().0 && slot.slot_type == CardSlotType::Hand)
                {
                    continue;
                }
                match *interaction {
                    Interaction::Pressed => match game_ui_controller.get_card_id(slot) {
                        Some(card) => {
                            custom_cursor.set_card(card);
                            game_ui_controller.take_card(slot);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        // place custom cursor down in play and adjust slots
        Some(cursor_card) => {
            for (interaction, slot) in &mut interaction_query {
                if !(slot.team == current_turn_team.get().0 && slot.slot_type == CardSlotType::Play)
                {
                    continue;
                }
                match *interaction {
                    Interaction::Pressed => {
                        game_ui_controller.push_card_at(
                            slot.slot_type,
                            slot.team,
                            cursor_card,
                            Some(slot.id),
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash, States, Reflect)]
pub struct NextTurnCardType(CardType);

#[derive(Component)]
pub struct GameUIController {
    current_cards: BTreeMap<CardSlot, Option<AssetId<Card>>>,
    valid_new_cards: Vec<AssetId<Card>>,
    push_card_actions: Vec<(Team, CardSlotType, AssetId<Card>, Option<usize>)>,
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
    pub fn card_stack_full(&self, team: Team, slot_type: CardSlotType) -> bool {
        for slot in (0..CARD_SLOT_COUNT).into_iter().map(|id| CardSlot {
            team: team,
            slot_type: slot_type,
            id: id,
        }) {
            if self.get_card_id(&slot).is_none() {
                return false;
            }
        }
        return true;
    }

    pub fn get_card_id(&self, slot: &CardSlot) -> Option<AssetId<Card>> {
        self.current_cards[slot].clone()
    }

    pub fn push_card_at<'a>(
        &mut self,
        slot_type: CardSlotType,
        team: Team,
        card: AssetId<Card>,
        slot_id: Option<usize>,
    ) {
        self.push_card_actions
            .push((team, slot_type, card, slot_id));
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
    for (team, slot_type, card, id) in game_ui_controller.push_card_actions.clone() {
        let card_asset = cards.get(card).unwrap();

        let textures: Vec<_> = query
            .iter()
            .map(|(_, image)| image.texture.clone())
            .collect();
        let slots_and_ui: Vec<_> = query
            .iter_mut()
            .filter(|(x, _)| x.team == team && x.slot_type == slot_type)
            .collect();

        match id {
            None => {
                for (slot, mut ui) in slots_and_ui {
                    if game_ui_controller.get_card_id(slot).is_some() {
                        // should we shift the stack?

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
            Some(mut slot_id) => {
                let first_empty_slot = slots_and_ui
                    .iter()
                    .map(|x| x.0)
                    .find(|x| game_ui_controller.get_card_id(x).is_none());
                if slot_id > first_empty_slot.unwrap().id {
                    slot_id = first_empty_slot.unwrap().id;
                }

                for (slot, mut ui) in slots_and_ui.into_iter().rev() {
                    if slot.id > first_empty_slot.unwrap().id {
                        continue;
                    }
                    if slot.id == slot_id {
                        ui.texture = card_asset.image_handle.clone();
                        game_ui_controller.current_cards.remove(&slot);
                        game_ui_controller
                            .current_cards
                            .insert(slot.clone().clone(), Some(card));
                        break;
                    }
                    ui.texture = textures[slot.id - 1].clone();
                }
            }
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
            if slot.id == 7 {
                continue;
            }
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

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CardSlot>()
            .register_type::<CardSlotType>()
            .register_type::<Team>()
            .init_state::<TurnState>()
            .init_state::<NextTurnCardType>()
            .init_state::<CurrentTurnTeam>()
            .add_plugins(CustomCursorPlugin)
            .add_plugins(CardAssetPlugin)
            .add_plugins(CardBackAssetPlugin)
            .add_systems(
                OnEnter(LoadState::Loaded),
                (spawn_game_ui_controller, spawn_game_ui),
            )
            .add_systems(Update, (set_cards, take_cards, draw_card, play_card));
    }
}
