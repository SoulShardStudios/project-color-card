extern crate bevy;
use bevy::prelude::*;

enum CardColor {
    Red,
    Yellow,
    Blue,
    Green,
    Purple,
    Teal,
}

enum CardType {
    Hero,
    Beast,
    Equipment,
    Food,
    Spell,
}

struct Card {
    colors: Vec<CardColor>,
    card_type: CardType,
    image: Handle<Image>,
    text: String,
}

fn main() {}
