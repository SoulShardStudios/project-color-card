use crate::cards::{
    get_card_back_image, Card, CardAssetPlugin, CardBack, CardBackAssetPlugin, CardBackType,
    CardType,
};
use crate::custom_cursor::{CustomCursor, CustomCursorPlugin};
use crate::game_state::{
    CardDeckMarker, CardSlot, CardSlotMarker, CardSlotType, CardStats, CurrentTurnTeam,
    NextTurnCardType, Team, TurnState,
};
use crate::game_ui_controller::{GameController, GameUiControllerPlugin};
use bevy::prelude::*;
use bevy_rand::prelude::WyRand;
use bevy_rand::resource::GlobalEntropy;
use num_traits::FromPrimitive;
use rand::Rng;

pub fn draw_card(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<CardDeckMarker>),
    >,
    mut game_ui_controller_query: Query<&mut GameController>,
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
                let card_color_count = rng.gen_range(1..3);
                let random_card_of_type = game_ui_controller.get_random_card_of_type_with_len(
                    &mut rng,
                    &cards,
                    current_card_type_state.get().0.clone(),
                    card_color_count,
                );
                let random_card_asset = cards.get(random_card_of_type).unwrap();
                match game_ui_controller
                    .get_first_open_slot(current_turn_team.0, CardSlotType::Hand)
                {
                    Some(x) => {
                        game_ui_controller.push_card_into_stack(
                            CardSlot {
                                id: x,
                                team: current_turn_team.0,
                                slot_type: CardSlotType::Hand,
                            },
                            random_card_of_type,
                            CardStats {
                                hp: random_card_asset.hp,
                            },
                        );
                        game_ui_controller.stack_cards(current_turn_team.0, CardSlotType::Hand);
                    }
                    None => {}
                }

                let new_card_type = CardType::from_i8(rng.gen_range(0..4)).unwrap();
                card_type_state.set(NextTurnCardType(new_card_type));
                match draw_image_query.get_single_mut() {
                    Ok(mut x) => {
                        x.texture =
                            get_card_back_image(&card_backs, CardBackType::CardType(new_card_type))
                    }
                    _ => {}
                }
                turn_state.set(TurnState::PlayCards);
            }
            _ => {}
        }
    }
}

