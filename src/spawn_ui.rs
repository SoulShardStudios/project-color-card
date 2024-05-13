use crate::cards::{get_card_back_image, CardBack, CardBackType};
use crate::constants::CARD_SLOT_COUNT;

use crate::game_state::{
    ButtonCardSlot, CardDeckMarker, CardSlot, CardSlotType, DiscardMarker, NextTurnCardType, Team
};
use bevy::prelude::*;
use bevy::render::texture::{
    ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};

pub fn spawn_game_ui(
    mut commands: Commands,
    card_backs: Res<Assets<CardBack>>,
    card_type_state: Res<State<NextTurnCardType>>,
    assets: Res<AssetServer>,
) {
    let sampler_desc = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..Default::default()
    };

    let settings = move |s: &mut ImageLoaderSettings| {
        s.sampler = ImageSampler::Descriptor(sampler_desc.clone());
    };

    let background: Handle<Image> = assets.load_with_settings("ui/Background.png", settings);

    let font = assets.load("ui/simple-pixel.ttf");
    let slot: Handle<Image> = assets.load("ui/Slot.png");

    commands.spawn(SpriteBundle {
        texture: background,
        sprite: Sprite {
            rect: Some(Rect::new(-5000.0, -5000.0, 5000.0, 5000.0)),
            ..default()
        },
        ..default()
    });

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
                .with_children(|parent| {
                    spawn_card_piles(parent, &card_backs, &card_type_state, font.clone(), &assets)
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
                        &Color::rgba(0.0, 0.0, 0.0, 0.0),
                        slot.clone(),
                        font.clone(),
                    );
                    spawn_slots_for_team(
                        parent,
                        Team::Blue,
                        CardSlotType::Play,
                        &Color::rgba(0.0, 0.0, 0.0, 0.0),
                        slot.clone(),
                        font.clone(),
                    );
                    spawn_slots_for_team(
                        parent,
                        Team::Red,
                        CardSlotType::Play,
                        &Color::rgba(0.0, 0.0, 0.0, 0.0),
                        slot.clone(),
                        font.clone(),
                    );
                    spawn_slots_for_team(
                        parent,
                        Team::Red,
                        CardSlotType::Hand,
                        &Color::rgba(0.0, 0.0, 0.0, 0.0),
                        slot.clone(),
                        font.clone(),
                    );
                });
        });
}

fn spawn_slots_for_team<'a>(
    parent: &mut ChildBuilder<'a>,
    team: Team,
    slot_type: CardSlotType,
    color: &Color,
    slot_image: Handle<Image>,
    font: Handle<Font>,
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
                let slot = CardSlot {
                    id: id,
                    team: team,
                    slot_type: slot_type,
                };
                parent
                    .spawn(ButtonBundle {
                        style: Style {
                            height: Val::Percent(100.0),
                            aspect_ratio: Some(72.0 / 102.0),
                            ..default()
                        },
                        image: UiImage {
                            texture: slot_image.clone(),
                            ..default()
                        },
                        ..default()
                    })       .insert(ButtonCardSlot(slot.clone()))    
                    .with_children(|parent| {
                        parent
                            .spawn(ImageBundle {
                                style: Style {
                                    height: Val::Percent(100.0),
                                    aspect_ratio: Some(72.0 / 102.0),
                                    ..default()
                                },
                                visibility: Visibility::Hidden,
                                ..default()
                            })
                            .insert(slot)
                            .with_children(|parent| {
                                parent.spawn(TextBundle {
                                    style: Style { right: Val::Percent(0.8 * 100.0), left: Val::Percent(0.1 * 100.0), top: Val::Percent(0.6274509803921569 * 100.0), bottom: Val::Percent(0.1 * 100.0), width: Val::Percent(80.0), height: Val::Percent(37.254901960784316), ..default() },
                                    text: Text {
                                        sections: vec![TextSection {
                                            style: TextStyle {
                                                font: font.clone(),
                                                color: Color::Rgba {
                                                    red: 0.0,
                                                    green: 0.0,
                                                    blue: 0.0,
                                                    alpha: 1.0,
                                                },
                                                font_size: 7.0,
                                                ..default()
                                            },
                                            value: "this is a test text for all cards so I can position this right".to_string(),
                                        }],
                                        linebreak_behavior: bevy::text::BreakLineOn::AnyCharacter,
                                        ..default()
                                    },
                                    
                                    ..default()
                                });
                            });
                    });
            }
        });
}

fn spawn_card_piles<'a>(
    parent: &mut ChildBuilder<'a>,
    card_backs: &Res<Assets<CardBack>>,
    card_type_state: &Res<State<NextTurnCardType>>,
    font: Handle<Font>,
    assets: &Res<AssetServer>,
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
            image: UiImage {
                texture: assets.load("ui/Heart1.png"),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "100",
                TextStyle {
                    font: font.clone(),
                    ..default()
                },
            ));
        });

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
