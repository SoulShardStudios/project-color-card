use crate::create_ron_asset_loader;
use bevy::{
    asset::{io::Reader, ron, AssetLoader, AsyncReadExt, LoadContext},
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
struct CardMetadata {
    colors: Vec<CardColor>,
    card_type: CardType,
    text: String,
}
struct CardMetadataAssetLoader;
#[derive(Serialize, Deserialize, Default)]
struct CardMetadataSettings;

create_ron_asset_loader!(
    CardMetadataAssetLoader,
    CardMetadata,
    CardMetadataSettings,
    mod_name,
    &["ron"],
    CardMetadataAssetPlugin
);

#[derive(Bundle, Default)]
struct Card {
    image: Handle<Image>,
    card_metadata: Handle<CardMetadata>,
}
