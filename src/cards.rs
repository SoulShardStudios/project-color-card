use crate::create_ron_nested_asset_loader;
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
#[repr(i32)]
pub enum CardColor {
    Red,
    Yellow,
    Blue,
    Green,
    Purple,
    Teal,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Default,
    Hash,
    States,
    Reflect,
    FromPrimitive,
    ToPrimitive,
)]
#[repr(i32)]
pub enum CardType {
    #[default]
    Hero,
    Beast,
    Equipment,
    Food,
    Spell,
}

#[derive(Serialize, Deserialize, Asset, TypePath, Debug)]
pub struct Card {
    pub name: String,
    pub colors: Vec<CardColor>,
    pub card_type: CardType,
    pub text: String,
    image: String,
    #[serde(skip)]
    pub image_handle: Handle<Image>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum CardBackType {
    CardType(CardType),
    Discard,
}

#[derive(Serialize, Deserialize, Asset, TypePath, Debug, PartialEq, Eq)]
pub struct CardBack {
    pub card_type: CardBackType,
    image: String,
    #[serde(skip)]
    pub image_handle: Handle<Image>,
}

create_ron_nested_asset_loader!(
    CardAssetLoader,
    Card,
    &["card.ron"],
    CardAssetPlugin,
    image -> image_handle,
    card_assets
);

create_ron_nested_asset_loader!(
    CardBackAssetLoader,
    CardBack,
    &["back.ron"],
    CardBackAssetPlugin,
    image -> image_handle,
    card_back_assets
);