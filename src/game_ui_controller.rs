use crate::assets::LoadState;
use crate::cards::{Card, CardType};
use crate::constants::CARD_SLOT_COUNT;
use crate::game_state::{CardSlot, CardSlotType, CardStats, Team};
use crate::spawn_ui::spawn_game_ui;
use bevy::prelude::*;
use bevy_rand::prelude::WyRand;
use bevy_rand::resource::GlobalEntropy;
use rand::Rng;
use std::collections::BTreeMap;

#[derive(Component)]
pub struct GameUIController {
    pub team_health: BTreeMap<Team, u32>,
    current_cards: BTreeMap<CardSlot, Option<AssetId<Card>>>,
    valid_new_cards: Vec<AssetId<Card>>,
    push_card_actions: Vec<(CardSlot, AssetId<Card>, CardStats)>,
    take_card_actions: Vec<CardSlot>,
    damage_card_actions: Vec<(CardSlot, u32)>,
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
            team_health: BTreeMap::from_iter([(Team::Red, 100), (Team::Blue, 100)]),
            current_cards: card_names,
            valid_new_cards,
            push_card_actions: vec![],
            take_card_actions: vec![],
            damage_card_actions: vec![],
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

    pub fn get_first_open_slot(&self, team: Team, slot_type: CardSlotType) -> Option<usize> {
        self.current_cards
            .iter()
            .filter(|(slot, card)| {
                slot.team == team && slot.slot_type == slot_type && card.is_none()
            })
            .nth(0)
            .map(|(slot, _)| slot.id)
    }

    pub fn push_card_at<'a>(&mut self, slot: CardSlot, card: AssetId<Card>, stats: CardStats) {
        self.push_card_actions.push((slot, card, stats));
    }

    pub fn take_card(&mut self, slot: &CardSlot) {
        self.take_card_actions.push(slot.clone());
    }

    pub fn damage_card(&mut self, slot: &CardSlot, damage: u32) {
        self.damage_card_actions.push((slot.clone(), damage));
    }

    pub fn get_random_card(&self, rng: &mut ResMut<GlobalEntropy<WyRand>>) -> AssetId<Card> {
        if self.valid_new_cards.len() == 0 {
            panic!("Card assets failed to load, quitting")
        }
        return self.valid_new_cards[rng.gen_range(0usize..self.valid_new_cards.len() - 1)];
    }

    pub fn get_random_card_of_type(
        &self,
        rng: &mut ResMut<GlobalEntropy<WyRand>>,
        cards: &Res<Assets<Card>>,
        card_type: CardType,
    ) -> AssetId<Card> {
        loop {
            let card_id = self.get_random_card(rng);
            let card = cards.get(card_id).unwrap();
            if card.card_type == card_type {
                return card_id;
            }
        }
    }
}

fn set_cards_main(
    mut game_ui_controller_query: Query<&mut GameUIController>,
    cards: Res<Assets<Card>>,
    mut query: Query<(&CardSlot, &mut UiImage, &mut Visibility, Entity)>,
    child_query: Query<&mut Children>,
    mut text_query: Query<&mut Text>,
) {
    let mut game_ui_controller = match game_ui_controller_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    // relationship is text -> slot not the other way around but we need a direct query on it to modify it properly, I presume....

    for (slot, card, stats) in game_ui_controller.push_card_actions.clone() {
        for (query_slot, mut image, mut visibility, entity) in query.iter_mut() {
            if *query_slot != slot {
                continue;
            }

            let card_asset = cards.get(card).unwrap();
            game_ui_controller.current_cards.remove(&slot);
            image.texture = card_asset.image_handle.clone();
            game_ui_controller
                .current_cards
                .insert(slot.clone(), Some(card));
            *visibility = Visibility::Visible;

            for (idx, decendant) in child_query.iter_descendants(entity).enumerate() {
                for grand_decendant in child_query.iter_descendants(decendant) {
                    if idx == 0 {
                        text_query.get_mut(grand_decendant).unwrap().sections[0].value =
                            card_asset.text.clone();
                    }
                    if idx == 1 {
                        text_query.get_mut(grand_decendant).unwrap().sections[0].value =
                            stats.hp.map(|hp| hp.to_string()).unwrap_or("".to_string())
                    }
                }
            }
        }
    }
    game_ui_controller.push_card_actions.clear();
}

fn damage_cards(
    mut query: Query<(&CardSlot, &mut CardStats)>,
    mut game_ui_controller_query: Query<&mut GameUIController>,
) {
    let mut game_ui_controller = match game_ui_controller_query.iter_mut().nth(0) {
        None => {
            return;
        }
        Some(x) => x,
    };
    let mut slots_to_take: Vec<CardSlot> = vec![];
    for (slot, damage) in game_ui_controller.damage_card_actions.iter() {
        match query.iter_mut().filter(|(x, _)| **x == *slot).nth(0) {
            Some(mut x) => match &mut x.1.hp {
                Some(mut hp) => {
                    hp = hp.saturating_sub(*damage);
                    if hp == 0 {
                        slots_to_take.push(slot.clone());
                    }
                }
                None => {}
            },
            _ => {}
        }
    }
    for slot in slots_to_take {
        game_ui_controller.take_card(&slot);
    }
}

fn take_cards(
    mut game_ui_controller_query: Query<&mut GameUIController>,
    mut query: Query<(&CardSlot, &mut UiImage, &mut Visibility)>,
) {
    let mut game_ui_controller = match game_ui_controller_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    // apply take actions
    for slot in game_ui_controller.take_card_actions.clone() {
        match query.iter_mut().filter(|(x, _, _)| **x == slot).nth(0) {
            Some(mut x) => x.1.texture = Handle::default(),
            _ => {}
        }
        game_ui_controller.current_cards.remove(&slot);
        game_ui_controller.current_cards.insert(slot.clone(), None);
        // reset the stack
        let textures: Vec<_> = query
            .iter()
            .map(|(_, image, _)| image.texture.clone())
            .collect();
        for (slot, mut ui, mut visibility) in query
            .iter_mut()
            .filter(|(x, _, _)| x.team == slot.team && x.slot_type == slot.slot_type)
        {
            if slot.id == CARD_SLOT_COUNT - 1 {
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
            *visibility = match next_slot_card {
                Some(_) => Visibility::Visible,
                None => Visibility::Hidden,
            }
        }
    }

    game_ui_controller.take_card_actions.clear();
}

fn spawn_game_ui_controller(mut commands: Commands, cards: Res<Assets<Card>>) {
    commands.spawn(GameUIController::new(&cards));
}

pub struct GameUiControllerPlugin;

impl Plugin for GameUiControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LoadState::Loaded),
            (spawn_game_ui_controller, spawn_game_ui),
        )
        .add_systems(Update, (set_cards_main, take_cards, damage_cards));
    }
}
