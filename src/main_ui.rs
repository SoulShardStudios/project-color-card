use crate::card::Card;
use bevy::{prelude::*, render::render_graph::SlotType};
use bevy_rand::resource;
use std::collections::BTreeMap;
use std::sync::Arc;

const CARD_SLOT_COUNT: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Team {
    Red,
    Blue,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CardSlotType {
    Hand,
    Play,
}

#[derive(Component)]
pub struct CardSlot {
    pub id: u32,
    pub team: Team,
    pub slot_type: CardSlotType,
}

pub fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                ..default()
            },
            background_color: BackgroundColor(*color),
            ..default()
        })
        .with_children(|parent| {
            for id in 0..5 {
                parent
                    .spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(72.0 * 2.0),
                            height: Val::Px(102.0 * 2.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
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

#[derive(Resource)]
pub struct GameUIController<'a> {
    card_names: BTreeMap<(Team, CardSlotType), Vec<Option<&'a str>>>,
}

impl<'a> GameUIController<'a> {
    pub fn new() -> Self {
        let mut map: BTreeMap<(Team, CardSlotType), Vec<Option<&'a str>>> = BTreeMap::new();
        for team in [Team::Blue, Team::Red] {
            for slot_type in [CardSlotType::Hand, CardSlotType::Play] {
                map.insert((team, slot_type), Vec::with_capacity(CARD_SLOT_COUNT));
            }
        }
        GameUIController { card_names: map }
    }
    pub fn get_card_id(self, team: Team, slot_type: CardSlotType, id: usize) -> Option<&'a str> {
        self.card_names[&(team, slot_type)][id].clone()
    }
    pub fn set_card(
        mut self,
        mut query: Query<(&CardSlot, &mut UiImage)>,
        team: Team,
        slot_type: CardSlotType,
        id: usize,
        card: &'a Card,
    ) {
        query
            .iter_mut()
            .filter(|(x, _)| x.slot_type == slot_type && x.team == team)
            .nth(id)
            .unwrap()
            .1
            .texture = card.image_handle.clone();
        self.card_names.get_mut(&(team, slot_type)).unwrap()[id] = Some(&card.name);
    }
}

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_game_ui)
            .insert_resource(GameUIController::new());
    }
}
