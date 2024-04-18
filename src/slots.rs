use crate::card::{Card, CardAssetPlugin};
use bevy::prelude::*;
use bevy_rand::prelude::{GlobalEntropy, WyRand};
use rand::{Rng, RngCore};
use std::rc::Rc;

// display: the deck -> wip, what's in the user's hand -> wip and what's on the board -> done
// user is given cards in their deck randomly at the start of the game, deck is rebalanced each time a card is removed automatically,
// user places one card and more if anything allows it cards are then
// all card names should be unique

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

pub enum GameState {
    DrawCards,
    PlayCards,
    ApplyMoves,
}

#[derive(Resource)]
pub struct GameStateRes(GameState);

fn apply_game_state(
    card_manager: Res<CardManager>,
    rng: ResMut<'static, GlobalEntropy<WyRand>>,
    game_state: ResMut<GameStateRes>,
) {
    card_manager.get_random_card(rng);
    match game_state.0 {
        GameState::DrawCards => {}
        GameState::PlayCards => {}
        GameState::ApplyMoves => {}
    }
}
