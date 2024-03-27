use crate::create_ron_nested_asset_loader;
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
enum CardColor {
    Red,
    Yellow,
    Blue,
    Green,
    Purple,
    Teal,
}
#[derive(Debug, Serialize, Deserialize)]
enum CardType {
    Hero,
    Beast,
    Equipment,
    Food,
    Spell,
}

#[derive(Serialize, Deserialize, Asset, TypePath, Debug)]
pub struct Card {
    colors: Vec<CardColor>,
    card_type: CardType,
    text: String,
    image: String,
    #[serde(skip)]
    image_handle: Handle<Image>,
}

create_ron_nested_asset_loader!(
    CardAssetLoader,
    Card,
    &["card.ron"],
    CardAssetPlugin,
    image -> image_handle
);
