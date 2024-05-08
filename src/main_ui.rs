use crate::card::{Card, CardAssetPlugin};
use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use bevy_rand::prelude::WyRand;
use bevy_rand::resource::GlobalEntropy;
use rand::Rng;
use std::collections::BTreeMap;

const CARD_SLOT_COUNT: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[repr(u32)]
pub enum Team {
    Red,
    Blue,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
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

pub fn setup_game_ui(mut commands: Commands) {
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
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(10.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            });

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

#[derive(Component)]
pub struct GameUIController {
    card_names: BTreeMap<CardSlot, Option<AssetId<Card>>>,
    valid_new_cards: Vec<AssetId<Card>>,
    set_card_actions: Vec<(CardSlot, AssetId<Card>)>,
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
            card_names,
            valid_new_cards,
            set_card_actions: vec![],
        }
    }
    pub fn get_card_id(&self, slot: &CardSlot) -> Option<AssetId<Card>> {
        self.card_names[slot].clone()
    }
    pub fn set_card<'a>(&mut self, slot: &'a CardSlot, card: AssetId<Card>) {
        self.set_card_actions.push((slot.clone(), card));
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
    for (slot, card) in game_ui_controller.set_card_actions.clone() {
        let card_asset = cards.get(card).unwrap();
        query
            .iter_mut()
            .filter(|(x, _)| **x == slot)
            .nth(slot.id)
            .unwrap()
            .1
            .texture = card_asset.image_handle.clone();
        game_ui_controller.card_names.remove(&slot);
        game_ui_controller
            .card_names
            .insert(slot.clone(), Some(card));
    }
    game_ui_controller.set_card_actions.clear();
}

pub struct GameUIPlugin;

#[derive(Resource, Default, Reflect)]
struct Cards(Handle<LoadedFolder>);

fn load_cards(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Cards(asset_server.load_folder("cards")));
}
fn spawn_game_ui_controller(mut commands: Commands, cards: Res<Assets<Card>>) {
    commands.spawn(GameUIController::new(&cards));
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States, Reflect)]
pub enum GameState {
    #[default]
    DrawCards,
    PlayCards,
    ApplyMoves,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States, Reflect)]
pub enum LoadState {
    #[default]
    Unloaded,
    Loaded,
}

fn check_assets_folder_loaded(
    mut next_state: ResMut<NextState<LoadState>>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
    cards: Res<Cards>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(cards.0.clone()) {
            next_state.set(LoadState::Loaded)
        }
    }
}

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins(CardAssetPlugin)
            .init_state::<LoadState>()
            .add_systems(OnEnter(LoadState::Unloaded), load_cards)
            .add_systems(
                Update,
                check_assets_folder_loaded.run_if(in_state(LoadState::Unloaded)),
            )
            .add_systems(
                OnEnter(LoadState::Loaded),
                (setup_game_ui, spawn_game_ui_controller),
            )
            .add_systems(Update, set_cards)
            .register_type::<CardSlot>()
            .register_type::<CardSlotType>()
            .register_type::<Team>();
    }
}