fn play_card(
    mut game_ui_controller_query: Query<&mut GameController>,
    mut custom_cursor_query: Query<&mut CustomCursor>,
    mut interaction_query: Query<
        (&Interaction, Entity),
        (Changed<Interaction>, With<Button>, With<CardSlotMarker>),
    >,
    card_slot_query: Query<&CardSlot>,
    children_query: Query<&Children>,
    current_turn_state: Res<State<TurnState>>,
    mut turn_state: ResMut<NextState<TurnState>>,
    current_turn_team: Res<State<CurrentTurnTeam>>,
    cards: Res<Assets<Card>>,
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
    let mut custom_cursor = match custom_cursor_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    match custom_cursor.clone() {
        // pick up card and set custom cursor
        CustomCursor::Default => {
            for (interaction, entity) in &mut interaction_query {
                let slot = card_slot_query
                    .get(children_query.iter_descendants(entity).nth(0).unwrap())
                    .unwrap();
                if !(slot.team == current_turn_team.get().0 && slot.slot_type == CardSlotType::Hand)
                {
                    continue;
                }
                match *interaction {
                    Interaction::Pressed => match game_ui_controller.get_card(&slot) {
                        Some(card) => {
                            *custom_cursor = CustomCursor::Card {
                                card: card.0,
                                stats: game_ui_controller.get_card(slot).unwrap().1,
                            };
                            game_ui_controller.remove_card(slot.clone());
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        // place custom cursor down in play and adjust slots
        CustomCursor::Card { card, stats } => {
            for (interaction, entity) in &mut interaction_query {
                let slot = card_slot_query
                    .get(children_query.iter_descendants(entity).nth(0).unwrap())
                    .unwrap();
                if !slot.team == current_turn_team.get().0 && slot.slot_type == CardSlotType::Play {
                    continue;
                }

                match *interaction {
                    Interaction::Pressed => {
                        match game_ui_controller.get_card(slot) {
                            Some(x) => {
                                let slot_card = cards.get(x.0).unwrap();
                                let held_card = cards.get(card).unwrap();
                                if cards_can_combine(slot_card, held_card)
                                    || cards_can_combine(held_card, slot_card)
                                {
                                    let mut colors = slot_card.colors.clone();
                                    colors.extend(held_card.colors.clone());
                                    match game_ui_controller.get_card_with_colors(
                                        colors,
                                        &*cards,
                                        get_upgraded_card_type(slot_card, held_card),
                                    ) {
                                        Some(x) => {
                                            game_ui_controller.push_card_at(
                                                slot.clone(),
                                                x.1,
                                                CardStats { hp: x.0.hp },
                                            );
                                        }
                                        None => {
                                            game_ui_controller.push_card_into_stack(
                                                slot.clone(),
                                                card,
                                                stats.clone(),
                                            );
                                        }
                                    }
                                } else {
                                    game_ui_controller.push_card_into_stack(
                                        slot.clone(),
                                        card,
                                        stats.clone(),
                                    );
                                }
                            }
                            None => {
                                game_ui_controller.push_card_into_stack(
                                    slot.clone(),
                                    card,
                                    stats.clone(),
                                );
                            }
                        };
                        game_ui_controller.stack_cards(slot.team, slot.slot_type);
                        *custom_cursor = CustomCursor::Default;
                        turn_state.set(TurnState::ApplyMoves);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn cards_can_combine(first_card: &Card, second_card: &Card) -> bool {
    return vec![CardType::Hero, CardType::Beast].contains(&first_card.card_type)
        && second_card
            .colors
            .iter()
            .all(|x| !first_card.colors.contains(x))
        && second_card.card_type == CardType::Equipment;
}

fn get_upgraded_card_type(first_card: &Card, second_card: &Card) -> CardType {
    if vec![CardType::Hero, CardType::Beast].contains(&first_card.card_type) {
        return first_card.card_type;
    }
    if vec![CardType::Hero, CardType::Beast].contains(&second_card.card_type) {
        return second_card.card_type;
    }
    panic!("unreachable")
}

fn apply_moves(
    current_turn_state: Res<State<TurnState>>,
    card_slot_query: Query<&CardSlot>,
    current_turn_team: Res<State<CurrentTurnTeam>>,
    mut game_ui_controller_query: Query<&mut GameController>,
    cards: Res<Assets<Card>>,
    mut team_state: ResMut<NextState<CurrentTurnTeam>>,
    mut turn_state: ResMut<NextState<TurnState>>,
) {
    let mut game_ui_controller = match game_ui_controller_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    let team_health = game_ui_controller
        .get_team_health(current_turn_team.get().0)
        .clone();
    if *current_turn_state.get() != TurnState::ApplyMoves {
        return;
    }
    for (current_slot, foe_slot) in card_slot_query
        .iter()
        .filter(|slot| {
            slot.team == current_turn_team.get().0 && slot.slot_type == CardSlotType::Play
        })
        .zip(card_slot_query.iter().filter(|slot| {
            slot.team == !current_turn_team.get().0 && slot.slot_type == CardSlotType::Play
        }))
    {
        let foe_card = game_ui_controller
            .get_card(foe_slot)
            .map(|x| cards.get(x.0).unwrap());
        let current_card = game_ui_controller
            .get_card(current_slot)
            .map(|x| cards.get(x.0).unwrap());
        match (current_card, foe_card) {
            (Some(current), Some(foe)) => {
                game_ui_controller.damage_card(foe_slot, current.damage.unwrap_or(0));
                game_ui_controller.damage_card(current_slot, foe.damage.unwrap_or(0));
            }
            (None, Some(foe)) => game_ui_controller.set_team_health(
                current_turn_team.get().0,
                team_health - foe.damage.unwrap_or(0),
            ),
            (Some(current), None) => game_ui_controller.set_team_health(
                current_turn_team.get().0,
                team_health - current.damage.unwrap_or(0),
            ),
            _ => {}
        }
    }

    turn_state.set(TurnState::DrawCards);
    team_state.set(CurrentTurnTeam(!current_turn_team.get().0));
}

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CardSlot>()
            .register_type::<CardSlotType>()
            .register_type::<Team>()
            .register_type::<CardSlot>()
            .init_state::<TurnState>()
            .init_state::<NextTurnCardType>()
            .init_state::<CurrentTurnTeam>()
            .add_plugins(CustomCursorPlugin)
            .add_plugins(CardAssetPlugin)
            .add_plugins(CardBackAssetPlugin)
            .add_plugins(GameUiControllerPlugin)
            .add_systems(Update, (draw_card, play_card, apply_moves));
    }
}
