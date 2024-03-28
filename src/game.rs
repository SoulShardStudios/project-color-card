use crate::card::{Card, CardAssetPlugin};
use bevy::prelude::*;
use bevy_rand::prelude::{GlobalEntropy, WyRand};
use rand::{Rng, RngCore};
use std::rc::Rc;

// display: the deck -> wip, what's in the user's hand -> wip and what's on the board -> done
// user is given cards in their deck randomly at the start of the game, deck is rebalanced each time a card is removed automatically,
// user places one card and more if anything allows it cards are then
// all card names should be unique

#[derive(Clone, Copy)]
enum Team {
    Red,
    Blue,
}
#[derive(Clone, Copy)]
enum CardSlotType {
    Hand,
    Play,
}

#[derive(Component)]
struct CardSlot {
    pub id: u32,
    pub team: Team,
    pub slot_type: CardSlotType,
}

#[derive(Resource)]
pub struct CardManager {
    valid_new_cards: Vec<AssetId<Card>>,
}

impl CardManager {
    pub fn new(cards: &Res<Assets<Card>>) -> Self {
        CardManager {
            valid_new_cards: cards
                .iter()
                .filter(|(_id, card)| -> bool { card.colors.len() < 3 })
                .map(|x| -> AssetId<Card> { x.0 })
                .collect(),
        }
    }

    pub fn get_random_card(&self, mut rng: ResMut<GlobalEntropy<WyRand>>) -> AssetId<Card> {
        return self.valid_new_cards[rng.gen_range(0usize..self.valid_new_cards.len())];
    }
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
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(50.0),
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            background_color: BackgroundColor(Color::rgb(1.0, 0.5, 0.5)),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_slots_for_team(parent, &Team::Red, &CardSlotType::Hand)
                        });
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(50.0),
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            background_color: BackgroundColor(Color::rgb(0.5, 0.5, 1.0)),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_slots_for_team(parent, &Team::Blue, &CardSlotType::Hand)
                        });
                });
        });
}

fn spawn_slots_for_team<'a>(parent: &mut ChildBuilder<'a>, team: &Team, slot_type: &CardSlotType) {
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
                team: team.clone(),
                slot_type: slot_type.clone(),
            });
    }
}
