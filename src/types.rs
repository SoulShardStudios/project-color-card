use crate::create_ron_asset_loader;
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
struct Card {
    colors: Vec<CardColor>,
    card_type: CardType,
    text: String,
    image: String,
    #[serde(skip)]
    image_handle: Handle<Image>,
}
struct CardAssetLoader;
#[derive(Serialize, Deserialize, Default)]
struct CardSettings;

create_ron_asset_loader!(
    CardAssetLoader,
    Card,
    CardSettings,
    mod_name,
    &["card.ron"],
    CardAssetPlugin
);
