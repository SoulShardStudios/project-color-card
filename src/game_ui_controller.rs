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

#[derive(Clone, Debug)]
enum ModifyCardAction {
    Remove {
        slot: CardSlot,
    },
    Damage {
        slot: CardSlot,
        damage: u32,
    },
    Push {
        slot: CardSlot,
        card: AssetId<Card>,
        stats: CardStats,
    },
}

#[derive(Component)]
pub struct GameController {
    pub team_health: BTreeMap<Team, u32>,
    current_cards: BTreeMap<CardSlot, Option<(AssetId<Card>, CardStats)>>,
    valid_new_cards: Vec<AssetId<Card>>,
    card_modifications: Vec<ModifyCardAction>,
}

impl GameController {
    pub fn new(cards: &Res<Assets<Card>>) -> Self {
        let mut card_names: BTreeMap<CardSlot, Option<(AssetId<Card>, CardStats)>> =
            BTreeMap::new();
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
        GameController {
            team_health: BTreeMap::from_iter([(Team::Red, 100), (Team::Blue, 100)]),
            current_cards: card_names,
            valid_new_cards,
            card_modifications: vec![],
        }
    }

    pub fn card_stack_full(&self, team: Team, slot_type: CardSlotType) -> bool {
        for slot in (0..CARD_SLOT_COUNT).into_iter().map(|id| CardSlot {
            team: team,
            slot_type: slot_type,
            id: id,
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

    pub fn push_card_at<'a>(&mut self, slot: CardSlot, card: AssetId<Card>, stats: CardStats) {
        self.card_modifications.push(ModifyCardAction::Push {
            slot: slot.clone(),
            card: card,
            stats: stats.clone(),
        });
        self.current_cards.remove(&slot);
        self.current_cards.insert(slot, Some((card, stats)));
    }

    pub fn take_card(&mut self, slot: &CardSlot) {
        self.card_modifications
            .push(ModifyCardAction::Remove { slot: slot.clone() });
        self.current_cards.remove(&slot);
        self.current_cards.insert(slot.clone(), None);
    }

    pub fn damage_card(&mut self, slot: &CardSlot, damage: u32) {
        self.card_modifications.push(ModifyCardAction::Damage {
            slot: slot.clone(),
            damage: damage,
        });
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

    pub fn stack_cards(&mut self, team: Team, slot_type: CardSlotType) {
        let bruh: Vec<_> = self
            .current_cards
            .iter()
            .filter(|(slot, _)| slot.team == team && slot.slot_type == slot_type)
            .map(|(x, y)| (x.clone(), y.clone()))
            .collect();
        for (slot, card) in bruh {
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
                let first_open_slot = CardSlot {
                    id: first_open,
                    team: slot.team,
                    slot_type: slot.slot_type,
                };
                self.card_modifications.push(ModifyCardAction::Push {
                    slot: first_open_slot.clone(),
                    card: card.0,
                    stats: card.1.clone(),
                });
                self.card_modifications
                    .push(ModifyCardAction::Remove { slot: slot.clone() });
                self.current_cards.remove(&slot);
                self.current_cards
                    .insert(first_open_slot, Some((card.0, card.1.clone())));
            }
        }
    }
}

fn apply_card_modifications(
    mut game_ui_controller_query: Query<&mut GameController>,
    cards: Res<Assets<Card>>,
    mut query: Query<(
        &CardSlot,
        &mut CardStats,
        &mut UiImage,
        &mut Visibility,
        Entity,
    )>,
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
                match query
                    .iter_mut()
                    .filter(|(x, _, _, _, _)| **x == slot)
                    .nth(0)
                {
                    Some((_, _, mut image, mut visibility, entity)) => {
                        let card_asset = cards.get(card).unwrap();
                        image.texture = card_asset.image_handle.clone();
                        *visibility = Visibility::Visible;
                        for (idx, decendant) in child_query.iter_descendants(entity).enumerate() {
                            for grand_decendant in child_query.iter_descendants(decendant) {
                                if idx == 0 {
                                    text_query.get_mut(grand_decendant).unwrap().sections[0]
                                        .value = card_asset.text.clone();
                                }
                                if idx == 1 {
                                    text_query.get_mut(grand_decendant).unwrap().sections[0].value =
                                        stats.hp.map(|hp| hp.to_string()).unwrap_or("".to_string())
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            ModifyCardAction::Remove { slot } => {
                match query
                    .iter_mut()
                    .filter(|(x, _, _, _, _)| **x == slot)
                    .nth(0)
                {
                    Some(mut x) => {
                        x.2.texture = Handle::default();
                        *x.3 = Visibility::Hidden;
                    }
                    _ => {}
                }
            }
            ModifyCardAction::Damage { slot, damage } => {
                let mut slots_to_take: Option<CardSlot> = None;
                match query
                    .iter_mut()
                    .filter(|(x, _, _, _, _)| **x == slot)
                    .nth(0)
                {
                    Some(mut x) => match &mut x.1.hp {
                        Some(mut hp) => {
                            hp = hp.saturating_sub(damage);
                            if hp == 0 {
                                slots_to_take = Some(slot.clone());
                            }
                        }
                        None => {}
                    },
                    _ => {}
                }
                match slots_to_take {
                    Some(x) => game_ui_controller.take_card(&x),
                    None => {}
                }
            }
        }
    }
    game_ui_controller.card_modifications.clear();
}

fn spawn_game_ui_controller(mut commands: Commands, cards: Res<Assets<Card>>) {
    commands.spawn(GameController::new(&cards));
}

pub struct GameUiControllerPlugin;

impl Plugin for GameUiControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LoadState::Loaded),
            (spawn_game_ui_controller, spawn_game_ui),
        )
        .add_systems(Update, apply_card_modifications);
    }
}
