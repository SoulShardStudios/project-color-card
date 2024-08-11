use crate::assets::LoadState;
use crate::cards::{Card, CardColor, CardType};
use crate::constants::CARD_SLOT_COUNT;
use crate::game_state::{
    BlueHealthMarker, CardSlot, CardSlotType, CardStats, RedHealthMarker, Team,
};
use crate::spawn_ui::spawn_game_ui;
use bevy::prelude::*;
use bevy_rand::prelude::WyRand;
use bevy_rand::resource::GlobalEntropy;
use rand::Rng;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
enum ModifyCardAction {
    Remove {
        slot: CardSlot,
    },
    Push {
        slot: CardSlot,
        card: AssetId<Card>,
        stats: CardStats,
    },
}

#[derive(Component)]
pub struct GameController {
    team_health: BTreeMap<Team, u32>,
    current_cards: BTreeMap<CardSlot, Option<(AssetId<Card>, CardStats)>>,
    valid_new_cards: Vec<AssetId<Card>>,
    card_ids: Vec<AssetId<Card>>,
    card_modifications: Vec<ModifyCardAction>,
    team_health_updated: bool,
}

impl GameController {
    pub fn new(cards: &Res<Assets<Card>>, rng: &mut ResMut<GlobalEntropy<WyRand>>) -> Self {
        let mut card_names: BTreeMap<CardSlot, Option<(AssetId<Card>, CardStats)>> =
            BTreeMap::new();
        for team in [Team::Blue, Team::Red] {
            for slot_type in [CardSlotType::Hand, CardSlotType::Play] {
                for id in 0..CARD_SLOT_COUNT {
                    card_names.insert(
                        CardSlot {
                            id,
                            slot_type,
                            team,
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
        let mut gc = GameController {
            team_health: BTreeMap::from_iter([(Team::Red, 100), (Team::Blue, 100)]),
            current_cards: card_names,
            valid_new_cards,
            card_ids: cards.iter().map(|(id, _card)| id).collect(),
            card_modifications: vec![],
            team_health_updated: false,
        };

        for _ in 0..4 {
            for team in vec![Team::Blue, Team::Red].iter() {
                let card = gc.get_random_card(rng);
                gc.push_card_into_stack(
                    CardSlot {
                        id: 0,
                        team: team.clone(),
                        slot_type: CardSlotType::Hand,
                    },
                    card,
                    CardStats {
                        hp: cards.get(card).unwrap().hp,
                    },
                );
            }
        }
        gc
    }

    pub fn get_team_health(&self, team: Team) -> u32 {
        return self.team_health[&team];
    }

    pub fn set_team_health(&mut self, team: Team, health: u32) {
        self.team_health_updated = true;
        self.team_health.insert(team, health);
    }

    pub fn card_stack_full(&self, team: Team, slot_type: CardSlotType) -> bool {
        for slot in (0..CARD_SLOT_COUNT).into_iter().map(|id| CardSlot {
            team,
            slot_type,
            id,
        }) {
            if self.get_card(&slot).is_none() {
                return false;
            }
        }
        return true;
    }

    pub fn get_card(&self, slot: &CardSlot) -> Option<(AssetId<Card>, CardStats)> {
        self.current_cards.get(slot).map(|x| x.clone())?
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

    pub fn push_card_at(&mut self, slot: CardSlot, card: AssetId<Card>, stats: CardStats) {
        self.card_modifications.push(ModifyCardAction::Push {
            slot: slot.clone(),
            card,
            stats: stats.clone(),
        });
        self.current_cards.insert(slot, Some((card, stats)));
    }

    pub fn remove_card(&mut self, slot: CardSlot) {
        self.card_modifications
            .push(ModifyCardAction::Remove { slot: slot.clone() });
        self.current_cards.insert(slot, None);
    }

    pub fn damage_card(&mut self, slot: &CardSlot, damage: u32) {
        let mut slots_to_take = None;
        let mut card_to_push = None;
        match self.get_card(slot) {
            None => {}
            Some(card) => match card.1.hp {
                Some(hp) => {
                    if hp.saturating_sub(damage) == 0 {
                        slots_to_take = Some(slot.clone());
                    } else {
                        card_to_push = Some(card);
                    }
                }
                None => {}
            },
        }
        match card_to_push {
            Some(card) => {
                let stats = CardStats {
                    hp: card.1.hp.map(|f| f.saturating_sub(damage)),
                };
                self.push_card_at(slot.clone(), card.0, stats);
            }
            None => {}
        }

        match slots_to_take {
            Some(x) => {
                self.remove_card(x.clone());
            }
            None => {}
        }
    }

    pub fn get_random_card(&self, rng: &mut ResMut<GlobalEntropy<WyRand>>) -> AssetId<Card> {
        if self.valid_new_cards.len() == 0 {
            panic!("Card assets failed to load, quitting")
        }
        return self.valid_new_cards[rng.gen_range(0usize..self.valid_new_cards.len() - 1)];
    }

    pub fn get_random_card_of_type_with_len(
        &self,
        rng: &mut ResMut<GlobalEntropy<WyRand>>,
        cards: &Res<Assets<Card>>,
        card_type: CardType,
        color_len: usize,
    ) -> AssetId<Card> {
        loop {
            let card_id = self.get_random_card(rng);
            let card = cards.get(card_id).unwrap();
            if card.card_type == card_type && card.colors.len() == color_len {
                return card_id;
            }
        }
    }

    pub fn get_card_with_colors<'a>(
        &self,
        colors: Vec<CardColor>,
        cards: &'a Assets<Card>,
        card_type: CardType,
    ) -> Option<(&'a Card, AssetId<Card>)> {
        for card_id in self.card_ids.iter() {
            let card = cards.get(*card_id).unwrap();
            if card.colors == colors && card.card_type == card_type {
                return Some((card, *card_id));
            }
        }
        return None;
    }

    fn _clone_iter_current(
        &mut self,
        team: Team,
        slot_type: CardSlotType,
    ) -> Vec<(CardSlot, Option<(AssetId<Card>, CardStats)>)> {
        self.current_cards
            .iter()
            .filter(|(slot, _)| slot.team == team && slot.slot_type == slot_type)
            .map(|(x, y)| (x.clone(), y.clone()))
            .collect()
    }

    pub fn push_card_into_stack(&mut self, slot: CardSlot, card: AssetId<Card>, stats: CardStats) {
        let first_slot = self.get_first_open_slot(slot.team, slot.slot_type);
        if first_slot.is_none() {
            return;
        }
        let mut last_card: Option<(AssetId<Card>, CardStats)> = None;
        for (iter_slot, iter_card) in self._clone_iter_current(slot.team, slot.slot_type) {
            let slot = slot.clone();
            if iter_slot.id < slot.id {
                continue;
            }
            if slot.id == iter_slot.id {
                self.push_card_at(slot, card, stats.clone());
                last_card = iter_card.clone();
                continue;
            }
            let last_card_unwrapped = match last_card {
                Some(x) => x,
                None => {
                    break;
                }
            };
            self.push_card_at(iter_slot, last_card_unwrapped.0, last_card_unwrapped.1);
            last_card = iter_card;
        }
    }

    pub fn stack_cards(&mut self, team: Team, slot_type: CardSlotType) {
        for (slot, card) in self._clone_iter_current(team, slot_type) {
            let first_open = match self.get_first_open_slot(team, slot_type) {
                Some(x) => x,
                None => {
                    return;
                }
            };
            let card = match card {
                Some(x) => x,
                None => {
                    continue;
                }
            };
            if slot.id > first_open {
                self.push_card_at(
                    CardSlot {
                        id: first_open,
                        team: slot.team,
                        slot_type: slot.slot_type,
                    },
                    card.0,
                    card.1.clone(),
                );
                self.remove_card(slot);
            }
        }
    }
}

fn push_card(
    query: &mut Query<(&CardSlot, &mut UiImage, &mut Visibility, Entity)>,
    child_query: &Query<&mut Children>,
    text_query: &mut Query<&mut Text>,
    card: &Card,
    stats: &CardStats,
    slot: CardSlot,
) {
    match query.iter_mut().filter(|(x, _, _, _)| **x == slot).nth(0) {
        Some((_, mut image, mut visibility, entity)) => {
            image.texture = card.image_handle.clone();
            *visibility = Visibility::Visible;
            for (idx, decendant) in child_query.iter_descendants(entity).enumerate() {
                for grand_decendant in child_query.iter_descendants(decendant) {
                    if idx == 0 {
                        text_query.get_mut(grand_decendant).unwrap().sections[0].value =
                            card.text.clone();
                    }
                    if idx == 1 {
                        text_query.get_mut(grand_decendant).unwrap().sections[0].value =
                            stats.hp.map(|hp| hp.to_string()).unwrap_or("".to_string())
                    }
                }
            }
        }
        None => {}
    }
}

fn remove_card(
    query: &mut Query<(&CardSlot, &mut UiImage, &mut Visibility, Entity)>,
    slot: CardSlot,
) {
    match query.iter_mut().filter(|(x, _, _, _)| **x == slot).nth(0) {
        Some(mut x) => {
            x.1.texture = Handle::default();
            *x.2 = Visibility::Hidden;
        }
        _ => {}
    }
}

fn apply_card_modifications(
    mut game_ui_controller_query: Query<&mut GameController>,
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

    for modification in game_ui_controller.card_modifications.clone() {
        match modification {
            ModifyCardAction::Push { slot, card, stats } => {
                push_card(
                    &mut query,
                    &child_query,
                    &mut text_query,
                    &cards.get(card).unwrap(),
                    &stats,
                    slot.clone(),
                );
            }
            ModifyCardAction::Remove { slot } => {
                remove_card(&mut query, slot.clone());
            }
        }
    }
    game_ui_controller.card_modifications.clear();
    game_ui_controller.stack_cards(Team::Red, CardSlotType::Play);
    game_ui_controller.stack_cards(Team::Blue, CardSlotType::Play);
}

fn spawn_game_ui_controller(
    mut commands: Commands,
    cards: Res<Assets<Card>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    commands.spawn(GameController::new(&cards, &mut rng));
}

fn update_team_health(
    mut game_ui_controller_query: Query<&mut GameController>,
    red_health_query: Query<Entity, With<RedHealthMarker>>,
    blue_health_query: Query<Entity, With<BlueHealthMarker>>,
    child_query: Query<&mut Children>,
    mut text_query: Query<&mut Text>,
) {
    let game_ui_controller = match game_ui_controller_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    if !game_ui_controller.team_health_updated {
        return;
    }
    for red_health in red_health_query.iter() {
        for decendant in child_query.iter_descendants(red_health) {
            text_query
                .get_mut(decendant)
                .expect("expected text")
                .sections[0]
                .value = game_ui_controller.team_health[&Team::Red].to_string();
        }
    }
    for blue_health in blue_health_query.iter() {
        for decendant in child_query.iter_descendants(blue_health) {
            text_query
                .get_mut(decendant)
                .expect("expected text")
                .sections[0]
                .value = game_ui_controller.team_health[&Team::Blue].to_string();
        }
    }
}

pub struct GameUiControllerPlugin;

impl Plugin for GameUiControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LoadState::Loaded),
            (spawn_game_ui_controller, spawn_game_ui),
        )
        .add_systems(Update, (apply_card_modifications, update_team_health));
    }
}
