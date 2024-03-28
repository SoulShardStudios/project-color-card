extern crate bevy;
extern crate bevy_inspector_egui;
extern crate bevy_rand;
extern crate serde;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
mod card;
mod ron_asset_macro;
use bevy_rand::prelude::{EntropyPlugin, WyRand};
use card::{Card, CardAssetPlugin};
mod game;
use game::{setup_game_ui, CardManager};
#[derive(Resource)]
struct CardTest {
    pub _card: UntypedHandle,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(CardAssetPlugin)
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_game_ui)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, cards: Res<Assets<Card>>) {
    commands.insert_resource(CardTest {
        _card: asset_server.load_folder("cards").into(),
    });
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(CardManager::new(&cards))
}
