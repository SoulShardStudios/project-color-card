use crate::assets::LoadState;
use crate::cards::{
    get_card_back_image, Card, CardAssetPlugin, CardBack, CardBackAssetPlugin, CardBackType,
    CardType,
};
use crate::constants::CARD_SLOT_COUNT;
use crate::custom_cursor::{CustomCursor, CustomCursorPlugin};
use crate::game_state::{
    CardDeckMarker, CardHealth, CardSlot, CardSlotType, CurrentTurnTeam, DiscardMarker,
    NextTurnCardType, Team, TurnState,
};
use bevy::prelude::*;
use bevy_rand::prelude::WyRand;
use bevy_rand::resource::GlobalEntropy;
use rand::Rng;
use std::collections::BTreeMap;

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
