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
    push_card_actions: Vec<(Team, CardSlotType, AssetId<Card>, Option<usize>)>,
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

    pub fn damage_card(&mut self, slot: &CardSlot, damage: u32) {
        self.damage_card_actions.push((slot.clone(), damage));
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
    mut query: Query<(&CardSlot, &mut UiImage, &mut Visibility, &mut Children)>,
    mut child_query: Query<&mut Text>,
) {
    let mut game_ui_controller = match game_ui_controller_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    for (team, slot_type, card, id) in game_ui_controller.push_card_actions.clone() {
        let card_asset = cards.get(card).unwrap();
        let slots_and_ui: Vec<_> = query
            .iter_mut()
            .filter(|(x, _, _, _)| x.team == team && x.slot_type == slot_type)
            .collect();

        match id {
            None => {
                for (slot, mut ui, mut visibility, children) in slots_and_ui {
                    if game_ui_controller.get_card_id(slot).is_some() {
                        // should we shift the stack?

                        continue;
                    }
                    ui.texture = card_asset.image_handle.clone();
                    game_ui_controller.current_cards.remove(&slot);
                    game_ui_controller
                        .current_cards
                        .insert(slot.clone(), Some(card));
                    *visibility = Visibility::Visible;
                    child_query
                        .get_mut(*children.iter().nth(0).unwrap())
                        .unwrap()
                        .sections[0]
                        .value = card_asset.text.clone();
                    break;
                }
            }
            Some(mut slot_id) => {
                let first_empty_slot = slots_and_ui
                    .iter()
                    .map(|x| x.0)
                    .find(|x| game_ui_controller.get_card_id(x).is_none());
                let first_empty_slot_id = match first_empty_slot {
                    Some(x) => x.id,
                    None => {
                        return;
                    }
                };
                if slot_id > first_empty_slot_id {
                    slot_id = first_empty_slot_id;
                }
                let mut last_card: Option<AssetId<Card>> = None;

                for (slot, mut ui, mut visibility, children) in
                    slots_and_ui.into_iter().skip(slot_id)
                {
                    if slot.id == slot_id {
                        last_card = *game_ui_controller.current_cards.get(&slot).unwrap();
                        game_ui_controller.current_cards.remove(&slot);
                        ui.texture = card_asset.image_handle.clone();
                        game_ui_controller
                            .current_cards
                            .insert(slot.clone(), Some(card));
                        *visibility = Visibility::Visible;
                        child_query
                            .get_mut(*children.iter().nth(0).unwrap())
                            .unwrap()
                            .sections[0]
                            .value = card_asset.text.clone();
                        continue;
                    }
                    let last_card_unwrapped = match last_card {
                        Some(x) => x,
                        _ => {
                            break;
                        }
                    };
                    last_card = *game_ui_controller.current_cards.get(&slot).unwrap();
                    game_ui_controller.current_cards.remove(&slot);
                    let last_card = cards.get(last_card_unwrapped).unwrap();
                    ui.texture = last_card.image_handle.clone();
                    game_ui_controller
                        .current_cards
                        .insert(slot.clone(), Some(last_card_unwrapped));
                    *visibility = Visibility::Visible;
                    child_query
                        .get_mut(*children.iter().nth(0).unwrap())
                        .unwrap()
                        .sections[0]
                        .value = last_card.text.clone();
                }
            }
        }
    }
    game_ui_controller.push_card_actions.clear();
}

fn damage_cards(
    mut query: Query<(&CardSlot, &mut CardStats)>,
    game_ui_controller_query: Query<&GameUIController>,
) {
    let game_ui_controller = match game_ui_controller_query.iter().nth(0) {
        None => {
            return;
        }
        Some(x) => x,
    };
    for (slot, damage) in game_ui_controller.damage_card_actions.iter() {
        match query.iter_mut().filter(|(x, _)| **x == *slot).nth(0) {
            Some(mut x) => x.1.hp -= damage,
            _ => {}
        }
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
        .add_systems(Update, (set_cards, take_cards, damage_cards));
    }
}
